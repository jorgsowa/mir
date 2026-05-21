use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_types::Union;
use php_ast::owned::{ExprKind, MatchExpr, NullCoalesceExpr, TernaryExpr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_ternary(&mut self, t: &TernaryExpr, ctx: &mut Context) -> Union {
        let cond_ty = self.analyze(&t.condition, ctx);
        match &t.then_expr {
            Some(then_expr) => {
                let mut then_ctx = ctx.fork();
                crate::narrowing::narrow_from_condition(
                    &t.condition,
                    &mut then_ctx,
                    true,
                    self.db,
                    &self.file,
                );
                let then_ty = self.analyze(then_expr, &mut then_ctx);

                let mut else_ctx = ctx.fork();
                crate::narrowing::narrow_from_condition(
                    &t.condition,
                    &mut else_ctx,
                    false,
                    self.db,
                    &self.file,
                );
                let else_ty = self.analyze(&t.else_expr, &mut else_ctx);

                for name in then_ctx.read_vars.iter().chain(else_ctx.read_vars.iter()) {
                    ctx.read_vars.insert(name.clone());
                }
                Union::merge(&then_ty, &else_ty)
            }
            None => {
                let else_ty = self.analyze(&t.else_expr, ctx);
                let truthy_ty = cond_ty.narrow_to_truthy();
                if truthy_ty.is_empty() {
                    else_ty
                } else {
                    Union::merge(&truthy_ty, &else_ty)
                }
            }
        }
    }

    pub(super) fn analyze_null_coalesce(
        &mut self,
        nc: &NullCoalesceExpr,
        ctx: &mut Context,
    ) -> Union {
        let left_ty = self.analyze(&nc.left, ctx);
        let right_ty = self.analyze(&nc.right, ctx);
        let non_null_left = left_ty.remove_null();
        if non_null_left.is_empty() {
            right_ty
        } else {
            Union::merge(&non_null_left, &right_ty)
        }
    }

    pub(super) fn analyze_match(&mut self, m: &MatchExpr, ctx: &mut Context) -> Union {
        let subject_ty = self.analyze(&m.subject, ctx);
        let subject_var = match &m.subject.kind {
            ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
            _ => None,
        };

        let mut result = Union::empty();
        for arm in m.arms.iter() {
            let mut arm_ctx = ctx.fork();
            // Always analyze conditions to check for undefined classes and get types
            let mut arm_ty = Union::empty();
            if let Some(conditions) = &arm.conditions {
                for cond in conditions.iter() {
                    let cond_ty = self.analyze(cond, ctx);
                    arm_ty = Union::merge(&arm_ty, &cond_ty);
                }
            }
            // Use type narrowing if the subject is a variable
            if let Some(var) = &subject_var {
                if !arm_ty.is_empty() && !arm_ty.is_mixed() {
                    let narrowed = subject_ty.intersect_with(&arm_ty);
                    if !narrowed.is_empty() {
                        arm_ctx.set_var(var, narrowed);
                    }
                }
            }
            // Narrow the arm context based on the condition expressions
            if let Some(conditions) = &arm.conditions {
                for cond in conditions.iter() {
                    crate::narrowing::narrow_from_condition(
                        cond,
                        &mut arm_ctx,
                        true,
                        self.db,
                        &self.file,
                    );
                }
            }
            let arm_body_ty = self.analyze(&arm.body, &mut arm_ctx);
            result = Union::merge(&result, &arm_body_ty);
            for name in &arm_ctx.read_vars {
                ctx.read_vars.insert(name.clone());
            }
        }
        if result.is_empty() {
            Union::mixed()
        } else {
            result
        }
    }
}
