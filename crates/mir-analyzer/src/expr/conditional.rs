use std::sync::Arc;

use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{atomic::Atomic, Type};
use php_ast::ast::UnaryPrefixOp;
use php_ast::owned::{Expr, ExprKind, MatchArm, MatchExpr, NullCoalesceExpr, TernaryExpr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_ternary(&mut self, t: &TernaryExpr, ctx: &mut FlowState) -> Type {
        let cond_ty = self.analyze(&t.condition, ctx);
        match &t.then_expr {
            Some(then_expr) => {
                let pre_ctx = ctx.clone();
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

                // A variable assigned inside a branch expression (e.g. `$c ? ($z =
                // "yes") : ($z = "no")`) is a real, permanent assignment on
                // whichever side actually ran — merge full variable state back,
                // not just read/consumed-write bookkeeping.
                *ctx = FlowState::merge_branches(&pre_ctx, then_ctx, Some(else_ctx));
                let mut merged = then_ty;
                merged.merge_with(&else_ty);
                merged
            }
            None => {
                // `$cond ?: $else`: `$else` only executes when `$cond` is
                // falsy — analyze it against a branched context so an
                // assignment there (`$cond ?: ($y = default())`) is treated
                // as conditional, not as always executing.
                let pre_ctx = ctx.clone();
                let mut else_ctx = ctx.branch();
                let else_ty = self.analyze(&t.else_expr, &mut else_ctx);
                *ctx = FlowState::merge_branches(&pre_ctx, pre_ctx.clone(), Some(else_ctx));
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
        // The RHS of `??` only executes when the LHS is null/undefined —
        // analyze it against a branched context so an assignment there
        // (`$x ?? ($y = default())`) is treated as conditional, not as
        // always executing.
        let pre_ctx = ctx.clone();
        let mut right_ctx = ctx.branch();
        let right_ty = self.analyze(&nc.right, &mut right_ctx);
        *ctx = FlowState::merge_branches(&pre_ctx, pre_ctx.clone(), Some(right_ctx));
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

        // `match (gettype($x)) { "int" => … }`: an arm whose string `gettype()`
        // can never return is dead (gettype returns "integer", not "int").
        if let Some(arg) = crate::contradiction::gettype_call_arg(&m.subject) {
            let arg_ty = self.analyze(arg, ctx);
            let possible = crate::contradiction::gettype_possible_values(&arg_ty);
            for arm in m.arms.iter() {
                let Some(conditions) = &arm.conditions else {
                    continue;
                };
                for cond in conditions.iter() {
                    let ExprKind::String(s) = &cond.kind else {
                        continue;
                    };
                    let s = s.as_ref();
                    let reason = if !crate::contradiction::gettype_is_valid(s) {
                        let hint = crate::contradiction::gettype_suggestion(s)
                            .map(|h| format!(" (did you mean \"{h}\"?)"))
                            .unwrap_or_default();
                        Some(format!("gettype() never returns \"{s}\"{hint}"))
                    } else if possible
                        .as_ref()
                        .is_some_and(|poss| poss.iter().all(|p| *p != s))
                    {
                        Some(format!("gettype() of {arg_ty} never returns \"{s}\""))
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        self.emit(
                            IssueKind::UnevaluatedCode { reason },
                            Severity::Info,
                            cond.span,
                        );
                    }
                }
            }
        }

        let subject_var = match &m.subject.kind {
            ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
            _ => None,
        };

        let pre_ctx = ctx.clone();
        let mut result = Type::empty();
        let mut arm_ctxs: Vec<FlowState> = Vec::new();
        for arm in m.arms.iter() {
            let mut arm_ctx = ctx.branch();
            // Always analyze conditions to check for undefined classes and get
            // types. Analyze against `arm_ctx` (not the parent) so an assignment
            // in a condition — `match (true) { ($g = f()) !== '' => $g, ... }` —
            // defines the variable for use in that arm's body.
            let mut arm_ty = Type::empty();
            if let Some(conditions) = &arm.conditions {
                for cond in conditions.iter() {
                    let cond_ty = self.analyze(cond, &mut arm_ctx);
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
            // Narrow the arm context based on the condition expressions.
            // Comma-separated conditions are OR semantics (the arm fires if
            // ANY is true) — `$x instanceof A, $x instanceof B` must narrow
            // $x to A|B, not have each instanceof applied in sequence (which
            // would AND-compose them and collapse to the last disjunct).
            if let Some(conditions) = &arm.conditions {
                let refs: Vec<&php_ast::owned::Expr> = conditions.iter().collect();
                let narrowed = crate::narrowing::narrow_instanceof_disjuncts(
                    &refs,
                    &mut arm_ctx,
                    self.db,
                    &self.file,
                )
                .is_some()
                    || crate::narrowing::narrow_type_fn_disjuncts(&refs, &mut arm_ctx).is_some()
                    || crate::narrowing::narrow_mixed_disjuncts(
                        &refs,
                        &mut arm_ctx,
                        self.db,
                        &self.file,
                    )
                    .is_some()
                    || crate::narrowing::narrow_prop_type_fn_disjuncts(
                        &refs,
                        &mut arm_ctx,
                        self.db,
                        &self.file,
                    )
                    .is_some();
                if !narrowed {
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
            }
            let arm_body_ty = self.analyze(&arm.body, &mut arm_ctx);
            result.merge_with(&arm_body_ty);
            arm_ctxs.push(arm_ctx);
        }

        // Exactly one arm's body runs (or PHP throws UnhandledMatchError), so a
        // variable assigned inside every arm — `$y = match($x) { 1 => $z = "a",
        // default => $z = "b" }` — is a real, permanent assignment; merge full
        // variable state back rather than just read/consumed-write bookkeeping.
        let mut merged: Option<FlowState> = None;
        for arm_ctx in arm_ctxs {
            merged = Some(match merged {
                Some(m) => FlowState::merge_branches(&pre_ctx, arm_ctx, Some(m)),
                None => arm_ctx,
            });
        }
        if let Some(m) = merged {
            *ctx = m;
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

    /// Resolves a `Class::CONST` (including `self::`/`static::`) match-arm
    /// condition to the constant's literal scalar value, when statically known —
    /// so `match ($x) { C::A => ..., C::B => ... }` folds into the same
    /// covered-literal set as inline `'a'`/`1` conditions instead of leaving the
    /// match looking non-exhaustive.
    fn resolve_class_const_literal(&self, cond: &Expr, ctx: &FlowState) -> Option<Atomic> {
        let ExprKind::ClassConstAccess(cca) = &cond.kind else {
            return None;
        };
        let resolved_class = match &cca.class.kind {
            ExprKind::Identifier(id) => {
                let r = crate::db::resolve_name(self.db, &self.file, id.as_ref());
                if r == "self" || r == "static" {
                    ctx.self_fqcn.as_deref().unwrap_or("").to_string()
                } else {
                    r
                }
            }
            _ => return None,
        };
        if resolved_class.is_empty() {
            return None;
        }
        let member = match &cca.member.kind {
            ExprKind::Identifier(s) | ExprKind::Variable(s) => s.as_ref(),
            _ => return None,
        };
        let here = crate::db::Fqcn::new(self.db, mir_types::Name::new(&resolved_class));
        let (_, cdef) = crate::db::find_class_constant_in_chain(self.db, here, member)?;
        match cdef.ty.types.as_slice() {
            [Atomic::TLiteralString(s)] => Some(Atomic::TLiteralString(s.clone())),
            [Atomic::TLiteralInt(n)] => Some(Atomic::TLiteralInt(*n)),
            _ => None,
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

        // Whether the subject can be `null` — used by Case 1/1b below to fold a
        // missing `null` arm into their own uncovered-set, mirroring how Case 2
        // (enum subjects) already tracks it.
        let subject_is_nullable = subject_ty.contains(|t| matches!(t, Atomic::TNull));
        let non_null_type_count = subject_ty
            .types
            .iter()
            .filter(|t| !matches!(t, Atomic::TNull))
            .count();

        // Case 1: Subject is a finite union of string literals, optionally nullable.
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
        if !string_atoms.is_empty() && string_atoms.len() == non_null_type_count {
            let mut null_covered = false;
            let covered: rustc_hash::FxHashSet<Box<str>> = arms
                .iter()
                .filter_map(|a| a.conditions.as_deref())
                .flatten()
                .filter_map(|cond| {
                    if matches!(cond.kind, ExprKind::Null) {
                        null_covered = true;
                        return None;
                    }
                    if let ExprKind::String(s) = &cond.kind {
                        Some(s.clone())
                    } else if let Some(Atomic::TLiteralString(s)) =
                        self.resolve_class_const_literal(cond, ctx)
                    {
                        Some(Box::from(s.as_ref()))
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

            if !uncovered.is_empty() || (subject_is_nullable && !null_covered) {
                let mut cases: Vec<String> = uncovered.iter().map(|s| format!("\"{s}\"")).collect();
                if subject_is_nullable && !null_covered {
                    cases.push("null".to_string());
                }
                return Some(cases.join(", "));
            }
            return None;
        }

        // Case 1b: Subject is a finite union of integer literals and/or small
        // bounded int ranges — `int<0, 2>` is just as enumerable as `0|1|2`,
        // so it's expanded into the same literal set rather than falling
        // through to Case 4's "unconditionally possibly-unmatched" bucket.
        // Also optionally nullable, same as Case 1 above.
        const MAX_RANGE_EXPANSION: i64 = 4096;
        let int_atoms: Option<Vec<i64>> = subject_ty
            .types
            .iter()
            .filter(|a| !matches!(a, Atomic::TNull))
            .map(|a| match a {
                Atomic::TLiteralInt(n) => Some(vec![*n]),
                Atomic::TIntRange {
                    min: Some(lo),
                    max: Some(hi),
                } if *hi >= *lo && *hi - *lo < MAX_RANGE_EXPANSION => Some((*lo..=*hi).collect()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|nested| {
                let mut values: Vec<i64> = nested.into_iter().flatten().collect();
                values.sort_unstable();
                values.dedup();
                values
            });
        if let Some(int_atoms) = int_atoms.filter(|v| !v.is_empty()) {
            let mut null_covered = false;
            let covered: rustc_hash::FxHashSet<i64> = arms
                .iter()
                .filter_map(|a| a.conditions.as_deref())
                .flatten()
                .filter_map(|cond| {
                    if matches!(cond.kind, ExprKind::Null) {
                        null_covered = true;
                        return None;
                    }
                    extract_literal_int(cond).or_else(|| {
                        if let Some(Atomic::TLiteralInt(n)) =
                            self.resolve_class_const_literal(cond, ctx)
                        {
                            Some(n)
                        } else {
                            None
                        }
                    })
                })
                .collect();

            let mut uncovered: Vec<i64> = int_atoms
                .iter()
                .filter(|n| !covered.contains(*n))
                .copied()
                .collect();
            uncovered.sort_unstable();

            if !uncovered.is_empty() || (subject_is_nullable && !null_covered) {
                let mut cases: Vec<String> = uncovered.iter().map(|n| n.to_string()).collect();
                if subject_is_nullable && !null_covered {
                    cases.push("null".to_string());
                }
                return Some(cases.join(", "));
            }
            return None;
        }

        // Case 2: Subject is a single named object (or self/static, possibly
        // nullable) that is an enum. A backed enum's case *set* is just as
        // finite and enumerable as a pure enum's — the backing scalar (its
        // value range) is irrelevant to exhaustiveness over case names, so
        // this no longer excludes backed enums.
        let non_null_atoms: Vec<&Atomic> = subject_ty
            .types
            .iter()
            .filter(|t| !matches!(t, Atomic::TNull))
            .collect();
        let enum_fqcn_opt: Option<String> = if non_null_atoms.len() == 1 {
            match non_null_atoms[0] {
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
                let mut null_covered = false;
                let covered: rustc_hash::FxHashSet<String> = arms
                    .iter()
                    .filter_map(|a| a.conditions.as_deref())
                    .flatten()
                    .filter_map(|cond| {
                        if matches!(cond.kind, ExprKind::Null) {
                            null_covered = true;
                            return None;
                        }
                        if let ExprKind::ClassConstAccess(cca) = &cond.kind {
                            let resolved_class = match &cca.class.kind {
                                ExprKind::Identifier(id) => {
                                    let r =
                                        crate::db::resolve_name(self.db, &self.file, id.as_ref());
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
                            Some(crate::util::php_ident_lowercase(member))
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut uncovered: Vec<String> = enum_def
                    .cases
                    .keys()
                    .filter(|k| !covered.contains(&crate::util::php_ident_lowercase(k.as_ref())))
                    .map(|k| format!("{enum_fqcn}::{k}"))
                    .collect();
                uncovered.sort();
                if subject_is_nullable && !null_covered {
                    uncovered.push("null".to_string());
                }

                if !uncovered.is_empty() {
                    return Some(uncovered.join(", "));
                }
                return None;
            }
        }

        // Case 3: Subject is entirely bool (TBool/TTrue/TFalse) — exhaustive
        // only when both `true` and `false` arms are present, since bool has
        // exactly two values. Excludes the `match(true)`/`match(false)`
        // chained-condition idiom, where the subject is a single literal
        // TTrue/TFalse constant and arms are arbitrary boolean expressions
        // (not literal true/false values) — exhaustiveness there depends on
        // the arms' condition coverage, which this function doesn't attempt
        // to prove.
        let is_match_true_idiom = subject_ty.types.len() == 1
            && matches!(subject_ty.types[0], Atomic::TTrue | Atomic::TFalse);
        if !is_match_true_idiom
            && !subject_ty.types.is_empty()
            && subject_ty
                .types
                .iter()
                .all(|a| matches!(a, Atomic::TBool | Atomic::TTrue | Atomic::TFalse))
        {
            let conds: Vec<&Expr> = arms
                .iter()
                .filter_map(|a| a.conditions.as_deref())
                .flatten()
                .collect();
            let has_true = conds.iter().any(|c| matches!(c.kind, ExprKind::Bool(true)));
            let has_false = conds
                .iter()
                .any(|c| matches!(c.kind, ExprKind::Bool(false)));
            if has_true && has_false {
                return None;
            }
            let mut missing = Vec::new();
            if !has_true {
                missing.push("true");
            }
            if !has_false {
                missing.push("false");
            }
            return Some(missing.join(", "));
        }

        // Case 4: Subject is an unconstrained scalar (plain int/string/float,
        // not a finite literal union or enum) with no default arm. No finite
        // set of literal arms can ever prove exhaustiveness here — PHP throws
        // UnhandledMatchError for any value the arms don't happen to list.
        // Cases 1/1b above already return early for a subject that IS a
        // finite literal union, so reaching here with a scalar atom means at
        // least one atom is genuinely unbounded.
        if !subject_ty.types.is_empty()
            && subject_ty.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TInt
                        | Atomic::TPositiveInt
                        | Atomic::TNonNegativeInt
                        | Atomic::TNegativeInt
                        | Atomic::TIntRange { .. }
                        | Atomic::TString
                        | Atomic::TNonEmptyString
                        | Atomic::TNumericString
                        | Atomic::TFloat
                        | Atomic::TIntegralFloat
                        | Atomic::TNumeric
                )
            })
        {
            return Some(format!("possibly-unmatched value of type '{subject_ty}'"));
        }

        None
    }
}

/// Extract a literal integer value from a match-arm condition expression.
/// PHP parses `-1` as `UnaryPrefix(Negate, Int(1))`, not `Int(-1)`.
fn extract_literal_int(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::Int(n) => Some(*n),
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::Negate => {
            if let ExprKind::Int(n) = &u.operand.kind {
                n.checked_neg()
            } else {
                None
            }
        }
        ExprKind::Parenthesized(inner) => extract_literal_int(inner),
        _ => None,
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
                | Atomic::TIntegralFloat
                | Atomic::TLiteralFloat(..)
                | Atomic::TBool
                | Atomic::TTrue
                | Atomic::TFalse
                | Atomic::TNull
        )
    })
}
