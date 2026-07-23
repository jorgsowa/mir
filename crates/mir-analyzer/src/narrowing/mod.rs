/// Type narrowing — refines variable types based on conditional expressions.
///
/// Given a condition expression and a branch direction (true/false), this
/// module updates the `FlowState` to narrow variable types accordingly.
use php_ast::ast::{AssignOp, BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_types::Atomic;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

mod arrays;
mod assertions;
mod class_const_compare;
mod class_introspection;
mod core;
mod enum_class;
mod instanceof_core;
mod instanceof_disjuncts;
mod literals;
mod strings;
mod type_fn;

use arrays::{
    array_access_base_target, extract_array_key_first_or_last_arg,
    extract_array_key_first_or_last_static_prop_arg, extract_haystack_type,
    haystack_admits_null_loosely, in_array_loose_narrowing_is_safe, narrow_array_count_condition,
    narrow_array_emptiness_condition, narrow_array_key_exists_condition,
    narrow_array_key_first_or_last_null, narrow_container_non_null_non_false,
    narrow_empty_shape_key, narrow_in_array_condition, narrow_isset_shape_key,
    narrow_isset_shape_key_false, narrow_prop_array_key_first_or_last_null,
    narrow_static_prop_array_key_first_or_last_null, narrow_to_haystack_values,
    strip_haystack_null,
};
pub(crate) use assertions::negate_assertion_type;
use assertions::{
    apply_docblock_assertions, apply_method_docblock_assertions, method_call_receiver_fqcn,
    resolve_static_call_class_fqcn,
};
use class_const_compare::narrow_from_static_or_class_const_comparison;
use class_introspection::{
    extract_dynamic_class_const_static_prop_var, extract_dynamic_class_const_var,
    extract_get_class_arg, extract_get_class_static_prop_arg, extract_get_debug_type_arg,
    extract_get_debug_type_static_prop_arg, extract_get_parent_class_arg,
    extract_get_parent_class_static_prop_arg, extract_gettype_arg, extract_gettype_static_prop_arg,
    narrow_from_get_debug_type_literal, narrow_from_get_parent_class_literal,
    narrow_from_gettype_literal, narrow_static_prop_from_get_debug_type_literal,
    narrow_static_prop_from_gettype_literal,
};
pub(crate) use core::{
    apply_prop_narrowed, extract_any_prop_access, extract_class_fqcn_from_expr,
    extract_expr_guard_key, extract_prop_access, extract_static_prop_access, is_numeric_string,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, MatchSubject,
};
use core::{
    extract_class_name, extract_null_coalesce, extract_nullsafe_prop_access, extract_var_name,
    narrow_count_or_strlen_equality, promote_assignment_effects, same_literal, set_narrowed,
    ScalarArgTarget, UnionNarrowExt,
};
use enum_class::{
    extract_enum_value_case, narrow_prop_to_specific_class, narrow_static_prop_to_specific_class,
    narrow_var_to_literal_enum_case, narrow_var_to_specific_class,
};
use instanceof_core::{
    filter_out_instanceof_match, filter_out_is_a_string_match,
    narrow_instanceof_preserving_subtypes, narrow_prop_instanceof, narrow_prop_is_a,
    narrow_prop_is_subclass_of, narrow_static_prop_instanceof, narrow_static_prop_is_a,
    narrow_static_prop_is_subclass_of, narrow_strict_subclass_of, partition_is_a_string_like,
};
pub(crate) use instanceof_disjuncts::{
    narrow_instanceof_disjuncts, narrow_mixed_disjuncts, narrow_mixed_prop_disjuncts,
    narrow_mixed_static_prop_disjuncts, narrow_prop_instanceof_disjuncts,
    narrow_prop_type_fn_disjuncts, narrow_static_prop_instanceof_disjuncts,
    narrow_static_prop_type_fn_disjuncts, narrow_type_fn_disjuncts,
};
use instanceof_disjuncts::{narrow_or_instanceof_true, narrow_or_isset_true};
use literals::{
    atom_safe_for_loose_int_narrowing, bool_narrow_type, expr_is_nonempty_string_literal,
    extract_int_literal, flip_comparison_op, is_truthy_bool_literal, literal_int_narrow_type,
    literal_string_narrow_type, narrow_prop_bool, narrow_prop_int_comparison,
    narrow_prop_literal_int, narrow_prop_literal_string, narrow_prop_loose_bool,
    narrow_prop_loose_int, narrow_prop_loose_null, narrow_static_prop_int_comparison,
    narrow_string_to_non_empty, narrow_var_bool, narrow_var_int_comparison, narrow_var_literal_int,
    narrow_var_literal_string, narrow_var_loose_bool, narrow_var_loose_int, narrow_var_loose_null,
    narrow_var_null,
};
use strings::{
    extract_strlen_arg, extract_strlen_static_prop_arg, narrow_prop_string_strlen_comparison,
    narrow_static_prop_string_strlen_comparison, narrow_string_false_comparable_condition,
    narrow_string_strlen_comparison,
};
use type_fn::{narrow_from_type_fn, narrow_prop_from_type_fn, narrow_static_prop_from_type_fn};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Narrow the types in `ctx` as if `expr` evaluates to `is_true`.
pub fn narrow_from_condition(
    expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    match &expr.kind {
        // Parenthesized — unwrap and narrow the inner expression
        ExprKind::Parenthesized(inner) => {
            narrow_from_condition(inner, ctx, is_true, db, file);
        }

        // !expr  →  narrow as if expr is !is_true
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
            narrow_from_condition(&u.operand, ctx, !is_true, db, file);
        }

        // $a && $b  →  if true: narrow both; if false: no constraint
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanAnd || b.op == BinaryOp::LogicalAnd => {
            if is_true {
                narrow_from_condition(&b.left, ctx, true, db, file);
                narrow_from_condition(&b.right, ctx, true, db, file);
                // When A && B is true, both sides were evaluated.
                // Promote variables from possibly_assigned to assigned for side effects in each.
                promote_assignment_effects(&b.left, ctx, db, file);
                promote_assignment_effects(&b.right, ctx, db, file);
            }
        }

        // $a || $b  →  if false: narrow both; if true: try to narrow same-var instanceof union
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            if !is_true {
                narrow_from_condition(&b.left, ctx, false, db, file);
                narrow_from_condition(&b.right, ctx, false, db, file);
                // When A || B is false, both sides were evaluated.
                // Promote variables from possibly_assigned to assigned for side effects in each.
                promote_assignment_effects(&b.left, ctx, db, file);
                promote_assignment_effects(&b.right, ctx, db, file);
            } else {
                // For `$x instanceof A || $x instanceof B` in true-branch: narrow $x to A|B
                narrow_or_instanceof_true(&b.left, &b.right, ctx, db, file);

                // For `!isset($x) || RHS` in true-branch: narrow RHS as if isset($x) is true
                narrow_or_isset_true(&b.left, &b.right, ctx, db, file);
            }
        }

        // $x === null / $x !== null
        ExprKind::Binary(b) if b.op == BinaryOp::Identical || b.op == BinaryOp::NotIdentical => {
            let is_identical = b.op == BinaryOp::Identical;
            let effective_true = if is_identical { is_true } else { !is_true };

            // `count($arr) === N` / `strlen($str) !== N`, etc. — independent of
            // the rest of this arm, mirrors the `<`/`<=`/`>`/`>=` handling below.
            narrow_count_or_strlen_equality(ctx, db, file, &b.left, &b.right, b.op, is_true);

            // `($x ?? FALLBACK) === FALLBACK` — on the false branch, $x was defined
            // Must be checked before literal comparisons because `b.right` matching a literal
            // would otherwise consume the arm before we check for NullCoalesce on `b.left`.
            if let Some(nc) = extract_null_coalesce(&b.left) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        // A non-FALLBACK result also proves the receiver itself wasn't
                        // null (a null receiver's `?->`/`->` access coalesces straight
                        // to FALLBACK) — without this, a later plain read of the same
                        // property re-admits null via the receiver's own nullability.
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&nc.left, ctx, db, file)
                {
                    // `(self::$prop ?? FALLBACK) === FALLBACK` — static-property
                    // counterpart of the instance-property case above (no receiver).
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                    }
                }
            } else if let Some(nc) = extract_null_coalesce(&b.right) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&nc.left, ctx, db, file)
                {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                    }
                }
            }
            // `$x === null`
            else if matches!(b.right.kind, ExprKind::Null) {
                if let Some(target) = extract_array_key_first_or_last_arg(&b.left) {
                    match target {
                        ScalarArgTarget::Var(arr_var) => {
                            narrow_array_key_first_or_last_null(ctx, &arr_var, effective_true)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_array_key_first_or_last_null(
                                ctx,
                                &obj,
                                &prop,
                                db,
                                file,
                                effective_true,
                            )
                        }
                    }
                } else if let Some((fqcn, prop)) =
                    extract_array_key_first_or_last_static_prop_arg(&b.left, ctx, db, file)
                {
                    narrow_static_prop_array_key_first_or_last_null(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        effective_true,
                    );
                } else if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.left) {
                    narrow_nullsafe_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let ExprKind::NullsafeMethodCall(mc) = &b.left.kind {
                    narrow_nullsafe_method_call_null(ctx, mc, db, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_null(ctx, &fqcn, &prop, db, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(target) = extract_array_key_first_or_last_arg(&b.right) {
                    match target {
                        ScalarArgTarget::Var(arr_var) => {
                            narrow_array_key_first_or_last_null(ctx, &arr_var, effective_true)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_array_key_first_or_last_null(
                                ctx,
                                &obj,
                                &prop,
                                db,
                                file,
                                effective_true,
                            )
                        }
                    }
                } else if let Some((fqcn, prop)) =
                    extract_array_key_first_or_last_static_prop_arg(&b.right, ctx, db, file)
                {
                    narrow_static_prop_array_key_first_or_last_null(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        effective_true,
                    );
                } else if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.right) {
                    narrow_nullsafe_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let ExprKind::NullsafeMethodCall(mc) = &b.right.kind {
                    narrow_nullsafe_method_call_null(ctx, mc, db, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.right) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_null(ctx, &fqcn, &prop, db, effective_true);
                }
            }
            // `$x === true` / `$x === false`
            else if matches!(b.right.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, true, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, true, effective_true);
                } else {
                    // `is_string($x) === true` / `($x instanceof Y) === true` — the
                    // left side is guaranteed to be a real bool (not just truthy),
                    // so narrowing it as a bare condition is exactly as sound as
                    // strict equality requires; re-entering the top-level match
                    // dispatches to whatever arm (FunctionCall, Instanceof, Isset,
                    // ...) already handles that expression shape directly.
                    narrow_from_condition(&b.left, ctx, effective_true, db, file);
                }
            } else if matches!(b.right.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, false, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, false, effective_true);
                } else {
                    // `strpos($h, $n) !== false` / `array_search($n, $h) === false`
                    narrow_from_false_comparable_call(&b.left, ctx, db, file, effective_true);
                    // `is_string($x) === false` — see the `=== true` comment above.
                    narrow_from_condition(&b.left, ctx, !effective_true, db, file);
                }
            }
            // `true === $x` / `false === $x` — symmetric; extract_var_name looks through
            // assignment exprs, so this also handles `false === ($x = expr)`.
            else if matches!(b.left.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, true, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, true, effective_true);
                } else {
                    narrow_from_condition(&b.right, ctx, effective_true, db, file);
                }
            } else if matches!(b.left.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, false, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, false, effective_true);
                } else {
                    narrow_from_false_comparable_call(&b.right, ctx, db, file, effective_true);
                    narrow_from_condition(&b.right, ctx, !effective_true, db, file);
                }
            }
            // `$enum->value === 'H'` narrows $enum to the specific backed-enum
            // case whose ->value equals the literal (and excludes exactly
            // that case on the false branch) — sound because PHP requires
            // distinct backing values across a backed enum's cases, so the
            // value uniquely identifies the case. Checked before the bare
            // literal-string/int arms below, which otherwise shadow it
            // whenever the literal happens to be on the same side they key
            // off of.
            else if let Some((var_name, enum_fqcn, case_name)) =
                extract_enum_value_case(&b.left, &b.right, ctx, db)
                    .or_else(|| extract_enum_value_case(&b.right, &b.left, ctx, db))
            {
                narrow_var_to_literal_enum_case(
                    db,
                    ctx,
                    &var_name,
                    &enum_fqcn,
                    &case_name,
                    effective_true,
                );
            }
            // `get_class($x) === 'ClassName'` — check before literal strings so it takes precedence
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(target) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_class_static_prop_arg(&b.left, ctx, db, file)
                {
                    // `get_class(self::$prop) === 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(target) = extract_gettype_arg(&b.left) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.left) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_gettype_static_prop_arg(&b.left, ctx, db, file)
                {
                    narrow_static_prop_from_gettype_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_get_debug_type_static_prop_arg(&b.left, ctx, db, file)
                {
                    narrow_static_prop_from_get_debug_type_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_parent_class_static_prop_arg(&b.left, ctx, db, file)
                {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.left) {
                    // `$obj::class === 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_dynamic_class_const_static_prop_var(&b.left, ctx, db, file)
                {
                    // `self::$prop::class === 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(name) = extract_var_name(&b.left) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    // `$this->prop === 'literal'`
                    narrow_prop_literal_string(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        class_name_str,
                        effective_true,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    // `self::$prop === 'literal'`
                    narrow_static_prop_literal_string(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        class_name_str,
                        effective_true,
                    );
                }
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(target) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_class_static_prop_arg(&b.right, ctx, db, file)
                {
                    // `'ClassName' === get_class(self::$prop)`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(target) = extract_gettype_arg(&b.right) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.right) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_gettype_static_prop_arg(&b.right, ctx, db, file)
                {
                    narrow_static_prop_from_gettype_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_get_debug_type_static_prop_arg(&b.right, ctx, db, file)
                {
                    narrow_static_prop_from_get_debug_type_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_parent_class_static_prop_arg(&b.right, ctx, db, file)
                {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.right) {
                    // `'ClassName' === $obj::class`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_dynamic_class_const_static_prop_var(&b.right, ctx, db, file)
                {
                    // `'ClassName' === self::$prop::class`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(name) = extract_var_name(&b.right) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    // `'literal' === $this->prop`
                    narrow_prop_literal_string(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        class_name_str,
                        effective_true,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    // `'literal' === self::$prop`
                    narrow_static_prop_literal_string(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        class_name_str,
                        effective_true,
                    );
                }
            }
            // `$x === 42`
            else if let ExprKind::Int(n) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_literal_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_literal_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            } else if let ExprKind::Int(n) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_literal_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_literal_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            }
            // `$arr === []` narrows $arr to empty; `$arr !== []` narrows to non-empty.
            // Checked before the enum-case/class-const guard below: a bare
            // `self::$prop === []` would otherwise match that guard's
            // "either side is a StaticPropertyAccess" condition and be
            // silently swallowed (that function only handles enum-case/
            // get_class-family comparisons, not array-emptiness).
            else if narrow_array_emptiness_condition(
                ctx,
                db,
                file,
                &b.left,
                &b.right,
                effective_true,
            ) {
            }
            // `$x === EnumName::CaseName` / get_class()/get_debug_type()/
            // get_parent_class()/$obj::class compared against `Foo::class` —
            // factored into narrow_from_static_or_class_const_comparison so
            // the loose `==`/`!=` arm can reuse it (see that function's doc
            // comment for why loose comparison is sound here specifically).
            else if matches!(b.right.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.left.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.right.kind, ExprKind::ClassConstAccess(_))
                || matches!(b.left.kind, ExprKind::ClassConstAccess(_))
            {
                narrow_from_static_or_class_const_comparison(ctx, b, effective_true, db, file);
            }
        }

        // $x < N, $x <= N, $x > N, $x >= N — comparison-driven integer range narrowing
        ExprKind::Binary(b)
            if matches!(
                b.op,
                BinaryOp::Less
                    | BinaryOp::LessOrEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterOrEqual
            ) =>
        {
            // `$x < N` / `N < $x` (and `$this->prop < N` / `N < $this->prop`) —
            // normalize so the variable/property is always on the left,
            // flipping the operator when the literal was on the left instead.
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_var_int_comparison(ctx, &var_name, b.op, n, is_true);
                }
            } else if let Some(var_name) = extract_var_name(&b.right) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_var_int_comparison(ctx, &var_name, flip_comparison_op(b.op), n, is_true);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_prop_int_comparison(ctx, &obj, &prop, db, file, b.op, n, is_true);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_prop_int_comparison(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        flip_comparison_op(b.op),
                        n,
                        is_true,
                    );
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.left, ctx, db, file) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_static_prop_int_comparison(ctx, &fqcn, &prop, db, b.op, n, is_true);
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.right, ctx, db, file) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_static_prop_int_comparison(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        flip_comparison_op(b.op),
                        n,
                        is_true,
                    );
                }
            }
            // count($arr) op N  /  N op count($arr)
            narrow_array_count_condition(ctx, db, file, &b.left, &b.right, b.op, is_true);
            // strlen($str) op N  /  N op strlen($str) — same normalization.
            let strlen_call_on_left = extract_strlen_arg(&b.left).is_some()
                || extract_strlen_static_prop_arg(&b.left, ctx, db, file).is_some();
            let (strlen_expr, strlen_cmp_op, strlen_lit) = if strlen_call_on_left {
                (&b.left, b.op, &b.right)
            } else {
                (&b.right, flip_comparison_op(b.op), &b.left)
            };
            if let (Some(target), Some(n)) = (
                extract_strlen_arg(strlen_expr),
                extract_int_literal(strlen_lit),
            ) {
                match target {
                    ScalarArgTarget::Var(str_var) => {
                        narrow_string_strlen_comparison(ctx, &str_var, strlen_cmp_op, n, is_true)
                    }
                    ScalarArgTarget::Prop(obj, prop) => narrow_prop_string_strlen_comparison(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        strlen_cmp_op,
                        n,
                        is_true,
                    ),
                }
            } else if let (Some((fqcn, prop)), Some(n)) = (
                extract_strlen_static_prop_arg(strlen_expr, ctx, db, file),
                extract_int_literal(strlen_lit),
            ) {
                narrow_static_prop_string_strlen_comparison(
                    ctx,
                    &fqcn,
                    &prop,
                    db,
                    strlen_cmp_op,
                    n,
                    is_true,
                );
            }
        }

        // $x == null  (loose equality)
        ExprKind::Binary(b) if b.op == BinaryOp::Equal || b.op == BinaryOp::NotEqual => {
            let is_equal = b.op == BinaryOp::Equal;
            let effective_true = if is_equal { is_true } else { !is_true };

            // `count($arr) == N` / `strlen($str) != N`, etc.
            narrow_count_or_strlen_equality(ctx, db, file, &b.left, &b.right, b.op, is_true);

            // `($x ?? FALLBACK) == FALLBACK` / `!= FALLBACK` — same reasoning as the
            // strict `===`/`!==` arm: `same_literal` requires an identical literal
            // AST, so when $x is null the coalesce result is exactly FALLBACK,
            // making `FALLBACK == FALLBACK` trivially true regardless of loose vs
            // strict — a `!=` result still proves $x wasn't null. Must be checked
            // before the literal-comparison arms below for the same reason as the
            // strict arm (a literal on the other side would otherwise consume it).
            if let Some(nc) = extract_null_coalesce(&b.left) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&nc.left, ctx, db, file)
                {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                    }
                }
            } else if let Some(nc) = extract_null_coalesce(&b.right) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&nc.left, ctx, db, file)
                {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                    }
                }
            } else if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_loose_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_loose_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_loose_null(ctx, &fqcn, &prop, db, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_loose_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_loose_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_loose_null(ctx, &fqcn, &prop, db, effective_true);
                }
            }
            // `$x == true` / `$x == false` (and negated forms) — PHP defines loose
            // comparison to a bool literal as `(bool)$x === value`, i.e. identical
            // to the truthy/falsy narrowing a bare `if ($x)` already gets.
            else if let ExprKind::Bool(value) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_loose_bool(ctx, &name, *value == effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_loose_bool(ctx, &obj, &prop, db, file, *value == effective_true);
                    // `null == true` is false, so a true match against the
                    // literal `true` also proves the receiver wasn't null —
                    // same reasoning as the strict `===` sibling arm above.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, *value == effective_true);
                } else {
                    if !value {
                        // `strpos($h, $n) == false` / `!= false` — same
                        // false-comparable narrowing as the strict `===`/`!==` arm.
                        narrow_from_false_comparable_call(&b.left, ctx, db, file, effective_true);
                    }
                    // `is_string($x) == true` / `== false` — loose comparison to a
                    // bool literal is `(bool)$x === value` for ANY $x (not just a
                    // guaranteed-bool expression), i.e. exactly the bare truthy/
                    // falsy narrowing `narrow_from_condition` itself provides.
                    narrow_from_condition(&b.left, ctx, *value == effective_true, db, file);
                }
            } else if let ExprKind::Bool(value) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_loose_bool(ctx, &name, *value == effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_loose_bool(ctx, &obj, &prop, db, file, *value == effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, *value == effective_true);
                } else {
                    if !value {
                        narrow_from_false_comparable_call(&b.right, ctx, db, file, effective_true);
                    }
                    narrow_from_condition(&b.right, ctx, *value == effective_true, db, file);
                }
            }
            // `$arr == []` / `$arr != []` — loose array equality requires identical
            // key/value pairs, so this is exactly as sound as the strict `===` case.
            else if narrow_array_emptiness_condition(
                ctx,
                db,
                file,
                &b.left,
                &b.right,
                effective_true,
            ) {
            }
            // `get_class($x) == 'ClassName'` / `get_debug_type($x) == 'ClassName'` /
            // `gettype($x) == 'ClassName'` / `$x::class == 'ClassName'` — loose `==`
            // mirrors the `===` handling below the `Identical`/`NotIdentical` arm:
            // class/type names are never numeric-looking strings, so loose
            // comparison agrees with strict comparison here.
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(target) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_class_static_prop_arg(&b.left, ctx, db, file)
                {
                    // `get_class(self::$prop) == 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(target) = extract_gettype_arg(&b.left) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.left) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_gettype_static_prop_arg(&b.left, ctx, db, file)
                {
                    narrow_static_prop_from_gettype_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_get_debug_type_static_prop_arg(&b.left, ctx, db, file)
                {
                    narrow_static_prop_from_get_debug_type_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_parent_class_static_prop_arg(&b.left, ctx, db, file)
                {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_dynamic_class_const_static_prop_var(&b.left, ctx, db, file)
                {
                    // `self::$prop::class == 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(target) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_class_static_prop_arg(&b.right, ctx, db, file)
                {
                    // `'ClassName' == get_class(self::$prop)`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                } else if let Some(target) = extract_gettype_arg(&b.right) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.right) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_gettype_static_prop_arg(&b.right, ctx, db, file)
                {
                    narrow_static_prop_from_gettype_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                    );
                } else if let Some((fqcn, prop)) =
                    extract_get_debug_type_static_prop_arg(&b.right, ctx, db, file)
                {
                    narrow_static_prop_from_get_debug_type_literal(
                        ctx,
                        &fqcn,
                        &prop,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some((fqcn_recv, prop)) =
                    extract_get_parent_class_static_prop_arg(&b.right, ctx, db, file)
                {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                } else if let Some((fqcn_recv, prop)) =
                    extract_dynamic_class_const_static_prop_var(&b.right, ctx, db, file)
                {
                    // `'ClassName' == self::$prop::class`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
            // `$x == 42` / `$x != 42` — sound only when every current atom is
            // already int-like (no string/float/bool atom could loosely equal
            // the same numeric value differently than strict comparison
            // would), reusing the strict `===` arm's literal_int_narrow_type
            // once that holds.
            else if let ExprKind::Int(n) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_loose_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_loose_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_loose_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            } else if let ExprKind::Int(n) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_loose_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_loose_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_loose_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            }
            // `$x == EnumName::CaseName` / `get_class($x) == Foo::class` etc. —
            // the `Foo::class`/enum-case counterpart of the string-literal arms
            // above, reusing the same logic the strict `===` arm uses (sound for
            // loose comparison — see narrow_from_static_or_class_const_comparison's
            // doc comment for why).
            else if matches!(b.right.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.left.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.right.kind, ExprKind::ClassConstAccess(_))
                || matches!(b.left.kind, ExprKind::ClassConstAccess(_))
            {
                narrow_from_static_or_class_const_comparison(ctx, b, effective_true, db, file);
            }
        }

        // $x instanceof ClassName  /  $this->prop instanceof ClassName
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    // Resolve the short name to its FQCN using file imports
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    let current = ctx.get_var(&var_name);
                    let narrowed = if is_true {
                        narrow_instanceof_preserving_subtypes(
                            &current,
                            &class_name,
                            db,
                            &ctx.template_param_names,
                        )
                    } else {
                        filter_out_instanceof_match(&current, &class_name, db)
                    };
                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                }
            } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_prop_instanceof(ctx, &obj, &prop, &class_name, db, file, is_true);
                    // `null instanceof X` is always false, so a true result also
                    // proves the receiver itself is non-null (see
                    // `narrow_nullsafe_prop_null` for the same reasoning).
                    if is_true {
                        narrow_var_null(ctx, &obj, false);
                    }
                }
            } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_prop_instanceof(ctx, &obj, &prop, &class_name, db, file, is_true);
                    // Same reasoning as the nullsafe arm above: `null instanceof X`
                    // is always false, so proving it true also proves `$obj` itself
                    // wasn't null (PHP 8 reads `$obj->prop` on a null `$obj` as a
                    // warning, still evaluating to null).
                    if is_true {
                        narrow_var_null(ctx, &obj, false);
                    }
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.left, ctx, db, file) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_static_prop_instanceof(ctx, &fqcn, &prop, &class_name, db, is_true);
                }
            }
        }

        // is_string($x), is_int($x), is_null($x), is_array($x), etc.
        // Also handles assert($x instanceof Y) — narrows like a bare condition.
        ExprKind::FunctionCall(call) => {
            // `ExprKind::Variable` (a dynamic `$fn()` call) is deliberately not
            // matched here: `name` would be the callable variable's own
            // identifier text, not its runtime string value, so every builtin
            // check below would only ever match by coincidence (e.g. a
            // variable literally named `$is_array`) while never actually
            // calling that builtin.
            let fn_name_opt: Option<&str> = match &call.name.kind {
                ExprKind::Identifier(name) => Some(name.as_ref()),
                _ => None,
            };
            if let Some(fn_name) = fn_name_opt {
                let bare = fn_name.trim_start_matches('\\');
                if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "class_exists" | "interface_exists" | "trait_exists" | "enum_exists"
                ) {
                    // `if (class_exists(\Foo\Bar::class)) { ... }` — record \Foo\Bar as
                    // proven-to-exist in the true branch so that UndefinedClass is
                    // suppressed for all usages within the guarded block.
                    // Variable form: `if (class_exists($var)) { ... }` — narrow $var to
                    // class-string so it satisfies class-string-typed parameters.
                    // `interface_exists($var)` narrows to the more precise interface-string.
                    // `enum_exists($var)`/`trait_exists($var)` narrow like class_exists —
                    // no dedicated enum-string/trait-string atomic exists.
                    if is_true {
                        if let Some(arg_expr) = call.args.first() {
                            if let Some(fqcn) = extract_class_fqcn_from_expr(
                                &arg_expr.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                ctx.class_exists_guards.insert(fqcn);
                            } else if let Some(var_name) = extract_var_name(&arg_expr.value) {
                                let current = ctx.get_var(&var_name);
                                let narrowed = if bare.eq_ignore_ascii_case("interface_exists") {
                                    current.narrow_to_interface_string()
                                } else {
                                    current.narrow_to_class_string()
                                };
                                set_narrowed(ctx, &var_name, &current, narrowed, true);
                            } else if let Some((obj, prop)) =
                                extract_any_prop_access(&arg_expr.value)
                            {
                                let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                if !current.is_mixed() {
                                    let narrowed = if bare.eq_ignore_ascii_case("interface_exists")
                                    {
                                        current.narrow_to_interface_string()
                                    } else {
                                        current.narrow_to_class_string()
                                    };
                                    apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, true);
                                    // `class_exists(null)` etc. can never be true, so a
                                    // true result also proves `$obj` itself wasn't null.
                                    narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
                                }
                            } else if let Some((fqcn_recv, prop)) =
                                extract_static_prop_access(&arg_expr.value, ctx, db, file)
                            {
                                let current =
                                    resolve_static_prop_current_type(ctx, &fqcn_recv, &prop, db);
                                if !current.is_mixed() {
                                    let narrowed = if bare.eq_ignore_ascii_case("interface_exists")
                                    {
                                        current.narrow_to_interface_string()
                                    } else {
                                        current.narrow_to_class_string()
                                    };
                                    apply_prop_narrowed(
                                        ctx, &fqcn_recv, &prop, current, narrowed, true,
                                    );
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("defined") {
                    // `if (defined('NAME')) { ... NAME ... }` — record NAME as
                    // proven-defined in the true branch to suppress
                    // UndefinedConstant within the guarded block.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(name) = &arg.value.kind {
                                let name = name.as_ref().trim_start_matches('\\');
                                if !name.is_empty() {
                                    ctx.defined_guards.insert(std::sync::Arc::from(name));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("function_exists") {
                    // `if (function_exists('fn')) { ... fn() ... }` — record fn
                    // as proven-to-exist in the true branch to suppress
                    // UndefinedFunction within the guarded block. Combined with
                    // negation + divergence (`if (!function_exists('fn')) throw;`)
                    // this also covers the early-exit pattern.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(name) = &arg.value.kind {
                                let name = name.as_ref().trim_start_matches('\\');
                                if !name.is_empty() {
                                    ctx.function_exists_guards
                                        .insert(std::sync::Arc::from(name));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("extension_loaded") {
                    // `if (extension_loaded('ext')) { ... }` — record the extension
                    // name so that `UndefinedClass` is suppressed for any class used
                    // inside the guarded block (the caller verified the extension is
                    // present). The false-branch / early-exit pattern also works via
                    // the normal divergence+narrowing mechanism.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(ext) = &arg.value.kind {
                                if !ext.is_empty() {
                                    ctx.extension_loaded_guards
                                        .insert(std::sync::Arc::from(ext.as_ref()));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("assert") {
                    // assert($condition) — narrow as if the condition is is_true
                    if let Some(arg_expr) = call.args.first() {
                        narrow_from_condition(&arg_expr.value, ctx, is_true, db, file);
                    }
                } else if bare.eq_ignore_ascii_case("method_exists")
                    || bare.eq_ignore_ascii_case("property_exists")
                {
                    // Narrow the first arg to TObject for simple variables (existing behaviour).
                    // Additionally record `(expr_key, method_name)` in method_exists_guards for
                    // property-access first args where variable narrowing can't reach — but only
                    // for method_exists(): properties and methods are independent namespaces in
                    // PHP, so property_exists($obj, 'foo') proves nothing about a method 'foo'.
                    if let Some(arg_expr) = call.args.first() {
                        if let Some(var_name) = extract_var_name(&arg_expr.value) {
                            narrow_from_type_fn(ctx, bare, &var_name, db, is_true);
                        } else if let Some((obj, prop)) = extract_any_prop_access(&arg_expr.value) {
                            narrow_prop_from_type_fn(ctx, bare, &obj, &prop, db, file, is_true);
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&arg_expr.value, ctx, db, file)
                        {
                            narrow_static_prop_from_type_fn(ctx, bare, &fqcn, &prop, db, is_true);
                        }
                        if is_true && bare.eq_ignore_ascii_case("method_exists") {
                            if let Some(expr_key) =
                                extract_expr_guard_key(&arg_expr.value, ctx, db, file)
                            {
                                if let Some(method_arg) = call.args.get(1) {
                                    if let ExprKind::String(method_name) = &method_arg.value.kind {
                                        let method_lc = std::sync::Arc::from(
                                            crate::util::php_ident_lowercase(method_name).as_str(),
                                        );
                                        ctx.method_exists_guards.insert((expr_key, method_lc));
                                    }
                                }
                            }
                        }
                    }
                } else if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "array_key_exists" | "key_exists"
                ) {
                    // `key_exists()` is a built-in alias of `array_key_exists()`
                    // with identical semantics.
                    narrow_array_key_exists_condition(ctx, call, is_true, db, file);
                } else if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "str_contains" | "str_starts_with" | "str_ends_with"
                ) {
                    // str_contains($haystack, 'x') true → $haystack is non-empty-string
                    // (when the needle is a non-empty literal — a non-empty needle can
                    // only be found in a non-empty haystack).
                    if is_true {
                        if let (Some(haystack_arg), Some(needle_arg)) =
                            (call.args.first(), call.args.get(1))
                        {
                            let needle_non_empty =
                                expr_is_nonempty_string_literal(&needle_arg.value, ctx, db, file);
                            if needle_non_empty {
                                match ScalarArgTarget::extract(&haystack_arg.value) {
                                    Some(ScalarArgTarget::Var(var_name)) => {
                                        let current = ctx.get_var(&var_name);
                                        if !current.is_mixed() {
                                            let narrowed = narrow_string_to_non_empty(&current);
                                            if narrowed != current {
                                                ctx.set_var(&var_name, narrowed);
                                            }
                                        }
                                    }
                                    Some(ScalarArgTarget::Prop(obj, prop)) => {
                                        let current =
                                            resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                        if !current.is_mixed() {
                                            let narrowed = narrow_string_to_non_empty(&current);
                                            apply_prop_narrowed(
                                                ctx, &obj, &prop, current, narrowed, false,
                                            );
                                        }
                                        // A non-empty needle can't be found in a null
                                        // receiver's coerced-to-"" property.
                                        narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
                                    }
                                    None => {
                                        // ScalarArgTarget has no static-property
                                        // variant (tracked as S19) — extract it
                                        // call-site-locally instead, mirroring
                                        // gettype()/count()/strlen() on a static prop.
                                        if let Some((fqcn, prop)) = extract_static_prop_access(
                                            &haystack_arg.value,
                                            ctx,
                                            db,
                                            file,
                                        ) {
                                            let current = resolve_static_prop_current_type(
                                                ctx, &fqcn, &prop, db,
                                            );
                                            if !current.is_mixed() {
                                                let narrowed = narrow_string_to_non_empty(&current);
                                                apply_prop_narrowed(
                                                    ctx, &fqcn, &prop, current, narrowed, false,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("in_array") {
                    narrow_in_array_condition(ctx, call, is_true, db, file);
                } else if bare.eq_ignore_ascii_case("is_a") {
                    // is_a($obj, 'ClassName') → instanceof semantics (includes exact class).
                    // When $allow_string (3rd arg) is truthy, the first arg may be a class-string
                    // — preserve string/class-string atoms so the true branch stays reachable
                    // and no false diverge is set in the false branch.
                    if let (Some(obj_arg), Some(class_arg)) = (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                let current = ctx.get_var(&var_name);
                                if allow_string {
                                    // When allow_string is true, string/class-string atoms are
                                    // valid is_a() true-branch values — preserve them so the
                                    // true branch stays reachable and type is not wrongly erased.
                                    let narrowed = if is_true {
                                        // Partition into string-like (kept only when consistent
                                        // with class_name) and object-like (narrowed via
                                        // instanceof) so `narrow_instanceof_preserving_subtypes`
                                        // fallback doesn't inject a spurious named-object atom
                                        // when the current type is purely string/class-string.
                                        let (mut result, obj_part) =
                                            partition_is_a_string_like(&current, &class_name, db);
                                        if !obj_part.is_empty() || current.is_mixed() {
                                            let obj_src = if obj_part.is_empty() {
                                                &current
                                            } else {
                                                &obj_part
                                            };
                                            let obj_narrowed =
                                                narrow_instanceof_preserving_subtypes(
                                                    obj_src,
                                                    &class_name,
                                                    db,
                                                    &ctx.template_param_names,
                                                );
                                            for atom in obj_narrowed.types.iter() {
                                                result.add_type(atom.clone());
                                            }
                                        }
                                        result
                                    } else {
                                        filter_out_is_a_string_match(&current, &class_name, db)
                                    };
                                    // Don't mark diverges when allow_string is set: a
                                    // class-string variable may still be a valid non-object
                                    // value that passes the test.
                                    set_narrowed(ctx, &var_name, &current, narrowed, false);
                                } else {
                                    let narrowed = if is_true {
                                        narrow_instanceof_preserving_subtypes(
                                            &current,
                                            &class_name,
                                            db,
                                            &ctx.template_param_names,
                                        )
                                    } else {
                                        filter_out_instanceof_match(&current, &class_name, db)
                                    };
                                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                                }
                            }
                        } else if let Some((obj, prop)) = extract_any_prop_access(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                narrow_prop_is_a(
                                    ctx,
                                    &obj,
                                    &prop,
                                    &class_name,
                                    allow_string,
                                    db,
                                    file,
                                    is_true,
                                );
                                narrow_receiver_non_null_on_prop_match(ctx, &obj, is_true);
                            }
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&obj_arg.value, ctx, db, file)
                        {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                narrow_static_prop_is_a(
                                    ctx,
                                    &fqcn,
                                    &prop,
                                    &class_name,
                                    allow_string,
                                    db,
                                    is_true,
                                );
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("is_subclass_of") {
                    // is_subclass_of($obj, 'ClassName') → strict-subclass check: the exact
                    // class itself is NOT a subclass of itself.
                    // True branch: keep only atoms that are known strict subclasses.
                    // False branch: no narrowing — is_subclass_of being false doesn't tell us
                    // the variable isn't Foo; Foo is not a subclass of itself, so Foo atoms
                    // must be kept (removing them would make a later `Foo` use diverge).
                    if let (Some(obj_arg), Some(class_arg)) = (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let current = ctx.get_var(&var_name);
                                if is_true {
                                    let narrowed = narrow_strict_subclass_of(
                                        &current,
                                        &class_name,
                                        db,
                                        &ctx.template_param_names,
                                    );
                                    // mark_diverges=false: the exact class being absent from
                                    // strict-subclass narrowing doesn't make the branch dead.
                                    set_narrowed(ctx, &var_name, &current, narrowed, false);
                                }
                                // False branch: leave current type unchanged.
                            }
                        } else if let Some((obj, prop)) = extract_any_prop_access(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                narrow_prop_is_subclass_of(
                                    ctx,
                                    &obj,
                                    &prop,
                                    &class_name,
                                    db,
                                    file,
                                    is_true,
                                );
                                narrow_receiver_non_null_on_prop_match(ctx, &obj, is_true);
                            }
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&obj_arg.value, ctx, db, file)
                        {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                narrow_static_prop_is_subclass_of(
                                    ctx,
                                    &fqcn,
                                    &prop,
                                    &class_name,
                                    db,
                                    is_true,
                                );
                            }
                        }
                    }
                } else if apply_docblock_assertions(call, ctx, is_true, db, file, fn_name) {
                    // User-defined assertion applied.
                } else if let Some(arg_expr) = call.args.first() {
                    if let Some(var_name) = extract_var_name(&arg_expr.value) {
                        narrow_from_type_fn(ctx, bare, &var_name, db, is_true);
                    } else if let Some((obj, prop)) = extract_any_prop_access(&arg_expr.value) {
                        narrow_prop_from_type_fn(ctx, bare, &obj, &prop, db, file, is_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&arg_expr.value, ctx, db, file)
                    {
                        narrow_static_prop_from_type_fn(ctx, bare, &fqcn, &prop, db, is_true);
                    }
                }
            }
        }

        // $obj->isFoo($x) with @psalm-assert-if-true/-if-false on isFoo() —
        // method-call counterpart of the FunctionCall arm above (which only
        // ever resolved a free function via `find_function`).
        ExprKind::MethodCall(mc) => {
            if let Some(fqcn) = method_call_receiver_fqcn(&mc.object, ctx, db, file) {
                if let ExprKind::Identifier(name) = &mc.method.kind {
                    let method_name_lower = crate::util::php_ident_lowercase(name);
                    if let Some(resolved) =
                        crate::call::method::resolve_method_from_db(db, &fqcn, &method_name_lower)
                    {
                        apply_method_docblock_assertions(
                            &mc.args, &resolved, ctx, is_true, db, file,
                        );
                    }
                }
            }
        }

        // Foo::isFoo($x) / self::isFoo($x) with @psalm-assert-if-true/-if-false
        // — static-call counterpart of the FunctionCall arm above.
        ExprKind::StaticMethodCall(smc) => {
            if let Some(fqcn) = resolve_static_call_class_fqcn(&smc.class, ctx, db, file) {
                if let ExprKind::Identifier(name) = &smc.method.kind {
                    let method_name_lower = crate::util::php_ident_lowercase(name);
                    if let Some(resolved) =
                        crate::call::method::resolve_method_from_db(db, &fqcn, &method_name_lower)
                    {
                        apply_method_docblock_assertions(
                            &smc.args, &resolved, ctx, is_true, db, file,
                        );
                    }
                }
            }
        }

        // isset($x)
        ExprKind::Isset(vars) => {
            for var_expr in vars.iter() {
                if let Some(var_name) = extract_var_name(var_expr) {
                    if is_true {
                        // remove null; mark as definitely assigned
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                        std::sync::Arc::make_mut(&mut ctx.assigned_vars)
                            .insert(mir_types::Name::from(var_name.as_str()));
                    } else if ctx.var_is_defined(&var_name) {
                        // isset($x) is false and $x is always assigned (e.g. a
                        // parameter) → the only other way isset() can be false is
                        // $x being null.
                        narrow_var_null(ctx, &var_name, true);
                    }
                } else if is_true {
                    // `isset($base[$k])` implies `$base` is a non-null, indexable
                    // value — remove null/false from the base (variable or
                    // property receiver) so a guarded access (`preg_split()`
                    // returns array|false) does not report PossiblyInvalidArrayAccess.
                    if let Some(target) = array_access_base_target(var_expr, ctx, db, file) {
                        narrow_container_non_null_non_false(ctx, &target, db, file);
                    }
                    // For a single-level `isset($arr['key'])` on a shape-typed
                    // base, also narrow that key's OWN value type: remove null
                    // and mark it no longer optional, so a later `$arr['key']`
                    // read inside the guard isn't reported as possibly-null (the
                    // isset check just proved the key is present and non-null).
                    narrow_isset_shape_key(var_expr, ctx, db, file);
                    // `isset($this->prop)` implies the property is non-null too
                    // — the property-receiver counterpart of the bare-variable
                    // case above, since `isset()` is false for both an unset
                    // and a null-valued property.
                    if let Some((obj_var, prop)) = extract_any_prop_access(var_expr) {
                        let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
                        if !current.is_mixed() {
                            let narrowed = current.remove_null();
                            apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
                        }
                        // isset() is only true when the receiver was non-null too
                        // (a null receiver's ->/?-> access is itself unset/null).
                        narrow_receiver_non_null_on_prop_match(ctx, &obj_var, true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(var_expr, ctx, db, file)
                    {
                        // `isset(self::$prop)` — static-property counterpart of the
                        // instance-property case above.
                        let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                        if !current.is_mixed() {
                            let narrowed = current.remove_null();
                            apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, true);
                        }
                    }
                } else if !is_true {
                    // `!isset($base['key'])` on a single-level shape-typed base:
                    // exclude union members where the key is provably present
                    // and non-null (isset() would have been true there).
                    narrow_isset_shape_key_false(var_expr, ctx, db, file);
                }
            }
        }

        // empty($x)
        ExprKind::Empty(var_expr) => {
            if let Some(var_name) = extract_var_name(var_expr) {
                let current = ctx.get_var(&var_name);
                let narrowed = if is_true {
                    // empty($x) is true: x is falsy
                    current.narrow_to_falsy()
                } else {
                    // empty($x) is false: x is truthy
                    current.narrow_to_truthy()
                };
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                }
            } else {
                if !is_true {
                    // `!empty($base[$k])` implies `$base` is a non-null, indexable
                    // value, same as the `isset($base[$k])` case above.
                    if let Some(target) = array_access_base_target(var_expr, ctx, db, file) {
                        narrow_container_non_null_non_false(ctx, &target, db, file);
                    }
                }
                // For a single-level `empty($arr['key'])` on a shape-typed base,
                // also narrow that key's OWN value type by truthiness, mirroring
                // narrow_isset_shape_key.
                narrow_empty_shape_key(var_expr, ctx, is_true, db, file);
                // `empty($this->prop)` — property-receiver counterpart of the
                // bare-variable truthy/falsy case, mirroring the `if ($this->prop)`
                // arm below (narrow_prop_loose_bool). `empty()` inverts truthiness:
                // is_true means the property is falsy.
                if let Some((obj_var, prop)) = extract_any_prop_access(var_expr) {
                    narrow_prop_loose_bool(ctx, &obj_var, &prop, db, file, !is_true);
                    // `empty()` is false (the property is truthy) only when the
                    // receiver was non-null too — a null receiver's ->/?->
                    // access is itself falsy, so `empty()` would be true.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj_var, !is_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(var_expr, ctx, db, file)
                {
                    // `empty(self::$prop)` — static-property counterpart of the
                    // instance-property case above.
                    narrow_static_prop_loose_bool(ctx, &fqcn, &prop, db, !is_true);
                }
            }
        }

        // ($x = expr) / ($x ??= expr) used as a condition
        // The assignment has already been evaluated (ctx holds the post-assignment type).
        // Narrow the target variable based on the truthiness of the expression result.
        ExprKind::Assign(a) if matches!(a.op, AssignOp::Assign | AssignOp::Coalesce) => {
            if let Some(var_name) = extract_var_name(&a.target) {
                let current = ctx.get_var(&var_name);
                let mut narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                // In the true-branch the assignment definitely executed, so
                // the variable is always defined here — clear possibly_undefined.
                if is_true {
                    narrowed.possibly_undefined = false;
                }
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                } else if !current.is_empty() && !current.is_mixed() {
                    ctx.diverges = true;
                }
            } else if let Some((obj_var, prop)) = extract_prop_access(&a.target) {
                // `if ($this->prop = expr)` — property-access counterpart of
                // the plain-variable case above. Reaching the assignment at
                // all (regardless of branch) already proves $obj_var was
                // non-null: PHP fatals assigning a property on a null
                // receiver, unlike a plain read.
                narrow_var_null(ctx, &obj_var, false);
                let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
                let mut narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if is_true {
                    narrowed.possibly_undefined = false;
                }
                apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&a.target, ctx, db, file)
            {
                // `if (self::$prop = expr)` — static-property counterpart of
                // the instance-property case above (no receiver to narrow).
                let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                let mut narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if is_true {
                    narrowed.possibly_undefined = false;
                }
                apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, true);
            }
        }

        // if ($x)  — truthy/falsy narrowing
        _ => {
            if let Some(var_name) = extract_var_name(expr) {
                let current = ctx.get_var(&var_name);
                let narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                } else if !current.is_empty()
                    && !current.is_mixed()
                    && ctx.var_is_defined(&var_name)
                {
                    // The variable's type can never satisfy this truthiness
                    // constraint → this branch is statically unreachable.
                    // Possibly-undefined variables are exempt: an unset
                    // variable reads as null (falsy), so the branch stays
                    // reachable at runtime.
                    ctx.diverges = true;
                }
            } else if let Some((obj_var, prop)) = extract_any_prop_access(expr) {
                // `if ($this->prop)` — property-receiver counterpart of the
                // bare-variable truthy/falsy case above.
                narrow_prop_loose_bool(ctx, &obj_var, &prop, db, file, is_true);
                // Truthy also proves the receiver was non-null (a null
                // receiver's ->/?-> access is itself falsy).
                narrow_receiver_non_null_on_prop_match(ctx, &obj_var, is_true);
            } else if let Some((fqcn, prop)) = extract_static_prop_access(expr, ctx, db, file) {
                // `if (self::$prop)` — static-property counterpart of the
                // instance-property case above.
                narrow_static_prop_loose_bool(ctx, &fqcn, &prop, db, is_true);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Narrow from a call compared against the `false` literal — the idiomatic
/// way to interpret `strpos()`/`array_search()`'s `int|string|false` result,
/// since a loose truthy check misfires on a match at offset/key 0. `is_false`
/// is whether `expr === false` holds in this branch (so `!is_false` means
/// the call proved a match — a substring was found, or the needle is
/// present in the haystack). The strpos()/stripos()-family arm (string
/// domain) is split out into `strings::narrow_string_false_comparable_condition`;
/// the `array_search()` arm below stays here (array domain).
fn narrow_from_false_comparable_call(
    expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    is_false: bool,
) {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return;
    };
    let fn_name_opt: Option<&str> = match &call.name.kind {
        ExprKind::Identifier(name) => Some(name.as_ref()),
        _ => None,
    };
    let Some(fn_name) = fn_name_opt else {
        return;
    };
    let bare = fn_name.trim_start_matches('\\');
    if matches!(
        bare.to_ascii_lowercase().as_str(),
        "strpos"
            | "stripos"
            | "strrpos"
            | "strripos"
            | "mb_strpos"
            | "mb_stripos"
            | "mb_strrpos"
            | "mb_strripos"
    ) {
        narrow_string_false_comparable_condition(call, ctx, db, file, is_false);
    } else if bare.eq_ignore_ascii_case("array_search") {
        // array_search($needle, $haystack) !== false / === false — same
        // haystack-literal narrowing as in_array(), keyed off the same
        // strict/loose-safety rule (see in_array_loose_narrowing_is_safe).
        let strict = call
            .args
            .get(2)
            .map(|a| is_truthy_bool_literal(&a.value))
            .unwrap_or(false);
        if let (Some(needle_arg), Some(haystack_arg)) = (call.args.first(), call.args.get(1)) {
            strip_haystack_null(&haystack_arg.value, ctx, db, file);
            if let Some(target) = ScalarArgTarget::extract(&needle_arg.value) {
                if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx, db, file)
                {
                    if !is_false {
                        if let ScalarArgTarget::Prop(obj, _) = &target {
                            // array_search(null, $haystack) only matches loosely when the
                            // haystack contains a falsy literal (0, "", "0"); mirrors
                            // in_array()'s identical reasoning.
                            if strict || !haystack_admits_null_loosely(&haystack_ty) {
                                narrow_receiver_non_null_on_prop_match(ctx, obj, true);
                            }
                        }
                    }
                    let current = match &target {
                        ScalarArgTarget::Var(name) => ctx.get_var(name),
                        ScalarArgTarget::Prop(obj, prop) => {
                            resolve_prop_current_type(ctx, obj, prop, db, file)
                        }
                    };
                    let loose_safe =
                        strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                    if !current.is_mixed() && loose_safe {
                        let narrowed = if !is_false {
                            // Found: keep only atoms that could match a haystack value.
                            let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                            (!narrowed.is_empty() && narrowed != current).then_some(narrowed)
                        } else {
                            // Not found: safe only when current is a finite literal
                            // union — remove the matched haystack values.
                            let all_literals = !current.types.is_empty()
                                && current.types.iter().all(|a| {
                                    matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                                });
                            all_literals
                                .then(|| {
                                    current.filter(|a| !haystack_ty.types.iter().any(|h| h == a))
                                })
                                .filter(|narrowed| !narrowed.is_empty() && *narrowed != current)
                        };
                        if let Some(narrowed) = narrowed {
                            match target {
                                ScalarArgTarget::Var(name) => ctx.set_var(&name, narrowed),
                                ScalarArgTarget::Prop(obj, prop) => {
                                    apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false)
                                }
                            }
                        }
                    }
                }
            } else if let Some((fqcn, prop)) =
                extract_static_prop_access(&needle_arg.value, ctx, db, file)
            {
                // ScalarArgTarget has no static-property variant (tracked as S19) —
                // extract it call-site-locally instead, mirroring the
                // str_contains-family recipe (no receiver to narrow non-null).
                if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx, db, file)
                {
                    let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                    let loose_safe =
                        strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                    if !current.is_mixed() && loose_safe {
                        let narrowed = if !is_false {
                            let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                            (!narrowed.is_empty() && narrowed != current).then_some(narrowed)
                        } else {
                            let all_literals = !current.types.is_empty()
                                && current.types.iter().all(|a| {
                                    matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                                });
                            all_literals
                                .then(|| {
                                    current.filter(|a| !haystack_ty.types.iter().any(|h| h == a))
                                })
                                .filter(|narrowed| !narrowed.is_empty() && *narrowed != current)
                        };
                        if let Some(narrowed) = narrowed {
                            apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                        }
                    }
                }
            }
        }
    }
}

/// Narrow a property access `$obj->prop` by a null check.
/// PHP 8 reads a plain `->` access on a null receiver as a warning, not a
/// fatal error, evaluating to `null` just like `?->` would — so when the
/// receiver's type actually admits null, `$obj->prop` carries the exact same
/// null-source ambiguity as the nullsafe form and is narrowed identically
/// (see `narrow_nullsafe_prop_null`). When the receiver's type provably
/// excludes null, that ambiguity doesn't exist and a property-null
/// contradiction is a genuine contradiction, so divergence is still marked.
fn narrow_prop_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    if !ctx.get_var(obj_var).is_nullable() {
        narrow_prop_null_with_divergence(ctx, obj_var, prop, db, file, is_null, true);
        return;
    }
    narrow_nullsafe_prop_null(ctx, obj_var, prop, db, file, is_null);
}

fn narrow_prop_null_with_divergence(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
    mark_diverges: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Narrow a nullsafe property access (`$obj?->prop`) by a null check. Also
/// used by the plain (`->`) form via `narrow_prop_null` — both read as
/// `null` when EITHER the receiver is null OR the property's own value is
/// null, so:
/// - the `is_null=true` direction must never mark divergence: proving the
///   property's own declared type excludes null doesn't rule out the
///   receiver-null source.
/// - the `is_null=false` direction additionally proves the receiver itself
///   is non-null, since a null receiver could only ever produce `null` here.
fn narrow_nullsafe_prop_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    narrow_prop_null_with_divergence(ctx, obj_var, prop, db, file, is_null, !is_null);
    if !is_null {
        narrow_var_null(ctx, obj_var, false);
    }
}

/// Narrow a nullsafe METHOD call (`$obj?->method()`) by a null check on its
/// result. Unlike a nullsafe property access, a method call result has no
/// standalone target to narrow later — the only provable fact is the
/// receiver's own nullability, and only when the method's declared return
/// type provably excludes null (a return type that could itself be null
/// carries the same receiver-vs-own-value ambiguity `narrow_nullsafe_prop_null`
/// documents for properties, so bail out then). When it does exclude null,
/// the deduction is actually stronger than the property case: since the
/// method can never contribute a null itself, `$obj?->method() === null`
/// holds if and only if `$obj` was null — both directions narrow `$obj`, not
/// just the non-null direction. Only handles a receiver resolved to a single
/// concrete class and a non-generic return type; anything else is left
/// unnarrowed rather than risk an unsound deduction.
fn narrow_nullsafe_method_call_null(
    ctx: &mut FlowState,
    mc: &php_ast::owned::MethodCallExpr,
    db: &dyn MirDatabase,
    is_null: bool,
) {
    let Some(obj_var) = extract_var_name(&mc.object) else {
        return;
    };
    let ExprKind::Identifier(method_name) = &mc.method.kind else {
        return;
    };
    let obj_ty = ctx.get_var(&obj_var);
    // Only the receiver's non-null atoms matter here — `?Bar $bar` has
    // `Bar|null`, and the whole point is deducing $bar's nullability, not
    // requiring it to already be known non-null.
    let non_null_atoms: Vec<&Atomic> = obj_ty
        .types
        .iter()
        .filter(|t| !matches!(t, Atomic::TNull))
        .collect();
    let fqcn = match non_null_atoms.as_slice() {
        [Atomic::TNamedObject { fqcn, .. }] => *fqcn,
        _ => return,
    };
    let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
    let Some((_, method)) = crate::db::find_method_in_chain(db, here, method_name.as_ref()) else {
        return;
    };
    let Some(return_ty) = method.return_type.as_deref() else {
        return;
    };
    if return_ty.is_mixed()
        || return_ty.contains(|t| matches!(t, Atomic::TNull | Atomic::TTemplateParam { .. }))
    {
        return;
    }
    narrow_var_null(ctx, &obj_var, is_null);
}

fn narrow_static_prop_null(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_null: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Loose-null counterpart of `narrow_static_prop_null`, for
/// `self::$prop == null` / `!= null` (and `static::$prop`/`Class::$prop`).
/// Loose semantics differ from strict: `== null` is also true for any other
/// falsy value (not just `null` itself), so the true direction only narrows
/// to falsy — same reasoning as `narrow_var_loose_null`/`narrow_prop_loose_null`.
/// The false direction stays exactly as sound as the strict form.
fn narrow_static_prop_loose_null(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_null: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_falsy()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Static-property counterpart of `narrow_prop_literal_string`, for
/// `self::$prop === 'literal'` / `static::$prop === 'literal'` /
/// `Class::$prop === 'literal'`.
fn narrow_static_prop_literal_string(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: &str,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    let narrowed = literal_string_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_literal_int`, for
/// `self::$prop === 42` / `static::$prop === 42` / `Class::$prop === 42`.
fn narrow_static_prop_literal_int(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: i64,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Loose-equality counterpart of `narrow_static_prop_literal_int`, for
/// `self::$prop == 42` / `static::$prop != 42` / `Class::$prop == 42` —
/// static-property sibling of `narrow_var_loose_int`, same safety gate.
fn narrow_static_prop_loose_int(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: i64,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.types.is_empty()
        || !current
            .types
            .iter()
            .all(|a| atom_safe_for_loose_int_narrowing(a, value))
    {
        return;
    }
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_bool`, for
/// `self::$prop === true` / `static::$prop === false` / `Class::$prop === true`.
fn narrow_static_prop_bool(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: bool,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = bool_narrow_type(&current, value, is_value);
    // mark_diverges=false: matches narrow_var_bool/narrow_prop_bool's rationale
    // — a separate contradiction pass already owns flagging an always-true/false compare.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// Static-property counterpart of `narrow_prop_loose_bool`, for
/// `self::$prop == true` / `false` (or `static::$prop`/`Class::$prop`).
fn narrow_static_prop_loose_bool(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    want_truthy: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = if want_truthy {
        current.narrow_to_truthy()
    } else {
        current.narrow_to_falsy()
    };
    // mark_diverges=false: matches narrow_prop_loose_bool's rationale — a
    // separate contradiction pass already owns flagging an always-true/false compare.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}
