//! Declarations and statements PHP deprecates at compile time, reported as `DeprecatedSyntax`.

use std::ops::ControlFlow;
use std::sync::Arc;

use mir_issues::{Issue, IssueKind};
use php_ast::ast::UseKind;
use php_ast::owned::visitor::{walk_owned_class_member, walk_owned_enum_member, OwnedVisitor};
use php_ast::owned::{
    Block, ClassMember, ClassMemberKind, EnumMember, EnumMemberKind, Expr, ExprKind, MethodDecl,
    Stmt, StmtKind, TypeHint, TypeHintKind,
};
use php_ast::Span;

use crate::php_version::PhpVersion;

/// Reports PHP 8.6's compile-time deprecations in every declaration of `stmts`.
pub(crate) fn check_deprecated_syntax(
    stmts: &[Stmt],
    php_version: PhpVersion,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<Issue>,
) {
    if php_version < PhpVersion::new(8, 6) {
        return;
    }
    let mut visitor = DeprecatedSyntaxVisitor {
        file,
        source,
        source_map,
        issues,
        classes: Vec::new(),
        finally_depth: 0,
        magic: None,
        generator_reported: false,
        namespace: None,
    };
    for stmt in stmts {
        let _ = visitor.visit_stmt(stmt);
    }
}

/// Whether `name` is one of the identifiers PHP 8.6 reserves for pattern matching.
fn is_reserved_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("let") || name.eq_ignore_ascii_case("is")
}

fn is_namespace_name(name: &php_ast::owned::Ident) -> bool {
    name.as_str()
        .is_some_and(|n| n.eq_ignore_ascii_case("namespace"))
}

fn last_segment(name: &str) -> &str {
    name.rsplit('\\').next().unwrap_or(name)
}

fn allows_null(hint: &TypeHint) -> bool {
    match &hint.kind {
        TypeHintKind::Nullable(_) => true,
        TypeHintKind::Keyword(php_ast::BuiltinType::Null, _) => true,
        TypeHintKind::Union(types) => types.iter().any(allows_null),
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Magic {
    Constructor,
    Destructor,
}

struct DeprecatedSyntaxVisitor<'a> {
    file: &'a Arc<str>,
    source: &'a str,
    source_map: &'a php_rs_parser::source_map::SourceMap,
    issues: &'a mut Vec<Issue>,
    /// Names of the enclosing class-likes; `None` for an anonymous class.
    classes: Vec<Option<String>>,
    /// `finally` blocks around the current statement within the current function.
    finally_depth: usize,
    /// The constructor or destructor whose body is being walked.
    magic: Option<Magic>,
    /// PHP reports a generator constructor or destructor once.
    generator_reported: bool,
    /// The current namespace; `None` in the global one.
    namespace: Option<String>,
}

impl DeprecatedSyntaxVisitor<'_> {
    fn report(&mut self, span: Span, message: String) {
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(self.source, span.start, self.source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(self.source, span.end, self.source_map);
        let location = mir_issues::Location {
            file: self.file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        };
        self.issues.push(
            Issue::new(IssueKind::DeprecatedSyntax { message }, location)
                .with_snippet(crate::parser::span_text(self.source, span).unwrap_or_default()),
        );
    }

    /// Walks a nested function body, which has its own `finally` and magic-method context.
    fn in_function(
        &mut self,
        magic: Option<Magic>,
        walk: impl FnOnce(&mut Self) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let saved = (self.finally_depth, self.magic, self.generator_reported);
        self.finally_depth = 0;
        self.magic = magic;
        self.generator_reported = false;
        let flow = walk(self);
        (self.finally_depth, self.magic, self.generator_reported) = saved;
        flow
    }

    fn in_class(
        &mut self,
        name: Option<String>,
        walk: impl FnOnce(&mut Self) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        let name = name.map(|n| match &self.namespace {
            Some(ns) => format!("{ns}\\{n}"),
            None => n,
        });
        self.classes.push(name);
        let flow = self.in_function(None, walk);
        self.classes.pop();
        flow
    }

    fn check_class_like_name(&mut self, name: Option<&php_ast::owned::Ident>, kind: &str) {
        let Some(ident) = name else { return };
        let name = ident.as_str().unwrap_or_default();
        if is_reserved_name(name) {
            let lower = name.to_ascii_lowercase();
            self.report(
                ident.span,
                format!("Using \"{lower}\" as {kind} name is deprecated since 8.6"),
            );
        }
    }

    fn check_method(&mut self, method: &MethodDecl) {
        let name = method.name.as_str().unwrap_or_default();
        if !name.eq_ignore_ascii_case("__debugInfo") {
            return;
        }
        let Some(hint) = method.return_type.as_ref().filter(|h| allows_null(h)) else {
            return;
        };
        let class = match self.classes.last() {
            Some(Some(class)) => class.clone(),
            _ => "class@anonymous".to_string(),
        };
        self.report(
            hint.span,
            format!(
                "Returning null from {class}::__debugInfo() is deprecated, make the return type non-nullable and return an empty array instead"
            ),
        );
    }

    fn method_magic(method: &MethodDecl) -> Option<Magic> {
        let name = method.name.as_str().unwrap_or_default();
        if name.eq_ignore_ascii_case("__construct") {
            Some(Magic::Constructor)
        } else if name.eq_ignore_ascii_case("__destruct") {
            Some(Magic::Destructor)
        } else {
            None
        }
    }

    fn visit_method(
        &mut self,
        method: &MethodDecl,
        walk: impl FnOnce(&mut Self) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        self.check_method(method);
        self.in_function(Self::method_magic(method), walk)
    }

    fn walk_finally(&mut self, block: &Block) -> ControlFlow<()> {
        self.finally_depth += 1;
        let mut flow = ControlFlow::Continue(());
        for stmt in block.stmts.iter() {
            flow = self.visit_stmt(stmt);
            if flow.is_break() {
                break;
            }
        }
        self.finally_depth -= 1;
        flow
    }
}

impl OwnedVisitor for DeprecatedSyntaxVisitor<'_> {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Return(value) => {
                if self.finally_depth > 0 {
                    self.report(
                        stmt.span,
                        "Returning from a finally block is deprecated".into(),
                    );
                }
                if value.is_some() {
                    match self.magic {
                        Some(Magic::Constructor) => self.report(
                            stmt.span,
                            "Returning a value from a constructor is deprecated".into(),
                        ),
                        Some(Magic::Destructor) => self.report(
                            stmt.span,
                            "Returning a value from a destructor is deprecated".into(),
                        ),
                        None => {}
                    }
                }
            }
            StmtKind::TryCatch(t) => {
                self.visit_block(&t.body)?;
                for catch in t.catches.iter() {
                    self.visit_block(&catch.body)?;
                }
                if let Some(finally) = &t.finally {
                    self.walk_finally(finally)?;
                }
                return ControlFlow::Continue(());
            }
            StmtKind::Function(f) => {
                let name = f.name.as_str().unwrap_or_default();
                if is_reserved_name(name) || name.eq_ignore_ascii_case("readonly") {
                    self.report(
                        f.name.span,
                        format!("Calling a function \"{name}\" is deprecated since 8.6"),
                    );
                }
                return self.in_function(None, |v| crate::recursion::walk_stmt(v, stmt));
            }
            StmtKind::Class(c) => {
                self.check_class_like_name(c.name.as_ref(), "a class");
                let name = c.name.as_ref().and_then(|n| n.as_str()).map(str::to_string);
                return self.in_class(name, |v| crate::recursion::walk_stmt(v, stmt));
            }
            StmtKind::Interface(d) => {
                self.check_class_like_name(Some(&d.name), "an interface");
                let name = d.name.as_str().map(str::to_string);
                return self.in_class(name, |v| crate::recursion::walk_stmt(v, stmt));
            }
            StmtKind::Trait(d) => {
                self.check_class_like_name(Some(&d.name), "a trait");
                let name = d.name.as_str().map(str::to_string);
                return self.in_class(name, |v| crate::recursion::walk_stmt(v, stmt));
            }
            StmtKind::Enum(d) => {
                self.check_class_like_name(Some(&d.name), "an enum");
                let name = d.name.as_str().map(str::to_string);
                return self.in_class(name, |v| crate::recursion::walk_stmt(v, stmt));
            }
            StmtKind::Namespace(ns) => {
                let name = ns
                    .name
                    .as_ref()
                    .map(|n| crate::parser::name_to_string_owned(n).into_owned());
                if let php_ast::owned::NamespaceBody::Braced(block) = &ns.body {
                    let outer = std::mem::replace(&mut self.namespace, name);
                    let flow = self.visit_block(block);
                    self.namespace = outer;
                    return flow;
                }
                self.namespace = name;
                return ControlFlow::Continue(());
            }
            StmtKind::Const(items) => {
                for item in items.iter() {
                    let name = item.name.as_str().unwrap_or_default();
                    // PHP matches a namespaced constant's short name case-sensitively.
                    let reserved = if self.namespace.is_some() {
                        matches!(name, "let" | "is")
                    } else {
                        is_reserved_name(name)
                    };
                    if reserved || name == "_" {
                        let lower = name.to_ascii_lowercase();
                        self.report(
                            item.name.span,
                            format!("Calling a constant \"{lower}\" is deprecated since 8.6"),
                        );
                    }
                }
            }
            StmtKind::Use(decl) => {
                for item in decl.uses.iter() {
                    let imported = match &item.alias {
                        Some(alias) => alias.as_str().unwrap_or_default().to_string(),
                        None => last_segment(&crate::parser::name_to_string_owned(&item.name))
                            .to_string(),
                    };
                    let message = match item.kind.unwrap_or(decl.kind) {
                        UseKind::Normal if is_reserved_name(&imported) || imported == "_" => {
                            format!("Using \"{imported}\" as a class name is deprecated")
                        }
                        UseKind::Function if is_reserved_name(&imported) => format!(
                            "Using \"{imported}\" as a function name is deprecated since 8.6"
                        ),
                        UseKind::Const if is_reserved_name(&imported) || imported == "_" => {
                            format!(
                                "Using \"{imported}\" as a constant name is deprecated since 8.6"
                            )
                        }
                        _ => continue,
                    };
                    self.report(item.span, message);
                }
            }
            _ => {}
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_class_member(&mut self, member: &ClassMember) -> ControlFlow<()> {
        match &member.kind {
            ClassMemberKind::Method(m) => {
                return self.visit_method(m, |v| walk_owned_class_member(v, member));
            }
            ClassMemberKind::ClassConst(c) if is_namespace_name(&c.name) => {
                self.report(
                    c.name.span,
                    "Declaring class constant called 'namespace' is deprecated".into(),
                );
            }
            _ => {}
        }
        walk_owned_class_member(self, member)
    }

    fn visit_enum_member(&mut self, member: &EnumMember) -> ControlFlow<()> {
        match &member.kind {
            EnumMemberKind::Method(m) => {
                return self.visit_method(m, |v| walk_owned_enum_member(v, member));
            }
            EnumMemberKind::ClassConst(c) if is_namespace_name(&c.name) => {
                self.report(
                    c.name.span,
                    "Declaring class constant called 'namespace' is deprecated".into(),
                );
            }
            _ => {}
        }
        walk_owned_enum_member(self, member)
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        match &expr.kind {
            ExprKind::Closure(_) | ExprKind::ArrowFunction(_) => {
                return self.in_function(None, |v| crate::recursion::walk_expr(v, expr));
            }
            ExprKind::AnonymousClass(_) => {
                return self.in_class(None, |v| crate::recursion::walk_expr(v, expr));
            }
            ExprKind::Yield(_) if !self.generator_reported => {
                self.generator_reported = self.magic.is_some();
                match self.magic {
                    Some(Magic::Constructor) => self.report(
                        expr.span,
                        "Making a constructor a Generator is deprecated".into(),
                    ),
                    Some(Magic::Destructor) => self.report(
                        expr.span,
                        "Making a destructor a Generator is deprecated".into(),
                    ),
                    None => {}
                }
            }
            _ => {}
        }
        crate::recursion::walk_expr(self, expr)
    }
}
