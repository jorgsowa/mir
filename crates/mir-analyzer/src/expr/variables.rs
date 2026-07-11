use std::sync::Arc;

use super::helpers::extract_simple_var;
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::symbol::ReferenceKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::owned::Expr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_variable(
        &mut self,
        name: &str,
        expr: &Expr,
        ctx: &mut FlowState,
    ) -> Type {
        let name_str = name.trim_start_matches('$');
        // Interned once: this runs for every `$var` read, and each string-keyed
        // FlowState call would re-hash + lock the global interner.
        let sym = mir_types::Name::from(name_str);
        // View template files (blade templates and files under resources/views/) have
        // variables injected from the calling scope, so undefined-variable diagnostics
        // are false positives there.
        let is_view_template = crate::diagnostics::is_view_template_path(&self.file);
        if !ctx.var_is_defined_sym(sym)
            && !self.in_existence_check
            && !is_view_template
            && !ctx.has_dynamic_var_def
        {
            if ctx.var_possibly_defined_sym(sym) {
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
        ctx.read_vars.insert(sym);
        ctx.mark_consumed_sym(sym);
        let ty = if name_str == "this" && !ctx.var_is_defined_sym(sym) {
            Type::never()
        } else {
            ctx.get_var_sym(sym)
        };
        self.record_symbol(
            expr.span,
            ReferenceKind::Variable(Arc::from(name_str)),
            ty.clone(),
        );
        ty
    }

    pub(super) fn analyze_variable_variable(&mut self, inner: &Expr, ctx: &mut FlowState) -> Type {
        let inner_ty = self.analyze(inner, ctx);
        if let Some(var_name) = extract_simple_var(inner) {
            ctx.read_vars
                .insert(mir_types::Name::from(var_name.as_str()));
            for atomic in &inner_ty.types {
                if let Atomic::TLiteralString(accessed_var_name) = atomic {
                    ctx.read_vars
                        .insert(mir_types::Name::from(accessed_var_name.as_ref()));
                }
            }
        }
        Type::mixed()
    }

    pub(super) fn analyze_identifier(
        &mut self,
        name: &str,
        expr: &Expr,
        ctx: &mut FlowState,
    ) -> Type {
        let name_str: &str = name;
        let name_str = name_str.strip_prefix('\\').unwrap_or(name_str);
        let ns_qualified = self
            .db
            .file_namespace(self.file.as_ref())
            .map(|ns| format!("{}\\{}", ns, name_str));

        let resolve_pull = |fqn: &str| -> Option<mir_types::Type> {
            let here = crate::db::Fqcn::from_str(self.db, fqn);
            crate::db::find_global_constant(self.db, here).map(|arc_union| (*arc_union).clone())
        };

        let ty = ns_qualified
            .as_deref()
            .and_then(resolve_pull)
            .or_else(|| resolve_pull(name_str));

        if let Some(ty) = ty {
            ty
        } else if ctx.defined_guards.contains(name_str)
            || ns_qualified
                .as_deref()
                .is_some_and(|q| ctx.defined_guards.contains(q))
        {
            // Guarded by `defined('NAME')` — the constant is defined at runtime.
            Type::mixed()
        } else {
            self.emit(
                IssueKind::UndefinedConstant {
                    name: name_str.to_string(),
                },
                Severity::Error,
                expr.span,
            );
            Type::mixed()
        }
    }
}
