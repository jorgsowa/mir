use std::sync::Arc;

use mir_codebase::storage::{
    wrap_return_type, wrap_template_bound, FnParam, FunctionDef, TemplateParam,
};
use mir_types::Name;

use super::DefinitionCollector;
use crate::parser::type_from_hint_owned;

/// Returns `true` if `stmts` (does not recurse into nested function/closure
/// bodies) contain a call to `func_get_args()`, `func_get_arg()`, or
/// `func_num_args()`.
///
/// When a function uses these PHP intrinsics it can accept more positional
/// arguments than its declared parameter list suggests. We add a synthetic
/// trailing variadic parameter so the arity checker does not emit
/// `TooManyArguments` for such functions.
#[allow(clippy::redundant_closure)]
pub(crate) fn stmts_use_func_get_args(stmts: &[php_ast::owned::Stmt]) -> bool {
    use php_ast::owned::{ExprKind, StmtKind};

    fn check_expr(expr: &php_ast::owned::Expr) -> bool {
        use ExprKind::*;
        match &expr.kind {
            FunctionCall(call) => {
                if let Identifier(name) = &call.name.kind {
                    if matches!(
                        name.as_ref(),
                        "func_get_args" | "func_get_arg" | "func_num_args"
                    ) {
                        return true;
                    }
                }
                call.args.iter().any(|a| check_expr(&a.value))
            }
            // Do NOT descend into new function/closure/arrow-fn bodies.
            Closure(_) | ArrowFunction(_) | AnonymousClass(_) => false,
            Assign(e) => check_expr(&e.target) || check_expr(&e.value),
            Binary(e) => check_expr(&e.left) || check_expr(&e.right),
            UnaryPrefix(e) => check_expr(&e.operand),
            UnaryPostfix(e) => check_expr(&e.operand),
            Cast(_, e) => check_expr(e),
            Ternary(e) => {
                check_expr(&e.condition)
                    || e.then_expr.as_ref().is_some_and(|t| check_expr(t))
                    || check_expr(&e.else_expr)
            }
            NullCoalesce(e) => check_expr(&e.left) || check_expr(&e.right),
            MethodCall(e) | NullsafeMethodCall(e) => {
                check_expr(&e.object) || e.args.iter().any(|a| check_expr(&a.value))
            }
            StaticMethodCall(e) => {
                check_expr(&e.class) || e.args.iter().any(|a| check_expr(&a.value))
            }
            StaticDynMethodCall(e) => {
                check_expr(&e.class)
                    || check_expr(&e.method)
                    || e.args.iter().any(|a| check_expr(&a.value))
            }
            New(e) => e.args.iter().any(|a| check_expr(&a.value)),
            Array(elems) => elems
                .iter()
                .any(|el| el.key.as_ref().is_some_and(|k| check_expr(k)) || check_expr(&el.value)),
            ArrayAccess(e) => {
                check_expr(&e.array) || e.index.as_ref().is_some_and(|i| check_expr(i))
            }
            PropertyAccess(e) | NullsafePropertyAccess(e) => check_expr(&e.object),
            Include(_, e) => check_expr(e),
            ThrowExpr(e) | Print(e) | Clone(e) | Empty(e) | ErrorSuppress(e) | Parenthesized(e)
            | Eval(e) | Exit(Some(e)) => check_expr(e),
            Yield(e) => {
                e.key.as_ref().is_some_and(|k| check_expr(k))
                    || e.value.as_ref().is_some_and(|v| check_expr(v))
            }
            Match(e) => {
                check_expr(&e.subject)
                    || e.arms.iter().any(|arm| {
                        arm.conditions
                            .as_ref()
                            .is_some_and(|conds| conds.iter().any(|c| check_expr(c)))
                            || check_expr(&arm.body)
                    })
            }
            Isset(exprs) => exprs.iter().any(|e| check_expr(e)),
            _ => false,
        }
    }

    fn check_stmt(stmt: &php_ast::owned::Stmt) -> bool {
        use StmtKind::*;
        match &stmt.kind {
            // Do NOT recurse into nested function/class declarations.
            Function(_) | Class(_) | Interface(_) | Trait(_) | Enum(_) => false,
            Expression(e) => check_expr(e),
            Return(Some(e)) => check_expr(e),
            Throw(e) => check_expr(e),
            Echo(exprs) => exprs.iter().any(|e| check_expr(e)),
            If(s) => {
                check_expr(&s.condition)
                    || check_stmt(&s.then_branch)
                    || s.elseif_branches
                        .iter()
                        .any(|b| check_expr(&b.condition) || check_stmt(&b.body))
                    || s.else_branch.as_ref().is_some_and(|b| check_stmt(b))
            }
            While(s) => check_expr(&s.condition) || check_stmt(&s.body),
            DoWhile(s) => check_stmt(&s.body) || check_expr(&s.condition),
            For(s) => {
                s.init.iter().any(|e| check_expr(e))
                    || s.condition.iter().any(|e| check_expr(e))
                    || s.update.iter().any(|e| check_expr(e))
                    || check_stmt(&s.body)
            }
            Foreach(s) => {
                check_expr(&s.expr)
                    || s.key.as_ref().is_some_and(|k| check_expr(k))
                    || check_expr(&s.value)
                    || check_stmt(&s.body)
            }
            Switch(s) => {
                check_expr(&s.expr)
                    || s.body.cases.iter().any(|c| {
                        c.value.as_ref().is_some_and(|cond| check_expr(cond))
                            || c.body.iter().any(|inner| check_stmt(inner))
                    })
            }
            TryCatch(t) => {
                t.body.stmts.iter().any(|s| check_stmt(s))
                    || t.catches
                        .iter()
                        .any(|c| c.body.stmts.iter().any(|s| check_stmt(s)))
                    || t.finally
                        .as_ref()
                        .is_some_and(|f| f.stmts.iter().any(|s| check_stmt(s)))
            }
            Block(b) => b.stmts.iter().any(|s| check_stmt(s)),
            _ => false,
        }
    }

    stmts.iter().any(|s| check_stmt(s))
}

impl DefinitionCollector<'_> {
    pub(super) fn collect_function(
        &mut self,
        decl: &php_ast::owned::FunctionDecl,
        stmt_span: php_ast::Span,
    ) {
        let short_name = decl.name.as_deref().unwrap_or_default().to_string();
        let fqn = if let Some(ns) = &self.namespace {
            format!("{ns}\\{short_name}")
        } else {
            short_name.clone()
        };

        let doc = self.parse_docblock_from_node(decl.doc_comment.as_ref());
        let doc_span = decl
            .doc_comment
            .as_ref()
            .map(|c| c.span.start)
            .unwrap_or(stmt_span.start);
        self.emit_docblock_issues(&doc, doc_span);

        if !self.version_allows(&doc) {
            return;
        }

        // Build template names first so bound resolution below can recognise template-param
        // names and avoid FQN-qualifying them (e.g. `@template T of K` where K is another param).
        let template_names: std::collections::HashSet<String> = doc
            .templates
            .iter()
            .map(|(n, _, _)| n.to_string())
            .collect();

        // Extract template parameters; resolve bounds with template-awareness so template
        // names used as bounds are stored as TTemplateParam, not wrongly namespace-qualified.
        let template_params = doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
                    self.resolve_union_doc_with_templates(b, &template_names, fqn.as_str(), &[])
                })),
                defining_entity: fqn.as_str().into(),
                variance: *variance,
            })
            .collect::<Vec<_>>();

        let mut params = Vec::new();
        let mut local_scalar = 0usize;
        let mut local_complex = 0usize;
        let mut local_defaults = 0usize;
        for p in decl.params.iter() {
            let param_name = p.name.as_deref().unwrap_or_default();
            let ty = doc
                .get_param_type(param_name)
                .cloned()
                .map(|u| {
                    // If the type is a simple named object that matches a template param,
                    // convert it to a TTemplateParam
                    self.resolve_union_doc_with_templates(
                        u,
                        &template_names,
                        &fqn,
                        &template_params,
                    )
                })
                .or_else(|| {
                    self.resolve_union_opt(
                        p.type_hint.as_ref().map(|h| type_from_hint_owned(h, None)),
                    )
                });
            if let Some(ty_ref) = &ty {
                if super::is_simple_scalar(ty_ref) {
                    local_scalar += 1;
                } else {
                    local_complex += 1;
                }
            }
            let has_default = p.default.is_some();
            if has_default {
                local_defaults += 1;
            }

            params.push(FnParam {
                name: Name::new(param_name),
                ty: mir_codebase::wrap_param_type(ty),
                has_default,
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: has_default || p.variadic,
            });
        }
        if local_scalar > 0 {
            super::SCALAR_PARAM_COUNT.fetch_add(local_scalar, std::sync::atomic::Ordering::Relaxed);
        }
        if local_complex > 0 {
            super::COMPLEX_PARAM_COUNT
                .fetch_add(local_complex, std::sync::atomic::Ordering::Relaxed);
        }
        if local_defaults > 0 {
            super::PARAM_WITH_DEFAULT
                .fetch_add(local_defaults, std::sync::atomic::Ordering::Relaxed);
        }

        // If the function body calls func_get_args() / func_get_arg() /
        // func_num_args(), it can accept more positional args than declared.
        // Add a synthetic untyped variadic param so TooManyArguments is not
        // emitted for such functions.
        let last_is_variadic = params.last().is_some_and(|p| p.is_variadic);
        if !last_is_variadic && stmts_use_func_get_args(&decl.body.stmts) {
            params.push(FnParam {
                name: Name::new("..."),
                ty: None,
                has_default: false,
                is_variadic: true,
                is_byref: false,
                is_optional: true,
            });
        }

        let return_type = match (doc.return_type.clone(), decl.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                Some(self.resolve_union_doc_with_templates(
                    ty,
                    &template_names,
                    &fqn,
                    &template_params,
                ))
            }
            (None, Some(h)) => self.resolve_union_opt(Some(type_from_hint_owned(h, None))),
            (None, None) => None,
        };

        let throws = doc
            .throws
            .iter()
            .map(|t| {
                Arc::from(
                    super::resolution::resolve_name(t, &self.namespace, &self.use_aliases).as_str(),
                )
            })
            .collect();

        let docstring = if doc.description.trim().is_empty() {
            None
        } else {
            Some(Arc::from(doc.description.as_str()))
        };

        let storage = FunctionDef {
            fqn: fqn.clone().into(),
            short_name: short_name.into(),
            params: Arc::from(params.into_boxed_slice()),
            return_type: wrap_return_type(return_type),
            inferred_return_type: None,
            template_params,
            assertions: self.build_assertions(&doc),
            throws,
            deprecated: doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                // Only detect #[Deprecated] without arguments (no-arg form used in
                // user code). Stubs use #[Deprecated(since: '...', ...)] with args
                // which would otherwise flood callers with spurious DeprecatedCall.
                if decl.attributes.iter().any(|a| {
                    a.args.is_empty()
                        && a.name
                            .parts
                            .last()
                            .map(|p| p.as_ref().eq_ignore_ascii_case("Deprecated"))
                            .unwrap_or(false)
                }) {
                    Some(Arc::from(""))
                } else {
                    None
                }
            }),
            is_pure: doc.is_pure,
            location: Some(self.location(stmt_span.start, stmt_span.end)),
            docstring,
        };

        self.slice.functions.push(std::sync::Arc::new(storage));

        // Scan the function body for `@var`-annotated global declarations.
        self.scan_stmts_for_global_vars(&decl.body.stmts);
    }

    pub(super) fn collect_global_stmt(&mut self, stmt: &php_ast::owned::Stmt) {
        // Top-level `global $x` — unusual in PHP but valid.
        self.try_collect_global_var_annotation(stmt);
    }
}
