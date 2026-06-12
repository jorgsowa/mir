use std::sync::Arc;

use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{atomic::Atomic, Type};
use php_ast::owned::{ExprKind, MatchArm, MatchExpr, NullCoalesceExpr, TernaryExpr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_ternary(&mut self, t: &TernaryExpr, ctx: &mut FlowState) -> Type {
        let cond_ty = self.analyze(&t.condition, ctx);
        match &t.then_expr {
            Some(then_expr) => {
                let mut then_ctx = ctx.branch();
                crate::narrowing::narrow_from_condition(
                    &t.condition,
                    &mut then_ctx,
                    true,
                    self.db,
                    &self.file,
                );
                let then_ty = self.analyze(then_expr, &mut then_ctx);

                let mut else_ctx = ctx.branch();
                crate::narrowing::narrow_from_condition(
                    &t.condition,
                    &mut else_ctx,
                    false,
                    self.db,
                    &self.file,
                );
                let else_ty = self.analyze(&t.else_expr, &mut else_ctx);

                ctx.absorb_branch_reads(&then_ctx);
                ctx.absorb_branch_reads(&else_ctx);
                let mut merged = then_ty;
                merged.merge_with(&else_ty);
                merged
            }
            None => {
                let else_ty = self.analyze(&t.else_expr, ctx);
                let truthy_ty = cond_ty.narrow_to_truthy();
                if truthy_ty.is_empty() {
                    else_ty
                } else {
                    let mut merged = truthy_ty;
                    merged.merge_with(&else_ty);
                    merged
                }
            }
        }
    }

    pub(super) fn analyze_null_coalesce(
        &mut self,
        nc: &NullCoalesceExpr,
        ctx: &mut FlowState,
    ) -> Type {
        let left_ty = self.with_existence_check(|ea| ea.analyze(&nc.left, ctx));
        let right_ty = self.analyze(&nc.right, ctx);
        let non_null_left = left_ty.remove_null();
        if non_null_left.is_empty() {
            right_ty
        } else {
            let mut merged = non_null_left;
            merged.merge_with(&right_ty);
            merged
        }
    }

    pub(super) fn analyze_match(
        &mut self,
        m: &MatchExpr,
        span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        // Flag match-arm conditions whose literal value repeats an earlier arm —
        // the duplicate branch can never be reached. Only literal conditions are
        // compared, so dynamic conditions are never flagged.
        let conditions = m
            .arms
            .iter()
            .filter_map(|a| a.conditions.as_deref())
            .flatten();
        for (span, value) in crate::expr::duplicate_literal_conditions(conditions) {
            self.emit(
                IssueKind::ParadoxicalCondition { value },
                Severity::Warning,
                span,
            );
        }

        let subject_ty = self.analyze(&m.subject, ctx);
        let subject_var = match &m.subject.kind {
            ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
            _ => None,
        };

        let mut result = Type::empty();
        for arm in m.arms.iter() {
            let mut arm_ctx = ctx.branch();
            // Always analyze conditions to check for undefined classes and get types
            let mut arm_ty = Type::empty();
            if let Some(conditions) = &arm.conditions {
                for cond in conditions.iter() {
                    let cond_ty = self.analyze(cond, ctx);
                    arm_ty.merge_with(&cond_ty);
                }
            }
            // Use type narrowing if the subject is a variable
            if let Some(var) = &subject_var {
                if !arm_ty.is_empty() && !arm_ty.is_mixed() {
                    let narrowed = subject_ty.intersect_with(&arm_ty);
                    if !subject_ty.is_mixed()
                        && narrowed.is_never()
                        && is_scalar_union(&subject_ty)
                        && is_scalar_union(&arm_ty)
                    {
                        if let Some(conditions) = &arm.conditions {
                            for cond in conditions {
                                self.emit(
                                    IssueKind::TypeDoesNotContainType {
                                        left: format!("{subject_ty}"),
                                        right: format!("{arm_ty}"),
                                    },
                                    Severity::Info,
                                    cond.span,
                                );
                            }
                        }
                    }
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
            result.merge_with(&arm_body_ty);
            ctx.absorb_branch_reads(&arm_ctx);
        }

        // Exhaustiveness check: emit UnhandledMatchCondition if the match does not
        // cover all possible values and has no default (conditions: None) arm.
        if let Some(detail) = self.check_match_exhaustiveness(&subject_ty, &m.arms, ctx) {
            self.emit(
                IssueKind::UnhandledMatchCondition { detail },
                Severity::Warning,
                span,
            );
        }

        if result.is_empty() {
            Type::mixed()
        } else {
            result
        }
    }

    fn check_match_exhaustiveness(
        &self,
        subject_ty: &Type,
        arms: &[MatchArm],
        ctx: &FlowState,
    ) -> Option<String> {
        // An empty match has no arms at all — every value is unmatched.
        if arms.is_empty() {
            return Some("no arms".to_string());
        }

        // A default arm (conditions: None) makes the match exhaustive.
        if arms.iter().any(|a| a.conditions.is_none()) {
            return None;
        }

        // Case 1: Subject is a finite union of string literals.
        let string_atoms: Vec<Arc<str>> = subject_ty
            .types
            .iter()
            .filter_map(|a| {
                if let Atomic::TLiteralString(s) = a {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();
        if !string_atoms.is_empty() && string_atoms.len() == subject_ty.types.len() {
            let covered: rustc_hash::FxHashSet<Box<str>> = arms
                .iter()
                .filter_map(|a| a.conditions.as_deref())
                .flatten()
                .filter_map(|cond| {
                    if let ExprKind::String(s) = &cond.kind {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let mut uncovered: Vec<&str> = string_atoms
                .iter()
                .filter(|s| !covered.contains(s.as_ref()))
                .map(|s| s.as_ref())
                .collect();
            uncovered.sort();

            if !uncovered.is_empty() {
                let cases = uncovered
                    .iter()
                    .map(|s| format!("\"{s}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Some(cases);
            }
            return None;
        }

        // Case 2: Subject is a single named object (or self/static) that is a pure
        // (non-backed) enum. Extract the FQCN from TNamedObject, TSelf, or TStaticObject.
        let enum_fqcn_opt: Option<String> = if subject_ty.types.len() == 1 {
            match &subject_ty.types[0] {
                Atomic::TNamedObject { fqcn, .. } => Some(fqcn.to_string()),
                Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => {
                    if fqcn.is_empty() {
                        ctx.self_fqcn.as_deref().map(str::to_string)
                    } else {
                        Some(fqcn.to_string())
                    }
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(enum_fqcn) = enum_fqcn_opt {
            let here = crate::db::Fqcn::new(self.db, mir_types::Name::new(&enum_fqcn));
            if let Some(crate::db::ClassLike::Enum(enum_def)) =
                crate::db::find_class_like(self.db, here)
            {
                if enum_def.scalar_type.is_none() {
                    let covered: rustc_hash::FxHashSet<String> = arms
                        .iter()
                        .filter_map(|a| a.conditions.as_deref())
                        .flatten()
                        .filter_map(|cond| {
                            if let ExprKind::ClassConstAccess(cca) = &cond.kind {
                                let resolved_class = match &cca.class.kind {
                                    ExprKind::Identifier(id) => {
                                        let r = crate::db::resolve_name(
                                            self.db,
                                            &self.file,
                                            id.as_ref(),
                                        );
                                        if r == "self" || r == "static" {
                                            ctx.self_fqcn.as_deref().unwrap_or("").to_string()
                                        } else {
                                            r
                                        }
                                    }
                                    _ => return None,
                                };
                                if !resolved_class.eq_ignore_ascii_case(&enum_fqcn) {
                                    return None;
                                }
                                let member = match &cca.member.kind {
                                    ExprKind::Identifier(s) | ExprKind::Variable(s) => s.as_ref(),
                                    _ => return None,
                                };
                                Some(member.to_lowercase())
                            } else {
                                None
                            }
                        })
                        .collect();

                    let mut uncovered: Vec<&str> = enum_def
                        .cases
                        .keys()
                        .filter(|k| !covered.contains(&k.to_lowercase()))
                        .map(|k| k.as_ref())
                        .collect();
                    uncovered.sort();

                    if !uncovered.is_empty() {
                        let cases = uncovered
                            .iter()
                            .map(|c| format!("{enum_fqcn}::{c}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Some(cases);
                    }
                    return None;
                }
            }
        }

        None
    }
}

/// Returns true when every atomic in `ty` is a scalar or literal type (string, int, float,
/// bool, null, or their literal variants). Named object types are excluded because class
/// hierarchies make the "can never contain" check unreliable without full inheritance data.
fn is_scalar_union(ty: &Type) -> bool {
    ty.types.iter().all(|a| {
        matches!(
            a,
            Atomic::TString
                | Atomic::TLiteralString(_)
                | Atomic::TInt
                | Atomic::TLiteralInt(_)
                | Atomic::TFloat
                | Atomic::TLiteralFloat(..)
                | Atomic::TBool
                | Atomic::TTrue
                | Atomic::TFalse
                | Atomic::TNull
        )
    })
}
