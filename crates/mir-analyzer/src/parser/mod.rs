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
    let abstract_methods = abstract_methods_by_class(parsed, version);
    let shadowed_set_hook = shadowed_set_hook_errors(parsed);
    parsed
        .errors
        .iter()
        .filter(|err| {
            !is_redundant_parse_error(err)
                && raised_at(err, source, &parsed.program, version)
                && !shadowed_by_final_abstract(err, &parsed.errors)
                && !shadowed_set_hook.contains(&(err.span().start, err.forbidden_rule()))
        })
        .filter_map(|err| {
            let mut issue = parse_error_to_issue(err, file, source, parsed, version);
            if let Some(class) = abstract_methods.iter().find(|c| {
                abstract_group(err.forbidden_rule()) == Some(c.is_enum)
                    && c.offsets.contains(&err.span().start)
            }) {
                if class.offsets[0] != err.span().start {
                    return None;
                }
                if let mir_issues::IssueKind::CompileError { message, .. } = &mut issue.kind {
                    *message = class.message(version);
                }
            }
            Some(issue)
        })
        .collect()
}

/// A class's or enum's abstract members that PHP reports together, before 8.5.
struct AbstractMethods {
    is_enum: bool,
    class: String,
    /// Methods, then property hooks such as `$p::get`.
    methods: Vec<String>,
    /// Offsets of the parser's per-member diagnostics.
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
            return format!(
                "Enum {} must implement {count} abstract private method{plural} ({list})",
                self.class
            );
        }
        format!(
            "Class {} contains {count} abstract method{plural} and must therefore be declared abstract or implement the remaining methods ({list})",
            self.class
        )
    }
}

/// Whether the parser's diagnostic is one PHP groups per enum (`Some(true)`) or class before 8.5.
fn abstract_group(rule: Option<ForbiddenRule>) -> Option<bool> {
    match rule? {
        ForbiddenRule::AbstractEnumMethod => Some(true),
        ForbiddenRule::AbstractAnonymousClassMethod
        | ForbiddenRule::AbstractMemberInConcreteClass => Some(false),
        _ => None,
    }
}

/// Before 8.5 PHP reports a concrete class's or enum's abstract members in one error.
fn abstract_methods_by_class(
    parsed: &php_rs_parser::ParseResult,
    version: PhpVersion,
) -> Vec<AbstractMethods> {
    let mut classes: Vec<(u32, AbstractMethods)> = Vec::new();
    if version >= PhpVersion::new(8, 5) {
        return Vec::new();
    }
    for err in &parsed.errors {
        let Some(is_enum) = abstract_group(err.forbidden_rule()) else {
            continue;
        };
        if shadowed_by_final_abstract(err, &parsed.errors) {
            continue;
        }
        let start = err.span().start;
        let Some((class, method, scope)) =
            find_decl_in_scope(&parsed.program, start, DeclTarget::Method, version).or_else(|| {
                find_decl_in_scope(&parsed.program, start, DeclTarget::Property, version)
            })
        else {
            continue;
        };
        match classes
            .iter_mut()
            .find(|(s, group)| *s == scope && group.is_enum == is_enum)
        {
            Some((_, group)) => {
                group.methods.push(method);
                group.offsets.push(start);
            }
            None => classes.push((
                scope,
                AbstractMethods {
                    is_enum,
                    class,
                    methods: vec![method],
                    offsets: vec![start],
                },
            )),
        }
    }
    for (scope, group) in &mut classes {
        if !group.is_enum {
            // The parser names only a concrete class's first abstract method.
            group.methods = abstract_member_names(&parsed.program, *scope);
        }
    }
    classes.into_iter().map(|(_, group)| group).collect()
}

/// The abstract methods, then the abstract property hooks, of the class starting at `start`.
fn abstract_member_names(program: &php_ast::owned::Program, start: u32) -> Vec<String> {
    use php_ast::owned::visitor::OwnedVisitor;
    struct ClassAt {
        start: u32,
        names: Vec<String>,
    }
    impl ClassAt {
        fn collect(&mut self, members: &[php_ast::owned::ClassMember]) {
            use php_ast::owned::{ClassMemberKind, PropertyHookBody};
            let mut hooks = Vec::new();
            for member in members {
                match &member.kind {
                    ClassMemberKind::Method(m) if m.modifiers.is_abstract => {
                        self.names
                            .push(m.name.as_str().unwrap_or_default().to_string());
                    }
                    ClassMemberKind::Property(p) if p.modifiers.is_abstract => {
                        for hook in p.hooks.iter() {
                            if matches!(hook.body, PropertyHookBody::Abstract) {
                                let kind = match hook.kind {
                                    php_ast::ast::PropertyHookKind::Get => "get",
                                    _ => "set",
                                };
                                let name = p.name.as_str().unwrap_or_default();
                                hooks.push(format!("${name}::{kind}"));
                            }
                        }
                    }
                    _ => {}
                }
            }
            self.names.extend(hooks);
        }
    }
    impl OwnedVisitor for ClassAt {
        type Break = ();
        fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
            if let php_ast::owned::StmtKind::Class(c) = &stmt.kind {
                if stmt.span.start == self.start {
                    self.collect(&c.body.members);
                    return std::ops::ControlFlow::Break(());
                }
            }
            crate::recursion::walk_stmt(self, stmt)
        }
        fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
            if let php_ast::owned::ExprKind::AnonymousClass(c) = &expr.kind {
                if expr.span.start == self.start {
                    self.collect(&c.body.members);
                    return std::ops::ControlFlow::Break(());
                }
            }
            crate::recursion::walk_expr(self, expr)
        }
    }
    let mut finder = ClassAt {
        start,
        names: Vec::new(),
    };
    for stmt in &program.stmts {
        if finder.visit_stmt(stmt).is_break() {
            break;
        }
    }
    finder.names
}

/// An enum's abstract-method error on a `final abstract` method, where PHP stops at the modifier error.
fn shadowed_by_final_abstract(
    err: &php_rs_parser::diagnostics::ParseError,
    errors: &[php_rs_parser::diagnostics::ParseError],
) -> bool {
    err.forbidden_rule() == Some(ForbiddenRule::AbstractEnumMethod)
        && errors.iter().any(|other| {
            other.forbidden_rule() == Some(ForbiddenRule::AbstractFinalConflict)
                && other.span().start == err.span().start
        })
}

/// PHP's order among a set hook's parameter errors; it reports only the first.
fn set_hook_error_rank(rule: Option<ForbiddenRule>) -> Option<u8> {
    match rule? {
        ForbiddenRule::SetHookParameterCount => Some(0),
        ForbiddenRule::SetHookParameterByReference => Some(1),
        ForbiddenRule::SetHookParameterVariadic => Some(2),
        ForbiddenRule::SetHookParameterDefault => Some(3),
        ForbiddenRule::SetHookParameterType => Some(4),
        _ => None,
    }
}

/// The set-hook parameter errors after the first one PHP reports for the same hook.
fn shadowed_set_hook_errors(
    parsed: &php_rs_parser::ParseResult,
) -> Vec<(u32, Option<ForbiddenRule>)> {
    let errors: Vec<_> = parsed
        .errors
        .iter()
        .filter_map(|err| Some((set_hook_error_rank(err.forbidden_rule())?, err)))
        .collect();
    if errors.len() < 2 {
        return Vec::new();
    }
    let mut hooks = SetHookSpans(Vec::new());
    for stmt in &parsed.program.stmts {
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut hooks, stmt);
    }
    // A nested hook comes after the hook containing it, so the last match is the innermost.
    let hook_of = |err: &php_rs_parser::diagnostics::ParseError| {
        let offset = err.span().start;
        hooks
            .0
            .iter()
            .rposition(|s| s.start <= offset && offset < s.end)
    };
    // Per hook: (hook, rank, index into `errors`) of the error PHP reports.
    let mut reported: Vec<(usize, u8, usize)> = Vec::new();
    for (i, (rank, err)) in errors.iter().enumerate() {
        let Some(hook) = hook_of(err) else {
            continue;
        };
        match reported.iter_mut().find(|(h, _, _)| *h == hook) {
            Some(first) if *rank < first.1 => *first = (hook, *rank, i),
            Some(_) => {}
            None => reported.push((hook, *rank, i)),
        }
    }
    errors
        .iter()
        .enumerate()
        .filter(|(i, (_, err))| {
            hook_of(err).is_some() && !reported.iter().any(|(_, _, kept)| kept == i)
        })
        .map(|(_, (_, err))| (err.span().start, err.forbidden_rule()))
        .collect()
}

/// Spans of `set` hooks, in source order.
struct SetHookSpans(Vec<Span>);

impl php_ast::owned::visitor::OwnedVisitor for SetHookSpans {
    type Break = ();

    fn visit_property_hook(
        &mut self,
        hook: &php_ast::owned::PropertyHook,
    ) -> std::ops::ControlFlow<Self::Break> {
        if hook.kind == php_ast::ast::PropertyHookKind::Set {
            self.0.push(hook.span);
        }
        php_ast::owned::visitor::walk_owned_property_hook(self, hook)
    }
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
    let message = with_anonymous_class_name(message, err.span().start, &parsed.program, version);
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

/// `message` with the parser's `class@anonymous` named as PHP does from 8.0, after the class's parent or first interface.
fn with_anonymous_class_name(
    message: String,
    offset: u32,
    program: &php_ast::owned::Program,
    version: PhpVersion,
) -> String {
    if version < PhpVersion::new(8, 0) || !message.contains("class@anonymous") {
        return message;
    }
    let class = [DeclTarget::Property, DeclTarget::Method]
        .into_iter()
        .find_map(|target| find_decl(program, offset, target, version))
        .map(|(class, _)| class);
    match class {
        Some(class) if class.ends_with("@anonymous") => message.replace("class@anonymous", &class),
        _ => message,
    }
}

/// Whether PHP rejects the parser's diagnostic at compile time (`php -l` prints `Fatal error`).
fn is_compile_error(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &str,
    program: &php_ast::owned::Program,
    version: PhpVersion,
) -> bool {
    match err.forbidden_rule() {
        Some(ForbiddenRule::MultipleVisibilityModifiers | ForbiddenRule::DuplicateModifier)
            if version < PhpVersion::new(8, 1) =>
        {
            promoted_param_modifier(err, source, program).is_none()
        }
        Some(
            ForbiddenRule::MultipleVisibilityModifiers
            | ForbiddenRule::MultipleSetVisibilityModifiers
            | ForbiddenRule::DuplicateModifier
            | ForbiddenRule::AbstractFinalConflict
            | ForbiddenRule::InvalidConstantModifier
            | ForbiddenRule::OutermostScopeOnly
            | ForbiddenRule::PartialApplicationInNew,
        ) => true,
        // A parameter modifier is a syntax error before 8.3, `readonly` before 8.1.
        Some(ForbiddenRule::ModifierNotAllowed) => {
            version >= PhpVersion::new(8, 3)
                || version >= PhpVersion::new(8, 1) && {
                    let message = err.to_string();
                    message.ends_with(" on a method") || message.ends_with(" as method modifier")
                }
        }
        Some(ForbiddenRule::InvalidCurlyInterpolation) => false,
        Some(ForbiddenRule::CurlyOffsetAccess) => version < PhpVersion::new(8, 4),
        Some(ForbiddenRule::ReservedClassName | ForbiddenRule::KeywordAsDeclarationName) => {
            reserved_declaration_name(err, source).is_some()
        }
        // `$x = &1` is a syntax error; `[&$a] = 1` a compile error.
        Some(ForbiddenRule::ReferenceToNonReferenceable) => !preceded_by(source, err.span(), '&'),
        _ => err.phase() == Phase::Compile,
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
            (message.ends_with("cannot declare a return type")
                || message.to_ascii_lowercase().contains("::__clone(): "))
                && !declared_in_interface(source, program, err.span().start)
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
            find_decl(program, err.span().start, DeclTarget::Method, version).is_none()
        }
        _ => true,
    }
}

/// Whether the method declared at `offset` belongs to an interface, which PHP 7.4 does not check.
fn declared_in_interface(source: &str, program: &php_ast::owned::Program, offset: u32) -> bool {
    find_decl_in_scope(program, offset, DeclTarget::Method, PhpVersion::LATEST).is_some_and(
        |(_, _, scope)| {
            let start = skip_leading_attributes(source, scope) as usize;
            source.get(start..).is_some_and(|rest| {
                rest.get(..9)
                    .is_some_and(|word| word.eq_ignore_ascii_case("interface"))
            })
        },
    )
}

/// The repeated modifier of a promoted parameter the parser rejects, which
/// PHP 8.0's grammar reads as a syntax error (8.1 compiles modifiers as a list).
fn promoted_param_modifier<'a>(
    err: &php_rs_parser::diagnostics::ParseError,
    source: &'a str,
    program: &php_ast::owned::Program,
) -> Option<&'a str> {
    let span = err.span();
    find_decl(program, span.start, DeclTarget::Param, PhpVersion::LATEST)?;
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
    match err.forbidden_rule()? {
        ForbiddenRule::InvalidInterfaceMethodModifier if before(8, 1) => {
            let (interface, method) =
                find_decl(program, err.span().start, DeclTarget::Method, version)?;
            Some(format!(
                "Access type for interface method {interface}::{method}() must be omitted"
            ))
        }
        // The parser words a trait alias's `readonly` in 8.3's text on every version.
        ForbiddenRule::ModifierNotAllowed if before(8, 3) => {
            let message = parser_message();
            let modifier = message
                .strip_prefix("Cannot use the ")?
                .strip_suffix(" modifier on a method")?;
            Some(format!("Cannot use '{modifier}' as method modifier"))
        }
        ForbiddenRule::InvalidTraitAliasModifier if before(8, 3) => {
            let message = parser_message();
            let modifier = message.split('"').nth(1)?;
            Some(format!("Cannot use '{modifier}' as method modifier"))
        }
        ForbiddenRule::AbstractPropertyWithoutAbstractHook if before(8, 3) => {
            Some("Properties cannot be declared abstract".to_string())
        }
        ForbiddenRule::AbstractFinalConflict if before(8, 3) => {
            let message = parser_message();
            (message != "Cannot use the final modifier on an abstract class")
                .then(|| "Cannot use the final modifier on an abstract class member".to_string())
        }
        ForbiddenRule::InvalidConstantModifier if before(8, 3) => {
            let message = parser_message();
            let modifier = message
                .strip_prefix("Cannot use the ")?
                .strip_suffix(" modifier on a class constant")?;
            Some(format!("Cannot use '{modifier}' as constant modifier"))
        }
        ForbiddenRule::EnumProperty if before(8, 2) => {
            Some("Enums may not include properties".to_string())
        }
        ForbiddenRule::VirtualPropertyAsymmetricVisibility if before(8, 6) => {
            let message = parser_message();
            Some(if let Some(rest) = message.strip_prefix("set-only ") {
                format!("Write-only {rest}")
            } else {
                format!("Read-only {}", message.strip_prefix("get-only ")?)
            })
        }
        ForbiddenRule::InterfacePropertyWithoutHooks if before(8, 4) => Some(if before(8, 1) {
            "Interfaces may not include member variables".to_string()
        } else {
            "Interfaces may not include properties".to_string()
        }),
        ForbiddenRule::ReservedClassName
        | ForbiddenRule::KeywordAsDeclarationName
        | ForbiddenRule::ReservedEnumName => {
            let (name, kind) = reserved_declaration_name(err, source)?;
            Some(if before(8, 4) {
                format!("Cannot use '{name}' as class name as it is reserved")
            } else {
                let article = if kind.starts_with(['a', 'e', 'i']) {
                    "an"
                } else {
                    "a"
                };
                format!("Cannot use \"{name}\" as {article} {kind} name as it is reserved")
            })
        }
        ForbiddenRule::ReservedClassReference if before(8, 4) => {
            let message = parser_message().replace('"', "'");
            Some(if before(8, 0) {
                message.replace(", as it", " as it")
            } else {
                message
            })
        }
        ForbiddenRule::ReservedTypeName if before(8, 4) => {
            let message = parser_message();
            let name = message.split('"').nth(1)?;
            Some(format!(
                "Cannot use '{name}' as class name as it is reserved"
            ))
        }
        ForbiddenRule::MultipleVisibilityModifiers | ForbiddenRule::DuplicateModifier
            if before(8, 1) =>
        {
            let token = promoted_param_modifier(err, source, program)?;
            Some(format!(
                "syntax error, unexpected token \"{}\", expecting variable",
                token.to_ascii_lowercase()
            ))
        }
        ForbiddenRule::ReferenceToNonReferenceable if before(8, 1) => {
            Some("Cannot assign reference to non referencable value".to_string())
        }
        ForbiddenRule::GeneratorReturnType if before(8, 0) => {
            let message = parser_message();
            // PHP 7.4 names a nullable type without its `?`.
            let ty = message
                .rsplit(", ")
                .next()?
                .strip_suffix(" given")?
                .trim_start_matches('?');
            Some(format!(
                "Generators may only declare a return type of Generator, Iterator, Traversable, or iterable, {ty} is not permitted"
            ))
        }
        ForbiddenRule::AbstractMemberInConcreteClass if before(8, 5) => {
            Some(parser_message().replace("the remaining method (", "the remaining methods ("))
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
            Some(if message.starts_with("null ") {
                "Null can not be used as a standalone type".to_string()
            } else {
                message
            })
        }
        ForbiddenRule::StandaloneTypeInComposite if before(8, 0) => {
            let message = parser_message();
            if !message.starts_with("Void ") {
                return Some(message);
            }
            let after = source.get(err.span().end as usize..).unwrap_or_default();
            Some(if after.trim_start().starts_with(['$', '&', '.']) {
                "void cannot be used as a parameter type".to_string()
            } else {
                "Void type cannot be nullable".to_string()
            })
        }
        ForbiddenRule::RedundantType if before(8, 4) => Some(parser_message().replace(
            ", bool must be used instead",
            ", bool should be used instead",
        )),
        ForbiddenRule::ClassFetchOnLiteral | ForbiddenRule::ClassFetchOnFoldedValue
            if before(8, 3) =>
        {
            let message = parser_message();
            let value = message.strip_prefix("Cannot use \"::class\" on ")?;
            let value = if matches!(value, "true" | "false") {
                "bool"
            } else {
                value
            };
            Some(format!("Cannot use \"::class\" on value of type {value}"))
        }
        ForbiddenRule::UnpackNonArray if before(8, 4) => {
            Some("Only arrays and Traversables can be unpacked".to_string())
        }
        ForbiddenRule::PositionalAfterNamedArgument if version == PhpVersion::new(8, 0) => {
            (parser_message() == "Cannot use argument unpacking after named arguments")
                .then(|| "Cannot combine named arguments and argument unpacking".to_string())
        }
        ForbiddenRule::DefaultValueType if before(8, 0) => {
            default_value_message_74(&parser_message())
        }
        ForbiddenRule::DefaultValueType if before(8, 3) => {
            let message = parser_message();
            let value = message.strip_prefix("Cannot use ")?.split(' ').next()?;
            matches!(value, "true" | "false").then(|| message.replacen(value, "bool", 1))
        }
        ForbiddenRule::PureEnumCaseValue if version == PhpVersion::new(8, 1) => {
            let (enum_name, case) =
                find_decl(program, err.span().start, DeclTarget::EnumCase, version)?;
            let mut message =
                format!("Case {case} of non-backed enum {enum_name} must not have a value");
            if let Some(ty) = literal_case_value_type(source, err.span()) {
                message.push_str(&format!(", try adding \": {ty}\" to the enum declaration"));
            }
            Some(message)
        }
        ForbiddenRule::RealCast => {
            Some("The (real) cast has been removed, use (float) instead".to_string())
        }
        ForbiddenRule::InvalidCodepointEscape => {
            let message = parser_message();
            message
                .strip_suffix(": empty code point")
                .map(str::to_string)
        }
        ForbiddenRule::HeredocBodyIndentation => {
            let indent = heredoc_closing_indent(source, err.span())?;
            Some(format!(
                "Invalid body indentation level (expecting an indentation level of at least {indent})"
            ))
        }
        ForbiddenRule::UnterminatedComment => {
            let line = source[..source.len().min(err.span().start as usize)]
                .matches('\n')
                .count()
                + 1;
            Some(format!("Unterminated comment starting line {line}"))
        }
        _ => None,
    }
}

/// PHP 7.4's message for a default value its single, possibly nullable, type rejects.
fn default_value_message_74(message: &str) -> Option<String> {
    let (owner, ty) = message
        .split_once(" as default value for ")?
        .1
        .rsplit_once(" of type ")?;
    let ty = ty.strip_prefix('?').unwrap_or(ty);
    let builtin = ty.to_ascii_lowercase();
    // PHP 7.4 has no `mixed`, `null`, `false` or `true` type, so those are class names.
    let is_class = !matches!(
        builtin.as_str(),
        "int" | "float" | "string" | "bool" | "array" | "iterable" | "callable" | "object"
    );
    if owner.starts_with("parameter ") {
        let rule = match builtin.as_str() {
            _ if is_class => "a class type can only be NULL",
            "int" => "a int type can only be int or NULL",
            "float" => "a float type can only be float, integer, or NULL",
            "string" => "a string type can only be string or NULL",
            "bool" => "a bool type can only be bool or NULL",
            "array" => "array type can only be an array or NULL",
            "iterable" => "iterable type can only be an array or NULL",
            "callable" => "callable type can only be NULL",
            "object" => "an object type can only be NULL",
            _ => return None,
        };
        return Some(format!("Default value for parameters with {rule}"));
    }
    if is_class {
        return Some(format!("Property of type {ty} may not have default value"));
    }
    let allowed = match builtin.as_str() {
        "int" => "int",
        "float" => "float or int",
        "string" => "string",
        "bool" => "bool",
        "array" | "iterable" => "an array",
        "object" => "object",
        _ => return None,
    };
    Some(format!(
        "Default value for property of type {ty} can only be {allowed}"
    ))
}

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
    version: PhpVersion,
) -> Option<(String, String)> {
    find_decl_in_scope(program, offset, target, version).map(|(class, name, _)| (class, name))
}

/// [`find_decl`] plus the start offset of the enclosing class-like.
fn find_decl_in_scope(
    program: &php_ast::owned::Program,
    offset: u32,
    target: DeclTarget,
    version: PhpVersion,
) -> Option<(String, String, u32)> {
    let mut finder = DeclAt {
        offset,
        target,
        version,
        scope: NameScope::default(),
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

#[derive(PartialEq)]
enum DeclTarget {
    /// A property or promoted parameter.
    Property,
    /// A method parameter.
    Param,
    Method,
    EnumCase,
}

/// Finds the class-like enclosing the `target` declared at `offset`, and the target's name.
struct DeclAt {
    offset: u32,
    target: DeclTarget,
    version: PhpVersion,
    scope: NameScope,
    /// Enclosing class-likes and their start offsets.
    classes: Vec<(String, u32)>,
    found: Option<(String, String, u32)>,
}

/// The namespace and class imports in effect while walking a file's statements.
#[derive(Default)]
pub(crate) struct NameScope {
    namespace: String,
    /// Class `use` imports, by alias.
    imports: rustc_hash::FxHashMap<String, String>,
}

impl NameScope {
    pub(crate) fn qualify(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{name}", self.namespace)
        }
    }

    pub(crate) fn resolve(&self, name: &php_ast::owned::Name) -> String {
        let namespace = (!self.namespace.is_empty()).then(|| self.namespace.clone());
        crate::collector::resolution::resolve_name(
            &name_to_string_owned(name),
            &namespace,
            &self.imports,
        )
    }

    /// Applies a `namespace` or `use` statement; for a braced namespace, returns the outer
    /// scope to restore once its body is walked.
    pub(crate) fn enter(&mut self, stmt: &php_ast::owned::Stmt) -> Option<NameScope> {
        use php_ast::owned::{NamespaceBody, StmtKind};
        match &stmt.kind {
            StmtKind::Namespace(ns) => {
                let inner = NameScope {
                    namespace: ns
                        .name
                        .as_ref()
                        .map(|n| name_to_string_owned(n).into_owned())
                        .unwrap_or_default(),
                    imports: Default::default(),
                };
                let outer = std::mem::replace(self, inner);
                matches!(ns.body, NamespaceBody::Braced(_)).then_some(outer)
            }
            StmtKind::Use(decl) => {
                for item in decl.uses.iter() {
                    if item.kind.unwrap_or(decl.kind) == php_ast::ast::UseKind::Normal {
                        let full = item.name.join_parts();
                        let alias = item
                            .alias
                            .as_ref()
                            .and_then(|a| a.as_str())
                            .unwrap_or_else(|| full.rsplit('\\').next().unwrap_or(&full));
                        self.imports.insert(alias.to_string(), full.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// PHP's name for an anonymous class: from 8.0 its parent's or first interface's.
    pub(crate) fn anonymous_class_name(
        &self,
        class: &php_ast::owned::ClassDecl,
        version: PhpVersion,
    ) -> String {
        let base = class.extends.as_ref().or(class.implements.first());
        match base {
            Some(base) if version >= PhpVersion::new(8, 0) => {
                format!("{}@anonymous", self.resolve(base))
            }
            _ => "class@anonymous".to_string(),
        }
    }
}

impl DeclAt {
    fn qualify(&self, name: &str) -> String {
        self.scope.qualify(name)
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
}

impl php_ast::owned::visitor::OwnedVisitor for DeclAt {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        use php_ast::owned::{NamespaceBody, StmtKind};
        // An unbraced `namespace X;` ends before the code it applies to.
        let unbraced_namespace = matches!(&stmt.kind, StmtKind::Namespace(ns) if !matches!(ns.body, NamespaceBody::Braced(_)));
        let import = matches!(&stmt.kind, StmtKind::Use(_));
        if stmt.span.end <= self.offset && !unbraced_namespace && !import {
            return std::ops::ControlFlow::Continue(());
        }
        match &stmt.kind {
            StmtKind::Namespace(_) | StmtKind::Use(_) => {
                if let Some(outer) = self.scope.enter(stmt) {
                    let flow = crate::recursion::walk_stmt(self, stmt);
                    self.scope = outer;
                    return flow;
                }
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
                return self.in_class(d.name.as_str().unwrap_or_default(), stmt);
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
            _ => {}
        }
        php_ast::owned::visitor::walk_owned_enum_member(self, member)
    }

    fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
        if let php_ast::owned::ExprKind::AnonymousClass(class) = &expr.kind {
            let name = self.scope.anonymous_class_name(class, self.version);
            self.classes.push((name, expr.span.start));
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
    ForbiddenRule::EnumCaseValueType,
    ForbiddenRule::DuplicateEnumCaseValue,
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
    match name.kind {
        php_ast::ast::NameKind::FullyQualified => Cow::Owned(format!("\\{}", name.as_str())),
        php_ast::ast::NameKind::Relative => Cow::Owned(format!("namespace\\{}", name.as_str())),
        _ => Cow::Borrowed(name.as_str()),
    }
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
        "<?php\nenum E: int { case A = 'a'; }\n",
        "<?php\nenum E: int { case A = 1; case B = 1; }\n",
    ];

    #[test]
    fn every_covered_rule_matches_its_parser_diagnostic() {
        assert_eq!(SOURCES.len(), ANALYZER_COVERED_RULES.len());
        // The enum case rules are PHP 8.1's only.
        let options =
            php_rs_parser::ParseOptions::default().version(php_rs_parser::PhpVersion::Php81);
        for (src, rule) in SOURCES.iter().zip(ANALYZER_COVERED_RULES) {
            let parsed = php_rs_parser::parse_with(src, &options);
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
mod docblocks_after_attributes_tests {
    use php_ast::owned::{ClassMemberKind, EnumMemberKind, StmtKind};

    #[test]
    fn member_and_param_docblocks_after_attributes() {
        let src = "<?php\nclass C {\n    #[A]\n    /** p */\n    public $p;\n    public $plain;\n    #[A]\n    /** c */\n    const X = 1;\n    #[A] /** m */\n    public function m(#[A] /** q */ $q) {}\n}\nenum E {\n    #[A]\n    /** e */\n    case K;\n}\n";
        let parsed = php_rs_parser::parse(src);
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
