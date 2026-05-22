use std::sync::Arc;

use super::helpers::extract_simple_var;
use super::ExpressionAnalyzer;
use crate::context::Context;
use crate::symbol::SymbolKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Symbol, Union};
use php_ast::owned::Expr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_variable(&mut self, name: &str, expr: &Expr, ctx: &mut Context) -> Union {
        let name_str = name.trim_start_matches('$');
        if !ctx.var_is_defined(name_str) {
            if ctx.var_possibly_defined(name_str) {
                self.emit(
                    IssueKind::PossiblyUndefinedVariable {
                        name: name_str.to_string(),
                    },
                    Severity::Warning,
                    expr.span,
                );
            } else if name_str == "this" {
                self.emit(
                    IssueKind::InvalidScope {
                        in_class: ctx.self_fqcn.is_some(),
                    },
                    Severity::Error,
                    expr.span,
                );
            } else {
                self.emit(
                    IssueKind::UndefinedVariable {
                        name: name_str.to_string(),
                    },
                    Severity::Error,
                    expr.span,
                );
            }
        }
        ctx.read_vars.insert(mir_types::Symbol::from(name_str));
        let ty = if name_str == "this" && !ctx.var_is_defined("this") {
            Union::never()
        } else {
            ctx.get_var(name_str)
        };
        self.record_symbol(
            expr.span,
            SymbolKind::Variable(Arc::from(name_str)),
            ty.clone(),
        );
        ty
    }

    pub(super) fn analyze_variable_variable(&mut self, inner: &Expr, ctx: &mut Context) -> Union {
        let inner_ty = self.analyze(inner, ctx);
        if let Some(var_name) = extract_simple_var(inner) {
            ctx.read_vars
                .insert(mir_types::Symbol::from(var_name.as_str()));
            for atomic in &inner_ty.types {
                if let Atomic::TLiteralString(accessed_var_name) = atomic {
                    ctx.read_vars
                        .insert(mir_types::Symbol::from(accessed_var_name.as_ref()));
                }
            }
        }
        Union::mixed()
    }

    pub(super) fn analyze_identifier(
        &mut self,
        name: &str,
        expr: &Expr,
        _ctx: &mut Context,
    ) -> Union {
        let name_str: &str = name;
        let name_str = name_str.strip_prefix('\\').unwrap_or(name_str);
        let ns_qualified = self
            .db
            .file_namespace(self.file.as_ref())
            .map(|ns| format!("{}\\{}", ns, name_str));

        let resolve_pull = |fqn: &str| -> Option<mir_types::Union> {
            let here = crate::db::Fqcn::new(self.db, Symbol::new(fqn));
            crate::db::find_global_constant(self.db, here).map(|arc_union| (*arc_union).clone())
        };

        let ty = ns_qualified
            .as_deref()
            .and_then(resolve_pull)
            .or_else(|| resolve_pull(name_str));

        if let Some(ty) = ty {
            ty
        } else {
            self.emit(
                IssueKind::UndefinedConstant {
                    name: name_str.to_string(),
                },
                Severity::Error,
                expr.span,
            );
            Union::mixed()
        }
    }
}
