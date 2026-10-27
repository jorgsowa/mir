pub mod docblock;
pub(crate) mod type_from_hint;

use std::borrow::Cow;
use std::sync::Arc;

use php_ast::Span;
use php_rs_parser::diagnostics::{ForbiddenRule, Phase};

use crate::php_version::PhpVersion;

pub use docblock::{DocblockParser, ParsedDocblock};
pub use type_from_hint::{type_from_hint, type_from_hint_owned};

// ---------------------------------------------------------------------------
// Parse-error → Issue conversion
// ---------------------------------------------------------------------------

/// The issues for `parsed`'s diagnostics at the target `version`, without the ones
/// the analyzer reports itself or PHP does not raise at `version`.
pub(crate) fn parse_error_issues(
    file: &Arc<str>,
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> Vec<mir_issues::Issue> {
    let abstract_methods = abstract_methods_by_class(source, parsed, version);
    let backing_types = unparsed_backing_types(source, parsed, version);
    let keyword_enum = keyword_enum_name(source, parsed, version);
    let keyword_enum_start = keyword_enum.as_ref().map(|k| k.enum_start);
    parsed
        .errors
        .iter()
        .filter(|err| {
            !is_redundant_parse_error(err)
                && raised_at(err, source, &parsed.program, version)
                && !backing_types
                    .iter()
                    .any(|b| b.header.contains(&err.span().start))
                && keyword_enum_start.is_none_or(|start| err.span().start < start)
        })
        .filter_map(|err| {
            let mut issue = parse_error_to_issue(err, file, source, parsed, version);
            let grouped = matches!(
                err.forbidden_rule(),
                Some(
                    ForbiddenRule::AbstractEnumMethod | ForbiddenRule::AbstractAnonymousClassMethod
                )
            );
            if let Some(class) = abstract_methods
                .iter()
                .find(|c| grouped && c.offsets.contains(&err.span().start))
            {
                if class.offsets[0] != err.span().start {
                    return None;
                }
                if let mir_issues::IssueKind::CompileError { message, .. } = &mut issue.kind {
                    *message = class.message(version);
                }
            }
            Some(issue)
        })
        .chain(unreported_compile_errors(
            file,
            source,
            parsed,
            version,
            &backing_types,
        ))
        .chain(keyword_enum.map(|k| k.issue(file, source, parsed, version)))
        .collect()
}

/// `enum self` / `enum parent`, which the parser reads as the constant `enum` followed by a
/// keyword; PHP declares the enum and rejects its reserved name.
struct KeywordEnumName {
    /// The `enum` keyword; the parser's errors from here on are its recovery.
    enum_start: u32,
    name: Span,
}

impl KeywordEnumName {
    fn issue(
        &self,
        file: &Arc<str>,
        source: &str,
        parsed: &php_rs_parser::ParseResult,
        version: PhpVersion,
    ) -> mir_issues::Issue {
        let name = &source[self.name.start as usize..self.name.end as usize];
        let message = if version < PhpVersion::new(8, 4) {
            format!("Cannot use '{name}' as class name as it is reserved")
        } else {
            format!("Cannot use \"{name}\" as an enum name as it is reserved")
        };
        let source_map = &parsed.source_map;
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, self.name.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, self.name.end, source_map);
        mir_issues::Issue::new(
            mir_issues::IssueKind::CompileError {
                message,
                code: "reserved-enum-name".to_string(),
            },
            mir_issues::Location {
                file: file.clone(),
                line,
                line_end,
                col_start,
                col_end,
            },
        )
    }
}

fn keyword_enum_name(
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> Option<KeywordEnumName> {
    if version < PhpVersion::new(8, 1) {
        return None;
    }
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || !c.is_ascii();
    parsed.errors.iter().find_map(|err| {
        let start = err.span().start as usize;
        let after = source.get(start..)?;
        let len = after.find(|c: char| !is_ident(c)).unwrap_or(after.len());
        let name = &after[..len];
        if !name.eq_ignore_ascii_case("self") && !name.eq_ignore_ascii_case("parent") {
            return None;
        }
        let rest = after[len..].trim_start();
        let declares = rest.starts_with(['{', ':'])
            || rest
                .get(..10)
                .is_some_and(|w| w.eq_ignore_ascii_case("implements"));
        let before = source[..start].trim_end();
        let keyword = before.get(before.len().checked_sub(4)?..)?;
        let enum_start = before.len() - 4;
        let at_boundary = !before[..enum_start].ends_with(|c: char| is_ident(c) || c == '$');
        (declares && keyword.eq_ignore_ascii_case("enum") && at_boundary).then(|| KeywordEnumName {
            enum_start: enum_start as u32,
            name: Span::new(start as u32, (start + len) as u32),
        })
    })
}

/// An enum backing type the parser stops at (`?int`, `int|string`), which PHP parses
/// and rejects at compile time.
struct UnparsedBackingType {
    span: Span,
    /// From the enum name to the end of the type; the parser's errors there are replaced.
    header: std::ops::Range<u32>,
    message: String,
}

fn unparsed_backing_types(
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> Vec<UnparsedBackingType> {
    let mut found: Vec<UnparsedBackingType> = Vec::new();
    if version < PhpVersion::new(8, 1) {
        return found;
    }
    for err in &parsed.errors {
        let offset = err.span().start;
        if err.forbidden_rule().is_some() || found.iter().any(|b| b.header.contains(&offset)) {
            continue;
        }
        let Some((enum_name, _, name_end)) =
            find_decl_in_scope(&parsed.program, offset, DeclTarget::EnumHeader)
        else {
            continue;
        };
        let Some(after_name) = source.get(name_end as usize..) else {
            continue;
        };
        let Some(after_colon) = after_name.trim_start().strip_prefix(':') else {
            continue;
        };
        let type_start = source.len() - after_colon.trim_start().len();
        let mut written = after_colon.trim_start();
        written = &written[..written.find('{').unwrap_or(written.len())];
        if let Some(implements) = written.to_ascii_lowercase().find("implements") {
            written = &written[..implements];
        }
        let written = written.trim_end();
        let type_end = type_start + written.len();
        if !written.contains(['?', '|']) || !(type_start..=type_end).contains(&(offset as usize)) {
            continue;
        }
        let Some(ty) = backing_type_text(written, &enum_name, version) else {
            continue;
        };
        found.push(UnparsedBackingType {
            span: Span::new(type_start as u32, type_end as u32),
            header: name_end..type_end as u32 + 1,
            message: format!("Enum backing type must be int or string, {ty} given"),
        });
    }
    found
}

/// A written `?T` or union type as PHP prints it; class names are qualified with the enum's
/// namespace (imports are not applied).
fn backing_type_text(written: &str, enum_name: &str, version: PhpVersion) -> Option<String> {
    use mir_types::{Atomic, Type};
    let (nullable, union) = match written.strip_prefix('?') {
        Some(rest) => (true, rest.trim_start()),
        None => (false, written),
    };
    let namespace = enum_name.rsplit_once('\\').map_or("", |(ns, _)| ns);
    let named = |fqcn: &str| Atomic::TNamedObject {
        fqcn: mir_types::Name::from(fqcn),
        type_params: Default::default(),
    };
    let array = || Atomic::TArray {
        key: Box::new(Type::array_key()),
        value: Box::new(Type::mixed()),
    };
    let mut atomics = Vec::new();
    for part in union.split('|').map(str::trim) {
        let is_name = !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '\\' || !c.is_ascii());
        if !is_name {
            return None;
        }
        let atomic = match part.to_ascii_lowercase().as_str() {
            "int" => Atomic::TInt,
            "string" => Atomic::TString,
            "float" => Atomic::TFloat,
            "bool" => Atomic::TBool,
            "false" => Atomic::TFalse,
            "true" => Atomic::TTrue,
            "null" => Atomic::TNull,
            "array" => array(),
            "object" => Atomic::TObject,
            "mixed" => Atomic::TMixed,
            "void" => Atomic::TVoid,
            "never" => Atomic::TNever,
            "iterable" => {
                atomics.push(array());
                named("Traversable")
            }
            "self" => named(enum_name),
            "callable" | "static" | "parent" => return None,
            _ => match part.strip_prefix('\\') {
                Some(fully_qualified) => named(fully_qualified),
                None if namespace.is_empty() => named(part),
                None => named(&format!("{namespace}\\{part}")),
            },
        };
        atomics.push(atomic);
    }
    if nullable {
        atomics.push(Atomic::TNull);
    }
    Some(crate::util::php_type_text(
        &Type::from_vec(atomics),
        version,
    ))
}

/// Errors PHP raises at `version` that the parser does not report.
fn unreported_compile_errors(
    file: &Arc<str>,
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
    backing_types: &[UnparsedBackingType],
) -> Vec<mir_issues::Issue> {
    let issue_at = |kind, span: Span| {
        let source_map = &parsed.source_map;
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, span.end, source_map);
        mir_issues::Issue::new(
            kind,
            mir_issues::Location {
                file: file.clone(),
                line,
                line_end,
                col_start,
                col_end,
            },
        )
    };
    let compile_error = |message: String, code: &str| mir_issues::IssueKind::CompileError {
        message,
        code: code.to_string(),
    };
    let mut finder = UnreportedDecls::default();
    for stmt in &parsed.program.stmts {
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut finder, stmt);
    }
    let mut issues: Vec<_> = backing_types
        .iter()
        .map(|b| {
            issue_at(
                compile_error(b.message.clone(), "invalid-enum-backing-type"),
                b.span,
            )
        })
        .collect();
    for (name, span) in finder.trait_names {
        let lower = name.to_ascii_lowercase();
        let reserved = match lower.as_str() {
            "mixed" => version >= PhpVersion::new(8, 0),
            "never" => version >= PhpVersion::new(8, 1),
            other => RESERVED_CLASS_NAMES.contains(&other),
        };
        if !reserved {
            continue;
        }
        let message = if version < PhpVersion::new(8, 4) {
            format!("Cannot use '{name}' as class name as it is reserved")
        } else {
            format!("Cannot use \"{name}\" as a trait name as it is reserved")
        };
        issues.push(issue_at(
            compile_error(message, "keyword-as-declaration-name"),
            span,
        ));
    }
    // From 8.0 `__clone()` may declare `void`, which the parser accepts on every version.
    if version < PhpVersion::new(8, 0) {
        for (method_start, return_span) in finder.clone_return_types {
            let Some((class, method)) =
                find_decl(&parsed.program, method_start, DeclTarget::Method)
            else {
                continue;
            };
            issues.push(issue_at(
                compile_error(
                    format!("Clone method {class}::{method}() cannot declare a return type"),
                    "magic-method-return-type",
                ),
                return_span,
            ));
        }
    }
    // Before property hooks `parent::$x::get()` is a static call on `parent::$x`.
    if version < PhpVersion::new(8, 4) {
        for err in &parsed.errors {
            if err.forbidden_rule() != Some(ForbiddenRule::InvalidParentHookCall) {
                continue;
            }
            let start = err.span().start;
            let mut scope = EnclosingClass {
                offset: start,
                has_parent: None,
            };
            for stmt in &parsed.program.stmts {
                let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut scope, stmt);
            }
            if scope.has_parent == Some(false) {
                let end = start + "parent".len() as u32;
                issues.push(issue_at(
                    mir_issues::IssueKind::ParentNotFound,
                    Span::new(start, end.min(err.span().end)),
                ));
            }
        }
    }
    issues
}

/// Trait names, and the return types of `__clone()` methods, for checks the parser skips.
#[derive(Default)]
struct UnreportedDecls {
    trait_names: Vec<(String, Span)>,
    /// Method start and return type span; PHP 7.4 skips interface methods.
    clone_return_types: Vec<(u32, Span)>,
    in_interface: bool,
}

impl php_ast::owned::visitor::OwnedVisitor for UnreportedDecls {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        use php_ast::owned::StmtKind;
        if let StmtKind::Trait(t) = &stmt.kind {
            if let Some(name) = t.name.as_str() {
                self.trait_names.push((name.to_string(), t.name.span));
            }
        }
        let outer = self.in_interface;
        match &stmt.kind {
            StmtKind::Interface(_) => self.in_interface = true,
            StmtKind::Class(_) | StmtKind::Trait(_) | StmtKind::Enum(_) => {
                self.in_interface = false
            }
            _ => {}
        }
        let flow = crate::recursion::walk_stmt(self, stmt);
        self.in_interface = outer;
        flow
    }

    fn visit_class_member(
        &mut self,
        member: &php_ast::owned::ClassMember,
    ) -> std::ops::ControlFlow<()> {
        if let php_ast::owned::ClassMemberKind::Method(m) = &member.kind {
            if let (Some(rt), false, true) = (
                &m.return_type,
                self.in_interface,
                m.name
                    .as_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case("__clone")),
            ) {
                self.clone_return_types.push((member.span.start, rt.span));
            }
        }
        php_ast::owned::visitor::walk_owned_class_member(self, member)
    }

    fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
        let outer = self.in_interface;
        if let php_ast::owned::ExprKind::AnonymousClass(_) = &expr.kind {
            self.in_interface = false;
        }
        let flow = crate::recursion::walk_expr(self, expr);
        self.in_interface = outer;
        flow
    }
}

/// Whether the innermost class-like around `offset` is a class with a parent (`None`: not a class).
struct EnclosingClass {
    offset: u32,
    has_parent: Option<bool>,
}

impl EnclosingClass {
    fn contains(&self, span: Span) -> bool {
        span.start <= self.offset && self.offset < span.end
    }
}

impl php_ast::owned::visitor::OwnedVisitor for EnclosingClass {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        use php_ast::owned::StmtKind;
        if !self.contains(stmt.span) {
            return std::ops::ControlFlow::Continue(());
        }
        match &stmt.kind {
            StmtKind::Class(c) => self.has_parent = Some(c.extends.is_some()),
            StmtKind::Trait(_) | StmtKind::Interface(_) | StmtKind::Enum(_) => {
                self.has_parent = None
            }
            _ => {}
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
        if !self.contains(expr.span) {
            return std::ops::ControlFlow::Continue(());
        }
        if let php_ast::owned::ExprKind::AnonymousClass(c) = &expr.kind {
            self.has_parent = Some(c.extends.is_some());
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// A class-like's abstract methods that PHP reports together, before 8.5.
struct AbstractMethods {
    class: String,
    is_enum: bool,
    methods: Vec<String>,
    /// Offsets of the parser's per-method diagnostics.
    offsets: Vec<u32>,
}

impl AbstractMethods {
    fn message(&self, version: PhpVersion) -> String {
        let count = self.methods.len();
        let plural = if count == 1 { "" } else { "s" };
        let mut list: Vec<String> = self
            .methods
            .iter()
            .take(3)
            .map(|m| format!("{}::{m}", self.class))
            .collect();
        if count > 3 {
            list.push("...".to_string());
        }
        let list = list.join(", ");
        if self.is_enum && version >= PhpVersion::new(8, 2) {
            format!(
                "Enum {} must implement {count} abstract private method{plural} ({list})",
                self.class
            )
        } else {
            format!(
                "Class {} contains {count} abstract method{plural} and must therefore be declared abstract or implement the remaining methods ({list})",
                self.class
            )
        }
    }
}

/// Before 8.5 PHP aggregates the abstract methods of an enum or anonymous class into one error.
fn abstract_methods_by_class(
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> Vec<AbstractMethods> {
    let mut classes: Vec<(u32, AbstractMethods)> = Vec::new();
    if version >= PhpVersion::new(8, 5) {
        return Vec::new();
    }
    for err in &parsed.errors {
        let is_enum = match err.forbidden_rule() {
            Some(ForbiddenRule::AbstractEnumMethod) => true,
            Some(ForbiddenRule::AbstractAnonymousClassMethod) => false,
            _ => continue,
        };
        // `final abstract` is PHP's modifier error, reported on its own.
        if is_enum
            && php_message(err, source, &parsed.program, version)
                .is_some_and(|m| m.contains("final modifier"))
        {
            continue;
        }
        let start = err.span().start;
        let Some((class, method, scope)) =
            find_decl_in_scope(&parsed.program, start, DeclTarget::Method)
        else {
            continue;
        };
        match classes.iter_mut().find(|(s, _)| *s == scope) {
            Some((_, group)) => {
                group.methods.push(method);
                group.offsets.push(start);
            }
            None => classes.push((
                scope,
                AbstractMethods {
                    class,
                    is_enum,
                    methods: vec![method],
                    offsets: vec![start],
                },
            )),
        }
    }
    classes.into_iter().map(|(_, group)| group).collect()
}

/// Convert a parser diagnostic to a [`mir_issues::Issue`], using the source
/// and source map to derive a precise location.
fn parse_error_to_issue(
    err: &php_rs_parser::diagnostics::ParseError,
    file: &Arc<str>,
    source: &str,
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> mir_issues::Issue {
    let span = err.span();
    let source_map = &parsed.source_map;
    let (line, col_start) = crate::diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = crate::diagnostics::offset_to_line_col(source, span.end, source_map);

    let message =
        php_message(err, source, &parsed.program, version).unwrap_or_else(|| err.to_string());
    let code = err.code().to_owned();
    let rule = err.forbidden_rule();
    let kind = if rule == Some(ForbiddenRule::ParentWithoutParentClass) {
        mir_issues::IssueKind::ParentNotFound
    } else if is_compile_error(err, source, &parsed.program, version) {
        mir_issues::IssueKind::CompileError { message, code }
    } else {
        mir_issues::IssueKind::ParseError { message, code }
    };
    mir_issues::Issue::new(
        kind,
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end,
        },
    )
}

/// Whether PHP rejects the parser's diagnostic at compile time (`php -l` prints `Fatal error`).
fn is_compile_error(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &str,
    program: &php_ast::owned::Program,
    version: PhpVersion,
) -> bool {
    let Some(rule) = err.forbidden_rule() else {
        return false;
    };
    match rule {
        ForbiddenRule::MultipleVisibilityModifiers | ForbiddenRule::DuplicateModifier
            if version < PhpVersion::new(8, 1) =>
        {
            promoted_param_modifier(err, source, program).is_none()
        }
        ForbiddenRule::MultipleVisibilityModifiers
        | ForbiddenRule::MultipleSetVisibilityModifiers
        | ForbiddenRule::DuplicateModifier
        | ForbiddenRule::AbstractFinalConflict
        | ForbiddenRule::InvalidConstantModifier
        | ForbiddenRule::OutermostScopeOnly
        | ForbiddenRule::PartialApplicationInNew => true,
        // A parameter modifier is a syntax error before 8.3, `readonly` before 8.1.
        ForbiddenRule::ModifierNotAllowed => {
            version >= PhpVersion::new(8, 3)
                || version >= PhpVersion::new(8, 1) && err.to_string().ends_with(" on a method")
        }
        ForbiddenRule::InvalidCurlyInterpolation => false,
        ForbiddenRule::CurlyOffsetAccess => version < PhpVersion::new(8, 4),
        ForbiddenRule::ReservedClassName | ForbiddenRule::KeywordAsDeclarationName => {
            reserved_declaration_name(err, source).is_some()
        }
        // `$x = &1` is a syntax error; `[&$a] = 1` a compile error.
        ForbiddenRule::ReferenceToNonReferenceable => !preceded_by(source, err.span(), '&'),
        _ => rule.phase() == Phase::Compile,
    }
}

/// Whether PHP raises the parser's diagnostic at `version`.
fn raised_at(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &str,
    program: &php_ast::owned::Program,
    version: PhpVersion,
) -> bool {
    match err.forbidden_rule() {
        // Before 8.0 PHP checks only that these three declare no return type.
        Some(ForbiddenRule::MagicMethodReturnType) if version < PhpVersion::new(8, 0) => {
            let message = err.to_string();
            message.ends_with("cannot declare a return type")
                || message.to_ascii_lowercase().contains("::__clone(): ")
        }
        Some(ForbiddenRule::MagicMethodParameterType) => version >= PhpVersion::new(8, 0),
        // Before property hooks `parent::$x::get()` calls `get()` on the class `parent::$x` names.
        Some(ForbiddenRule::InvalidParentHookCall) => version >= PhpVersion::new(8, 4),
        // PHP 8.0 lets a nullsafe chain be destructured by reference.
        Some(ForbiddenRule::ReferenceToNullsafe) if version < PhpVersion::new(8, 1) => {
            !preceded_by(source, err.span(), '=')
        }
        // Promotion before 8.0 and `readonly` before 8.1 are already `version-too-low`.
        Some(ForbiddenRule::MultipleVisibilityModifiers | ForbiddenRule::DuplicateModifier)
            if version < PhpVersion::new(8, 1)
                && promoted_param_modifier(err, source, program).is_some() =>
        {
            version >= PhpVersion::new(8, 0)
                && promoted_param_modifier(err, source, program)
                    .is_some_and(|m| !m.eq_ignore_ascii_case("readonly"))
        }
        // Before 8.1 `enum` is a syntax error; the enum's `version-too-low` covers it.
        Some(
            ForbiddenRule::AbstractEnumMethod
            | ForbiddenRule::EnumCaseNamedClass
            | ForbiddenRule::EnumProperty
            | ForbiddenRule::InvalidEnumBackingType
            | ForbiddenRule::MissingEnumCaseValue
            | ForbiddenRule::PureEnumCaseValue
            | ForbiddenRule::ReservedEnumName,
        ) => version >= PhpVersion::new(8, 1),
        // Before 8.0 the `dynamic-class-fetch` error covers it.
        Some(ForbiddenRule::ClassFetchOnLiteral) => version >= PhpVersion::new(8, 0),
        // Reported next to the modifier error for `readonly static function`.
        Some(ForbiddenRule::StaticReadonlyProperty) => {
            find_decl(program, err.span().start, DeclTarget::Method).is_none()
        }
        _ => true,
    }
}

/// The repeated modifier of a promoted parameter the parser rejects, which
/// PHP 8.0's grammar reads as a syntax error (8.1 compiles modifiers as a list).
fn promoted_param_modifier<'a>(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &'a str,
    program: &php_ast::owned::Program,
) -> Option<&'a str> {
    let span = err.span();
    find_decl(program, span.start, DeclTarget::Param)?;
    // `duplicate-modifier` marks the repeat, `multiple-visibility-modifiers` the first.
    if err.forbidden_rule() == Some(ForbiddenRule::DuplicateModifier) {
        return source.get(span.start as usize..span.end as usize);
    }
    let after = source.get(span.end as usize..)?.trim_start();
    let len = after
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    Some(&after[..len]).filter(|word| !word.is_empty())
}

/// Whether the first non-blank character before `span` is `expected`.
fn preceded_by(source: &str, span: Span, expected: char) -> bool {
    source
        .get(..span.start as usize)
        .and_then(|before| before.trim_end().chars().next_back())
        == Some(expected)
}

/// Reserved type names PHP rejects as class-like names at compile time; keywords are syntax errors.
const RESERVED_CLASS_NAMES: [&str; 14] = [
    "bool", "false", "float", "int", "null", "parent", "self", "string", "true", "void", "never",
    "iterable", "object", "mixed",
];

/// The name and declaration kind (`class`, `interface`, `trait`, `enum`) of a class-like
/// declared with a reserved type name.
fn reserved_declaration_name<'a>(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &'a str,
) -> Option<(&'a str, &'static str)> {
    let span = err.span();
    let name = source.get(span.start as usize..span.end as usize)?;
    if !RESERVED_CLASS_NAMES.contains(&name.to_ascii_lowercase().as_str()) {
        return None;
    }
    let message = err.to_string();
    ["class", "interface", "trait", "enum"]
        .into_iter()
        .find(|kind| {
            message.contains(&format!(" as {kind} name"))
                || message.contains(&format!(" as a {kind} name"))
                || message.contains(&format!(" as an {kind} name"))
        })
        .map(|kind| (name, kind))
}

/// PHP's wording for parser diagnostics whose text differs from PHP's at `version`.
fn php_message(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &str,
    program: &php_ast::owned::Program,
    version: PhpVersion,
) -> Option<String> {
    let parser_message = || err.to_string();
    let before = |major, minor| version < PhpVersion::new(major, minor);
    let (target, message): (DeclTarget, fn(&str, &str) -> String) = match err.forbidden_rule()? {
        ForbiddenRule::ReadonlyPropertyHooks => {
            return Some("Hooked properties cannot be readonly".to_string())
        }
        ForbiddenRule::AbstractFinalConflict => {
            let message = match parser_message() {
                m if m.starts_with("Cannot use") => m,
                _ if find_decl(program, err.span().start, DeclTarget::Method).is_some() => {
                    "Cannot use the final modifier on an abstract method".to_string()
                }
                _ => "Cannot use the final modifier on an abstract property".to_string(),
            };
            if before(8, 3) && message.ends_with("an abstract method") {
                return Some(
                    "Cannot use the final modifier on an abstract class member".to_string(),
                );
            }
            return Some(message);
        }
        ForbiddenRule::InvalidConstantModifier => {
            let message = parser_message();
            let modifier = message.split('\'').nth(1)?;
            return Some(if before(8, 3) {
                format!("Cannot use '{modifier}' as constant modifier")
            } else {
                format!("Cannot use the {modifier} modifier on a class constant")
            });
        }
        ForbiddenRule::InvalidInterfaceMethodModifier if before(8, 1) => {
            let (interface, method) = find_decl(program, err.span().start, DeclTarget::Method)?;
            return Some(format!(
                "Access type for interface method {interface}::{method}() must be omitted"
            ));
        }
        ForbiddenRule::ModifierNotAllowed if before(8, 3) => {
            let message = parser_message();
            let modifier = message
                .strip_prefix("Cannot use the ")?
                .strip_suffix(" modifier on a method")?;
            return Some(format!("Cannot use '{modifier}' as method modifier"));
        }
        ForbiddenRule::InvalidTraitAliasModifier if before(8, 3) => {
            let message = parser_message();
            let modifier = message.split('"').nth(1)?;
            return Some(format!("Cannot use '{modifier}' as method modifier"));
        }
        ForbiddenRule::AbstractPropertyWithoutAbstractHook => {
            return (parser_message() == "properties cannot be abstract").then(|| {
                if before(8, 3) {
                    "Properties cannot be declared abstract"
                } else if before(8, 4) {
                    "Cannot use the abstract modifier on a property"
                } else {
                    "Only hooked properties may be declared abstract"
                }
                .to_string()
            });
        }
        ForbiddenRule::InterfacePropertyWithoutHooks if before(8, 4) => {
            return Some(if before(8, 1) {
                "Interfaces may not include member variables".to_string()
            } else {
                "Interfaces may not include properties".to_string()
            });
        }
        ForbiddenRule::ReservedClassName
        | ForbiddenRule::KeywordAsDeclarationName
        | ForbiddenRule::ReservedEnumName => {
            let (name, kind) = reserved_declaration_name(err, source)?;
            return Some(if before(8, 4) {
                format!("Cannot use '{name}' as class name as it is reserved")
            } else {
                let article = if kind.starts_with(['a', 'e', 'i']) {
                    "an"
                } else {
                    "a"
                };
                format!("Cannot use \"{name}\" as {article} {kind} name as it is reserved")
            });
        }
        ForbiddenRule::ReservedClassReference if before(8, 4) => {
            let message = parser_message().replace('"', "'");
            return Some(if before(8, 0) {
                message.replace(", as it", " as it")
            } else {
                message
            });
        }
        ForbiddenRule::ReservedTypeName if before(8, 4) => {
            let message = parser_message();
            let name = message.split('"').nth(1)?;
            return Some(format!(
                "Cannot use '{name}' as class name as it is reserved"
            ));
        }
        ForbiddenRule::PartialApplicationInNew => {
            return Some("Cannot create Closure for new expression".to_string())
        }
        ForbiddenRule::MultipleVisibilityModifiers | ForbiddenRule::DuplicateModifier
            if before(8, 1) =>
        {
            let token = promoted_param_modifier(err, source, program)?;
            return Some(format!(
                "syntax error, unexpected token \"{}\", expecting variable",
                token.to_ascii_lowercase()
            ));
        }
        ForbiddenRule::ReferenceToNonReferenceable if before(8, 1) => {
            return Some("Cannot assign reference to non referencable value".to_string())
        }
        ForbiddenRule::MagicMethodReturnType if before(8, 0) => {
            let message = parser_message();
            let lower = message.to_ascii_lowercase();
            let method = message
                .split(' ')
                .find(|word| word.contains("::"))?
                .trim_end_matches(':');
            let label = if lower.contains("::__construct()") {
                "Constructor"
            } else if lower.contains("::__destruct()") {
                "Destructor"
            } else {
                "Clone method"
            };
            return Some(format!("{label} {method} cannot declare a return type"));
        }
        ForbiddenRule::GeneratorReturnType if before(8, 0) => {
            let message = parser_message();
            let ty = message.rsplit(", ").next()?.strip_suffix(" given")?;
            return Some(format!(
                "Generators may only declare a return type of Generator, Iterator, Traversable, or iterable, {ty} is not permitted"
            ));
        }
        ForbiddenRule::DeprecatedOnInvalidTarget if version == PhpVersion::new(8, 4) => {
            return Some(format!(
                "Attribute \"Deprecated\" cannot target class (allowed targets: {DEPRECATED_TARGETS_84})"
            ));
        }
        ForbiddenRule::AttributeTargetMismatch if version == PhpVersion::new(8, 4) => {
            let message = parser_message();
            let target = message
                .strip_prefix("Attribute \"Deprecated\" cannot target ")?
                .split(" (")
                .next()?;
            return Some(format!(
                "Attribute \"Deprecated\" cannot target {target} (allowed targets: {DEPRECATED_TARGETS_84})"
            ));
        }
        ForbiddenRule::AllowDynamicPropertiesOnInvalidClass if before(8, 5) => {
            let message = parser_message().replace("#[\\", "#[");
            if before(8, 4) && !message.contains(" readonly class ") {
                return Some(message.rsplit_once(' ')?.0.to_string());
            }
            return Some(message);
        }
        ForbiddenRule::AbstractMemberInConcreteClass if before(8, 5) => {
            return Some(
                parser_message().replace("the remaining method (", "the remaining methods ("),
            );
        }
        ForbiddenRule::VirtualPropertyAsymmetricVisibility if !before(8, 6) => {
            let message = parser_message();
            return Some(match message.strip_prefix("Read-only ") {
                Some(rest) => format!("get-only {rest}"),
                None => message,
            });
        }
        ForbiddenRule::InvalidNullableType if before(8, 2) => {
            let message = parser_message();
            // `null` is not a type before 8.0.
            if before(8, 0) && message.starts_with("null ") {
                let name = source.get(err.span().start as usize..err.span().end as usize)?;
                return Some(format!(
                    "Cannot use '{name}' as class name as it is reserved"
                ));
            }
            return Some(if message.starts_with("null ") {
                "Null can not be used as a standalone type".to_string()
            } else {
                message
            });
        }
        ForbiddenRule::StandaloneTypeInComposite if before(8, 0) => {
            let message = parser_message();
            if !message.starts_with("Void ") {
                return Some(message);
            }
            let after = source.get(err.span().end as usize..).unwrap_or_default();
            return Some(if after.trim_start().starts_with(['$', '&', '.']) {
                "void cannot be used as a parameter type".to_string()
            } else {
                "Void type cannot be nullable".to_string()
            });
        }
        ForbiddenRule::RedundantType if before(8, 4) => {
            return Some(parser_message().replace(
                ", bool must be used instead",
                ", bool should be used instead",
            ));
        }
        ForbiddenRule::ClassFetchOnLiteral if before(8, 3) => {
            return Some(
                parser_message().replace("\"::class\" on ", "\"::class\" on value of type "),
            );
        }
        ForbiddenRule::PureEnumCaseValue if version == PhpVersion::new(8, 1) => {
            let (enum_name, case) = find_decl(program, err.span().start, DeclTarget::EnumCase)?;
            let mut message =
                format!("Case {case} of non-backed enum {enum_name} must not have a value");
            if let Some(ty) = literal_case_value_type(source, err.span()) {
                message.push_str(&format!(", try adding \": {ty}\" to the enum declaration"));
            }
            return Some(message);
        }
        ForbiddenRule::DuplicateType => {
            let message = parser_message();
            let written = message.split('\'').nth(1)?;
            // A relative class name would need the file's imports to print as PHP does.
            let name = match written.strip_prefix('\\') {
                Some(fully_qualified) => fully_qualified.to_string(),
                None if BUILTIN_TYPE_NAMES.contains(&written.to_ascii_lowercase().as_str()) => {
                    written.to_ascii_lowercase()
                }
                None => return None,
            };
            return Some(format!("Duplicate type {name} is redundant"));
        }
        ForbiddenRule::AppendOutsideWrite => {
            let message = parser_message();
            let mut chars = message.chars();
            let first = chars.next()?;
            return Some(first.to_uppercase().chain(chars).collect());
        }
        ForbiddenRule::DuplicateMatchDefault => {
            return Some("Match expressions may only contain one default arm".to_string())
        }
        ForbiddenRule::RealCast => {
            return Some("The (real) cast has been removed, use (float) instead".to_string())
        }
        ForbiddenRule::UnsetCast => {
            return Some("The (unset) cast is no longer supported".to_string())
        }
        ForbiddenRule::YieldOutsideFunction => {
            return Some("The \"yield\" expression can only be used inside a function".to_string())
        }
        ForbiddenRule::InvalidCodepointEscape => {
            let message = parser_message();
            return message
                .strip_suffix(": empty code point")
                .map(str::to_string);
        }
        ForbiddenRule::HeredocBodyIndentation => {
            let indent = heredoc_closing_indent(source, err.span())?;
            return Some(format!(
                "Invalid body indentation level (expecting an indentation level of at least {indent})"
            ));
        }
        ForbiddenRule::UnterminatedComment => {
            let line = source[..source.len().min(err.span().start as usize)]
                .matches('\n')
                .count()
                + 1;
            return Some(format!("Unterminated comment starting line {line}"));
        }
        ForbiddenRule::GetHookParameters => (DeclTarget::Property, |class, prop| {
            format!("get hook of property {class}::${prop} must not have a parameter list")
        }),
        ForbiddenRule::SetHookParameterCount => (DeclTarget::Property, |class, prop| {
            format!("set hook of property {class}::${prop} must accept exactly one parameters")
        }),
        ForbiddenRule::SetVisibilityWiderThanVisibility => {
            (DeclTarget::Property, |class, prop| {
                format!("Visibility of property {class}::${prop} must not be weaker than set visibility")
            })
        }
        ForbiddenRule::PrivateFinalConstant => (DeclTarget::ClassConst, |class, name| {
            format!(
                "Private constant {class}::{name} cannot be final as it is not visible to other classes"
            )
        }),
        ForbiddenRule::EnumCaseNamedClass => return Some(
            "A class constant must not be called 'class'; it is reserved for class name fetching"
                .to_string(),
        ),
        ForbiddenRule::PureEnumCaseValue => (DeclTarget::EnumCase, |enum_name, case| {
            format!("Case {case} of non-backed enum {enum_name} must not have a value")
        }),
        ForbiddenRule::MissingEnumCaseValue => (DeclTarget::EnumCase, |enum_name, case| {
            format!("Case {case} of backed enum {enum_name} must have a value")
        }),
        ForbiddenRule::UntypedReadonlyProperty => (DeclTarget::Property, |class, prop| {
            format!("Readonly property {class}::${prop} must have type")
        }),
        ForbiddenRule::StaticReadonlyProperty => (DeclTarget::Property, |class, prop| {
            format!("Static property {class}::${prop} cannot be readonly")
        }),
        // PHP checks the modifiers first; the span covers them.
        ForbiddenRule::AbstractEnumMethod
            if source
                .get(err.span().start as usize..err.span().end as usize)
                .is_some_and(|modifiers| {
                    modifiers
                        .split_whitespace()
                        .any(|m| m.eq_ignore_ascii_case("final"))
                }) =>
        {
            return Some(if before(8, 3) {
                "Cannot use the final modifier on an abstract class member".to_string()
            } else {
                "Cannot use the final modifier on an abstract method".to_string()
            });
        }
        ForbiddenRule::AbstractEnumMethod => (DeclTarget::Method, |enum_name, method| {
            format!("Enum method {enum_name}::{method}() must not be abstract")
        }),
        ForbiddenRule::InvalidEnumBackingType => (DeclTarget::EnumBackingType, |_, ty| {
            format!("Enum backing type must be int or string, {ty} given")
        }),
        _ => return None,
    };
    let (class, name) = find_decl(program, err.span().start, target)?;
    Some(message(&class, &name))
}

/// `#[\Deprecated]`'s targets in PHP 8.4; 8.5 adds classes and constants.
const DEPRECATED_TARGETS_84: &str = "function, method, class constant";

/// `int` or `string` for a literal enum case value after the `=` at `eq`.
fn literal_case_value_type(source: &str, eq: Span) -> Option<&'static str> {
    let value = source.get(eq.end as usize..)?.split(';').next()?.trim();
    if value.starts_with(['\'', '"']) {
        return Some("string");
    }
    let digits = value.strip_prefix('-').unwrap_or(value);
    let is_int = digits.starts_with(|c: char| c.is_ascii_digit())
        && digits
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && (digits.starts_with("0x") || digits.starts_with("0X") || !digits.contains(['e', 'E']));
    is_int.then_some("int")
}

/// Indentation of the closing marker of the heredoc whose body line is `line`.
fn heredoc_closing_indent(source: &str, line: Span) -> Option<usize> {
    let is_label_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80;
    let opening = source.get(..line.start as usize)?.rfind("<<<")? + 3;
    let label = source[opening..].trim_start_matches([' ', '\t', '"', '\'']);
    let label = &label[..label.bytes().take_while(|&b| is_label_byte(b)).count()];
    if label.is_empty() {
        return None;
    }
    source.get(line.end as usize..)?.lines().find_map(|text| {
        let body = text.trim_start_matches([' ', '\t']);
        let rest = body.strip_prefix(label)?;
        (!rest.bytes().next().is_some_and(is_label_byte)).then(|| text.len() - body.len())
    })
}

/// The class-like and name of the `target` declared at `offset`.
fn find_decl(
    program: &php_ast::owned::Program,
    offset: u32,
    target: DeclTarget,
) -> Option<(String, String)> {
    find_decl_in_scope(program, offset, target).map(|(class, name, _)| (class, name))
}

/// [`find_decl`] plus the start offset of the enclosing class-like.
fn find_decl_in_scope(
    program: &php_ast::owned::Program,
    offset: u32,
    target: DeclTarget,
) -> Option<(String, String, u32)> {
    let mut finder = DeclAt {
        offset,
        target,
        namespace: String::new(),
        classes: Vec::new(),
        found: None,
    };
    for stmt in &program.stmts {
        if php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut finder, stmt).is_break() {
            break;
        }
    }
    finder.found
}

/// Type keywords PHP prints in lowercase in its messages.
const BUILTIN_TYPE_NAMES: [&str; 17] = [
    "int", "float", "string", "bool", "void", "never", "mixed", "object", "iterable", "callable",
    "array", "null", "true", "false", "self", "parent", "static",
];

#[derive(PartialEq)]
enum DeclTarget {
    /// A property or promoted parameter.
    Property,
    /// A method parameter.
    Param,
    Method,
    ClassConst,
    EnumCase,
    EnumBackingType,
    /// An enum declaration; found with the end of its name in place of the enum's start.
    EnumHeader,
}

/// Finds the class-like enclosing the `target` declared at `offset`, and the target's name.
struct DeclAt {
    offset: u32,
    target: DeclTarget,
    namespace: String,
    /// Enclosing class-likes and their start offsets.
    classes: Vec<(String, u32)>,
    found: Option<(String, String, u32)>,
}

impl DeclAt {
    fn qualify(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{name}", self.namespace)
        }
    }

    fn in_class(&mut self, name: &str, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        self.classes.push((self.qualify(name), stmt.span.start));
        let flow = crate::recursion::walk_stmt(self, stmt);
        self.classes.pop();
        flow
    }

    fn found(
        &mut self,
        target: DeclTarget,
        span: Span,
        name: &php_ast::owned::Ident,
    ) -> std::ops::ControlFlow<()> {
        if target != self.target || span.start > self.offset || self.offset >= span.end {
            return std::ops::ControlFlow::Continue(());
        }
        let (class, scope) = self.classes.last().cloned().unwrap_or_default();
        self.found = Some((class, name.as_str().unwrap_or_default().to_string(), scope));
        std::ops::ControlFlow::Break(())
    }

    /// The backing type as PHP names it: a builtin lowercased, a class name qualified.
    fn backing_type_name(&self, name: &php_ast::owned::Name) -> String {
        let written = name_to_string_owned(name);
        if let Some(fully_qualified) = written.strip_prefix('\\') {
            return fully_qualified.to_string();
        }
        let lower = written.to_ascii_lowercase();
        if BUILTIN_TYPE_NAMES.contains(&lower.as_str()) {
            lower
        } else {
            self.qualify(&written)
        }
    }
}

impl php_ast::owned::visitor::OwnedVisitor for DeclAt {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        use php_ast::owned::{NamespaceBody, StmtKind};
        // An unbraced `namespace X;` ends before the code it applies to.
        let unbraced_namespace = matches!(&stmt.kind, StmtKind::Namespace(ns) if !matches!(ns.body, NamespaceBody::Braced(_)));
        if stmt.span.end <= self.offset && !unbraced_namespace {
            return std::ops::ControlFlow::Continue(());
        }
        match &stmt.kind {
            StmtKind::Namespace(ns) => {
                let name = ns
                    .name
                    .as_ref()
                    .map(|n| name_to_string_owned(n).into_owned())
                    .unwrap_or_default();
                if let NamespaceBody::Braced(_) = &ns.body {
                    let outer = std::mem::replace(&mut self.namespace, name);
                    let flow = crate::recursion::walk_stmt(self, stmt);
                    self.namespace = outer;
                    return flow;
                }
                self.namespace = name;
            }
            StmtKind::Class(c) => {
                let name = c.name.as_ref().and_then(|n| n.as_str()).unwrap_or_default();
                return self.in_class(name, stmt);
            }
            StmtKind::Trait(d) => return self.in_class(d.name.as_str().unwrap_or_default(), stmt),
            StmtKind::Interface(d) => {
                return self.in_class(d.name.as_str().unwrap_or_default(), stmt)
            }
            StmtKind::Enum(d) => {
                let name = d.name.as_str().unwrap_or_default();
                if let Some(ty) = d.scalar_type.as_ref().filter(|ty| {
                    self.target == DeclTarget::EnumBackingType && ty.span().start == self.offset
                }) {
                    self.found = Some((
                        self.qualify(name),
                        self.backing_type_name(ty),
                        stmt.span.start,
                    ));
                    return std::ops::ControlFlow::Break(());
                }
                if self.target == DeclTarget::EnumHeader && stmt.span.start <= self.offset {
                    self.found = Some((self.qualify(name), String::new(), d.name.span.end));
                    return std::ops::ControlFlow::Break(());
                }
                return self.in_class(name, stmt);
            }
            _ => {}
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_class_member(
        &mut self,
        member: &php_ast::owned::ClassMember,
    ) -> std::ops::ControlFlow<()> {
        use php_ast::owned::ClassMemberKind;
        match &member.kind {
            ClassMemberKind::Property(p) => {
                self.found(DeclTarget::Property, member.span, &p.name)?
            }
            ClassMemberKind::ClassConst(c) => {
                self.found(DeclTarget::ClassConst, member.span, &c.name)?
            }
            ClassMemberKind::Method(m) => {
                self.found(DeclTarget::Method, member.span, &m.name)?;
                for param in m.params.iter() {
                    self.found(DeclTarget::Property, param.span, &param.name)?;
                    self.found(DeclTarget::Param, param.span, &param.name)?;
                }
            }
            _ => {}
        }
        php_ast::owned::visitor::walk_owned_class_member(self, member)
    }

    fn visit_enum_member(
        &mut self,
        member: &php_ast::owned::EnumMember,
    ) -> std::ops::ControlFlow<()> {
        use php_ast::owned::EnumMemberKind;
        match &member.kind {
            EnumMemberKind::Method(m) => self.found(DeclTarget::Method, member.span, &m.name)?,
            EnumMemberKind::Case(c) => self.found(DeclTarget::EnumCase, member.span, &c.name)?,
            EnumMemberKind::ClassConst(c) => {
                self.found(DeclTarget::ClassConst, member.span, &c.name)?
            }
            _ => {}
        }
        php_ast::owned::visitor::walk_owned_enum_member(self, member)
    }

    fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
        if let php_ast::owned::ExprKind::AnonymousClass(_) = &expr.kind {
            self.classes
                .push(("class@anonymous".to_string(), expr.span.start));
            let flow = crate::recursion::walk_expr(self, expr);
            self.classes.pop();
            return flow;
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// Compile-time rules the parser checks that the analyzer reports itself with a
/// dedicated issue kind; the parser's diagnostic is dropped as a duplicate.
const ANALYZER_COVERED_RULES: &[ForbiddenRule] = &[
    ForbiddenRule::ToStringReturnType,
    ForbiddenRule::MissingReturnValue,
    ForbiddenRule::VoidReturnValue,
    ForbiddenRule::NeverReturn,
    ForbiddenRule::ReadonlyPropertyDefault,
];

fn is_analyzer_covered_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    err.forbidden_rule()
        .is_some_and(|rule| ANALYZER_COVERED_RULES.contains(&rule))
}

/// Parser diagnostics that are dropped from the issue stream and never block
/// analysis: the ones the analyzer reports itself.
fn is_redundant_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    is_analyzer_covered_error(err)
}

/// Whether `errors` leave a recovery guess in the AST, which blocks body
/// analysis. `Forbidden` and `VersionTooLow` are raised on a fully built node.
pub(crate) fn blocks_body_analysis(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    errors.iter().any(|err| err.leaves_ast_incomplete())
}

/// Whether `errors` keep the definition slice out of the parse and stub caches.
/// The parser's diagnostics are cached with the slice, so only an incomplete
/// AST blocks: `Forbidden` and `VersionTooLow` files (bundled stubs declaring
/// per-version variants, polyfills behind `PHP_VERSION_ID` checks) are cached.
pub(crate) fn parse_errors_block_cache(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    blocks_body_analysis(errors)
}

// ---------------------------------------------------------------------------
// Source location helpers
// ---------------------------------------------------------------------------

/// Extract the exact source text covered by a span.
pub(crate) fn span_text(src: &str, span: Span) -> Option<String> {
    if span.start >= span.end {
        return None;
    }
    let s = span.start as usize;
    let e = (span.end as usize).min(src.len());
    src.get(s..e)
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Offset of the first token after the `#[...]` groups (and comments) that open a declaration span.
pub(crate) fn skip_leading_attributes(src: &str, offset: u32) -> u32 {
    let bytes = src.as_bytes();
    let mut i = offset as usize;
    if !src.get(i..).is_some_and(|s| s.starts_with("#[")) {
        return offset;
    }
    loop {
        while i < bytes.len() {
            match bytes[i] {
                b if b.is_ascii_whitespace() => i += 1,
                b'/' if bytes.get(i + 1) == Some(&b'*') => {
                    i = src[i + 2..]
                        .find("*/")
                        .map_or(bytes.len(), |p| i + 2 + p + 2);
                }
                b'#' if bytes.get(i + 1) != Some(&b'[') => i = skip_line(bytes, i),
                b'/' if bytes.get(i + 1) == Some(&b'/') => i = skip_line(bytes, i),
                _ => break,
            }
        }
        if !src[i..].starts_with("#[") {
            return i as u32;
        }
        i += 2;
        let mut depth = 1u32;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                quote @ (b'\'' | b'"') => {
                    i += 1;
                    while i < bytes.len() && bytes[i] != quote {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                }
                _ => {}
            }
            i += 1;
        }
        i = i.min(bytes.len());
    }
}

fn skip_line(bytes: &[u8], i: usize) -> usize {
    bytes[i..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |p| i + p + 1)
}

// ---------------------------------------------------------------------------
// Docblock extraction from source text
// ---------------------------------------------------------------------------

/// Scan backwards from `offset` and return the `/** ... */` docblock comment
/// that immediately precedes the token at that position, if any.
///
/// Whitespace and class-level modifier keywords (`final`, `abstract`,
/// `readonly`) between the docblock and the declaration are skipped — the
/// php-rs-parser places `span.start` at the `class`/`interface`/`trait`
/// keyword, after any modifiers.
///
/// A `?>`/inline-HTML/`<?php` block boundary is also skipped: a Yii2-style
/// view template routinely writes `/** @var Post $model */` immediately
/// before closing the PHP block, then reopens it later to actually use the
/// variable (php-lsp#235). The docblock is textually adjacent to the
/// declaration in every sense that matters here — nothing but tag
/// punctuation and inert markup sits between them.
pub(crate) fn find_preceding_docblock(source: &str, offset: u32) -> Option<&str> {
    let offset = (offset as usize).min(source.len());
    if offset == 0 {
        return None;
    }
    let mut trimmed = source[..offset].trim_end();
    loop {
        let after_ws = trimmed.trim_end();

        if let Some(before_tag) = strip_trailing_reopen_tag(after_ws) {
            let before_html = before_tag.trim_end();
            trimmed = match before_html.rfind("?>") {
                Some(close_idx) => &before_html[..close_idx],
                // A re-opening tag with nothing before it to close — this is
                // the file's very first `<?php`, not a split block boundary.
                None => before_html,
            };
            continue;
        }

        // Strip trailing modifier keywords like `final` or `abstract readonly`.
        let last_word_start = after_ws
            .char_indices()
            .rfind(|(_, c)| !c.is_ascii_alphabetic())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        let word = &after_ws[last_word_start..];
        if matches!(word, "final" | "abstract" | "readonly") {
            trimmed = &after_ws[..last_word_start];
        } else {
            trimmed = after_ws;
            break;
        }
    }
    if !trimmed.ends_with("*/") {
        return None;
    }
    let end = trimmed.rfind("*/")?;
    // Prefer /** docblocks; fall back to /* for inline @var annotations (e.g. Yii2 view files).
    let start = trimmed[..end]
        .rfind("/**")
        .or_else(|| trimmed[..end].rfind("/*"))?;
    Some(&trimmed[start..end + 2])
}

/// A closure's docblock: PHP takes the last one before `function` / `fn`, which may follow its attributes.
pub(crate) fn closure_docblock<'s>(
    source: &'s str,
    span_start: u32,
    attributes: &[php_ast::owned::Attribute],
) -> Option<&'s str> {
    attributes
        .last()
        .and_then(|attr| {
            // The attribute span may stop before the group's `,` / `]`.
            let rest = source.get(attr.span.end as usize..)?.trim_start();
            let rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
            let rest = rest.strip_prefix(']').unwrap_or(rest).trim_start();
            if !rest.starts_with("/*") {
                return None;
            }
            Some(&rest[..rest.find("*/")? + 2])
        })
        .or_else(|| find_preceding_docblock(source, span_start))
}

/// Strip a trailing PHP re-opening tag (`<?php`, `<?=`, or short-open `<?`),
/// returning the text before it. Used by [`find_preceding_docblock`] to look
/// past a `?> ... <?php` gap for the docblock that precedes it.
fn strip_trailing_reopen_tag(s: &str) -> Option<&str> {
    for tag in ["<?php", "<?=", "<?"] {
        if let Some(stripped) = s.strip_suffix(tag) {
            return Some(stripped);
        }
    }
    None
}

/// Gives each class member and parameter the docblock PHP reads for it. The parser looks for it
/// only before the first `#[`, so one written after the attributes went to the next member.
pub(crate) fn attach_docblocks_after_attributes(parsed: &mut php_rs_parser::ParseResult) {
    use php_ast::owned::fold::{
        fold_owned_class_member, fold_owned_enum_member, fold_owned_param, OwnedFold,
    };
    use php_ast::owned::{
        Attribute, ClassMember, ClassMemberKind, Comment, EnumMember, EnumMemberKind, Param,
        Program,
    };

    struct Attach<'s> {
        source: &'s str,
        prev_end: u32,
    }

    impl Attach<'_> {
        fn fix(&self, floor: u32, doc: &mut Option<Comment>, attrs: &[Attribute], name_start: u32) {
            if doc.as_ref().is_some_and(|d| d.span.start < floor) {
                *doc = None;
            }
            let Some(last_attr) = attrs.last() else {
                return;
            };
            let from = last_attr.span.end as usize;
            let Some(between) = self.source.get(from..name_start as usize) else {
                return;
            };
            let Some(open) = between.rfind("/**") else {
                return;
            };
            let Some(len) = between[open..].find("*/") else {
                return;
            };
            let start = (from + open) as u32;
            let text = &between[open..open + len + 2];
            *doc = Some(Comment {
                kind: php_ast::ast::CommentKind::Doc,
                text: text.into(),
                span: Span::new(start, start + text.len() as u32),
            });
        }
    }

    impl OwnedFold for Attach<'_> {
        fn fold_class_member(&mut self, member: ClassMember) -> ClassMember {
            let floor = self.prev_end;
            let mut member = fold_owned_class_member(self, member);
            match &mut member.kind {
                ClassMemberKind::Property(p) => {
                    self.fix(floor, &mut p.doc_comment, &p.attributes, p.name.span.start)
                }
                ClassMemberKind::Method(m) => {
                    self.fix(floor, &mut m.doc_comment, &m.attributes, m.name.span.start)
                }
                ClassMemberKind::ClassConst(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                _ => {}
            }
            self.prev_end = member.span.end;
            member
        }

        fn fold_enum_member(&mut self, member: EnumMember) -> EnumMember {
            let floor = self.prev_end;
            let mut member = fold_owned_enum_member(self, member);
            match &mut member.kind {
                EnumMemberKind::Case(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                EnumMemberKind::Property(p) => {
                    self.fix(floor, &mut p.doc_comment, &p.attributes, p.name.span.start)
                }
                EnumMemberKind::Method(m) => {
                    self.fix(floor, &mut m.doc_comment, &m.attributes, m.name.span.start)
                }
                EnumMemberKind::ClassConst(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                _ => {}
            }
            self.prev_end = member.span.end;
            member
        }

        fn fold_param(&mut self, param: Param) -> Param {
            let floor = self.prev_end;
            let mut param = fold_owned_param(self, param);
            self.fix(
                floor,
                &mut param.doc_comment,
                &param.attributes,
                param.name.span.start,
            );
            self.prev_end = param.span.end;
            param
        }
    }

    let source = parsed.source.as_str();
    let docblock_after_attribute = source
        .match_indices("/**")
        .any(|(i, _)| source[..i].trim_end().ends_with(']'));
    if !docblock_after_attribute {
        return;
    }
    let empty = Program {
        stmts: Box::new([]),
        span: parsed.program.span,
    };
    let program = std::mem::replace(&mut parsed.program, empty);
    parsed.program = Attach {
        source: &parsed.source,
        prev_end: 0,
    }
    .fold_program(program);
}

// ---------------------------------------------------------------------------
// Name resolution helper — join Name parts to a string
// ---------------------------------------------------------------------------

/// A relative name keeps its `namespace\` prefix so resolution skips `use` imports.
pub(crate) fn name_to_string<'src>(name: &php_ast::ast::Name<'_, 'src>) -> Cow<'src, str> {
    if name.kind() == php_ast::ast::NameKind::Relative {
        return Cow::Owned(format!("namespace\\{}", name.join_parts()));
    }
    name.to_string_repr()
}

/// Same as [`name_to_string`] but for the owned (lifetime-free) AST.
pub(crate) fn name_to_string_owned(name: &php_ast::owned::Name) -> Cow<'_, str> {
    let fully_qualified = name.kind == php_ast::ast::NameKind::FullyQualified;
    let relative = name.kind == php_ast::ast::NameKind::Relative;
    if let [single] = &*name.parts {
        if !fully_qualified && !relative {
            return Cow::Borrowed(single);
        }
    }
    let len = name.parts.iter().map(|p| p.len() + 1).sum::<usize>();
    let mut out = String::with_capacity(len + 9);
    if relative {
        out.push_str("namespace");
    }
    for (i, part) in name.parts.iter().enumerate() {
        if fully_qualified || relative || i > 0 {
            out.push('\\');
        }
        out.push_str(part);
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod ast_incomplete_tests {
    use super::{blocks_body_analysis, parse_errors_block_cache};

    fn blocks(src: &str) -> bool {
        let parsed = php_rs_parser::parse(src);
        assert!(!parsed.errors.is_empty());
        parse_errors_block_cache(&parsed.errors)
    }

    #[test]
    fn redeclared_method_leaves_ast_complete() {
        assert!(!blocks(
            "<?php\nclass A { function f() {} function f() {} }\n"
        ));
    }

    #[test]
    fn reserved_function_name_leaves_ast_complete() {
        assert!(!blocks("<?php\nfunction exit() {}\n"));
    }

    #[test]
    fn version_too_low_leaves_ast_complete() {
        let mut options = php_rs_parser::ParseOptions::default();
        options.version = php_ast::PhpVersion::Php74;
        let parsed = php_rs_parser::parse_with("<?php\n$x = match (1) { 1 => 2 };\n", &options);
        assert!(!parsed.errors.is_empty());
        assert!(!blocks_body_analysis(&parsed.errors));
        assert!(!parse_errors_block_cache(&parsed.errors));
    }

    #[test]
    fn syntax_error_leaves_ast_incomplete() {
        assert!(blocks("<?php\nfunction f( {}\n"));
    }

    /// The depth diagnostic must block caching on its own, so a slice built
    /// from a truncated AST is never cached.
    #[test]
    fn expression_depth_limit_leaves_ast_incomplete() {
        let src = format!("<?php\n$x = {}1{};\n", "(".repeat(1000), ")".repeat(1000));
        let parsed = php_rs_parser::parse(&src);
        let depth_err = parsed
            .errors
            .iter()
            .find(|e| {
                matches!(
                    e,
                    php_rs_parser::diagnostics::ParseError::NestingTooDeep { .. }
                )
            })
            .expect("parser no longer emits the depth-limit diagnostic");
        assert!(depth_err.leaves_ast_incomplete());
    }
}

#[cfg(test)]
mod analyzer_covered_tests {
    use super::{is_redundant_parse_error, ANALYZER_COVERED_RULES};

    /// One source per rule in `ANALYZER_COVERED_RULES`, in the same order.
    const SOURCES: &[&str] = &[
        "<?php\nclass A { function __toString(): int { return 1; } }\n",
        "<?php\nfunction f(): int { return; }\n",
        "<?php\nfunction f(): void { return 1; }\n",
        "<?php\nfunction f(): never { return; }\n",
        "<?php\nclass A { public readonly int $x = 1; }\n",
    ];

    #[test]
    fn every_covered_rule_matches_its_parser_diagnostic() {
        assert_eq!(SOURCES.len(), ANALYZER_COVERED_RULES.len());
        for (src, rule) in SOURCES.iter().zip(ANALYZER_COVERED_RULES) {
            let parsed = php_rs_parser::parse(src);
            let rules: Vec<_> = parsed.errors.iter().map(|e| e.forbidden_rule()).collect();
            assert_eq!(rules, [Some(*rule)], "{src:?}: {:?}", parsed.errors);
            assert!(parsed.errors.iter().all(is_redundant_parse_error));
        }
    }

    #[test]
    fn neighbouring_rules_are_not_covered() {
        for src in [
            "<?php\nclass A { function __serialize(): int { return 1; } }\n",
            "<?php\nfunction f($_GET) {}\n",
            "<?php\nfunction f() { unset($this); }\n",
            "<?php\nclass A { function f() { parent::f(); } }\n",
            "<?php\nfunction f() { self::g(); }\n",
            "<?php\nfunction f(): int { yield 1; }\n",
            "<?php\nfunction f($a, $a) {}\n",
            "<?php\n#[Attribute] trait T {}\n",
            "<?php\n#[Deprecated] class A {}\n",
            "<?php\n#[Override] class A {}\n",
            "<?php\nfunction f() { $this = 1; }\n",
        ] {
            let parsed = php_rs_parser::parse(src);
            assert!(
                !parsed.errors.is_empty() && !parsed.errors.iter().any(is_redundant_parse_error),
                "{src:?}: {:?}",
                parsed.errors
            );
        }
    }
}

#[cfg(test)]
mod docblock_lookup_tests {
    use super::find_preceding_docblock;

    #[test]
    fn plain_docblock_immediately_before_offset() {
        let src = "<?php\n/** @var Post $model */\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    #[test]
    fn docblock_survives_a_split_php_html_block() {
        let src = "<?php\nclass Post { public string $title = ''; }\n\
                   /** @var Post $model */\n?>\n<div>\n<?php if ($model->title !== ''): ?>\n";
        let offset = src.rfind("if (").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    #[test]
    fn docblock_survives_a_split_block_via_short_echo_reopen() {
        let src = "<?php\n/** @var Post $model */\n?>\n<div>\n<?= $model->title ?>\n";
        let offset = src.rfind("$model->title").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    /// A re-opening `<?php` with no preceding `?>` is the file's very first
    /// tag, not a split block — there's nothing to skip past, and (since
    /// nothing precedes it) no docblock to find either.
    #[test]
    fn leading_reopen_tag_with_no_prior_close_finds_nothing() {
        let src = "<?php\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    /// Guards against treating unrelated HTML/PHP alternation as a docblock
    /// boundary when there genuinely is no docblock to find.
    #[test]
    fn split_block_with_no_docblock_finds_nothing() {
        let src = "<?php\n$unrelated = 1;\n?>\n<div>\n<?php $x = 1;\n";
        let offset = src.rfind("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    #[test]
    fn modifier_keywords_still_skip_after_a_split_block() {
        // Not a realistic PHP construct (a docblock + modifier can't
        // actually precede a re-opening tag), but exercises that the two
        // skip loops compose without infinite-looping or misordering.
        let src = "<?php\n/** doc */\nfinal\n?>\n<?php\n";
        let offset = src.len() as u32;
        assert_eq!(find_preceding_docblock(src, offset), Some("/** doc */"));
    }
}

#[cfg(test)]
mod skip_leading_attributes_tests {
    use super::skip_leading_attributes;

    fn after(src: &str) -> &str {
        &src[skip_leading_attributes(src, 0) as usize..]
    }

    #[test]
    fn skips_groups_comments_and_brackets_in_strings() {
        assert_eq!(after("#[A] #[B(['x' => \"]\"])]\nclass C {}"), "class C {}");
        assert_eq!(
            after("#[A, B('\\']')] /** doc */ // c\n# c\nfinal class C {}"),
            "final class C {}"
        );
        assert_eq!(after("public $x;"), "public $x;");
    }

    #[test]
    fn stops_at_end_of_unterminated_attribute() {
        assert_eq!(after("#[A('x"), "");
    }
}

#[cfg(test)]
mod attach_docblocks_tests {
    use php_ast::owned::{ClassMemberKind, EnumMemberKind, StmtKind};

    #[test]
    fn member_and_param_docblocks_after_attributes() {
        let src = "<?php\nclass C {\n    #[A]\n    /** p */\n    public $p;\n    public $plain;\n    #[A]\n    /** c */\n    const X = 1;\n    #[A] /** m */\n    public function m(#[A] /** q */ $q) {}\n}\nenum E {\n    #[A]\n    /** e */\n    case K;\n}\n";
        let mut parsed = php_rs_parser::parse(src);
        super::attach_docblocks_after_attributes(&mut parsed);
        let mut docs = Vec::new();
        for stmt in parsed.program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Class(c) => {
                    for m in c.body.members.iter() {
                        match &m.kind {
                            ClassMemberKind::Property(p) => docs.push(p.doc_comment.clone()),
                            ClassMemberKind::ClassConst(k) => docs.push(k.doc_comment.clone()),
                            ClassMemberKind::Method(f) => {
                                docs.push(f.doc_comment.clone());
                                docs.push(f.params[0].doc_comment.clone());
                            }
                            _ => {}
                        }
                    }
                }
                StmtKind::Enum(e) => {
                    for m in e.body.members.iter() {
                        if let EnumMemberKind::Case(k) = &m.kind {
                            docs.push(k.doc_comment.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let texts: Vec<_> = docs
            .iter()
            .map(|d| d.as_ref().map(|c| c.text.to_string()))
            .collect();
        let expected = [
            Some("/** p */"),
            None,
            Some("/** c */"),
            Some("/** m */"),
            Some("/** q */"),
            Some("/** e */"),
        ];
        assert_eq!(texts, expected.map(|t| t.map(str::to_string)));
    }
}
