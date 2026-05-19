use std::sync::Arc;

use super::helpers::extract_simple_var;
use super::ExpressionAnalyzer;
use crate::context::Context;
use crate::symbol::SymbolKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};
use php_ast::ast::Expr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_variable<'arena, 'src>(
        &mut self,
        name: &php_ast::ast::NameStr<'arena, 'src>,
        expr: &Expr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        let name_str = name.as_str().trim_start_matches('$');
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
        ctx.read_vars.insert(name_str.to_string());
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

    pub(super) fn analyze_variable_variable<'arena, 'src>(
        &mut self,
        inner: &Expr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        let inner_ty = self.analyze(inner, ctx);
        if let Some(var_name) = extract_simple_var(inner) {
            ctx.read_vars.insert(var_name.clone());
            for atomic in &inner_ty.types {
                if let Atomic::TLiteralString(accessed_var_name) = atomic {
                    ctx.read_vars.insert(accessed_var_name.to_string());
                }
            }
        }
        Union::mixed()
    }

    pub(super) fn analyze_identifier<'arena, 'src>(
        &mut self,
        name: &php_ast::ast::NameStr<'arena, 'src>,
        expr: &Expr<'arena, 'src>,
        _ctx: &mut Context,
    ) -> Union {
        let name_str: &str = name.as_str();
        let name_str = name_str.strip_prefix('\\').unwrap_or(name_str);
        let ns_qualified = self
            .db
            .file_namespace(self.file.as_ref())
            .map(|ns| format!("{}\\{}", ns, name_str));

        let resolve_pull = |fqn: &str| -> Option<mir_types::Union> {
            let here = crate::db::Fqcn::new(self.db, Arc::<str>::from(fqn));
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
