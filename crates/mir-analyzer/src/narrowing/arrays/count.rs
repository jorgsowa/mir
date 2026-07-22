//! `count()`/`sizeof()`/`iterator_count()` and `array_key_first()`/
//! `array_key_last()` narrowing, for variable, property, and
//! static-property receivers.
use php_ast::ast::BinaryOp;
use php_ast::owned::ExprKind;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::core::{
    apply_prop_narrowed, count_or_strlen_emptiness, extract_static_prop_access,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, ScalarArgTarget,
};
use super::super::literals::{extract_int_literal, flip_comparison_op};

/// Extract the variable/property target from `count($var)` / `sizeof($var)` /
/// `iterator_count($var)` — all three return an int length and narrow
/// identically.
pub(crate) fn extract_count_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("count")
            || bare.eq_ignore_ascii_case("sizeof")
            || bare.eq_ignore_ascii_case("iterator_count")
        {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// Static-property counterpart of `extract_count_arg` — see
/// `extract_gettype_static_prop_arg` for why this is a separate,
/// call-site-local extractor rather than a `ScalarArgTarget` variant.
pub(crate) fn extract_count_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("count")
            || bare.eq_ignore_ascii_case("sizeof")
            || bare.eq_ignore_ascii_case("iterator_count")
        {
            if let Some(arg) = call.args.first() {
                return extract_static_prop_access(&arg.value, ctx, db, file);
            }
        }
    }
    None
}

/// Extract the variable/property target from `array_key_first($var)` /
/// `array_key_last($var)`.
pub(crate) fn extract_array_key_first_or_last_arg(
    expr: &php_ast::owned::Expr,
) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("array_key_first")
            || bare.eq_ignore_ascii_case("array_key_last")
        {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// Static-property counterpart of `extract_array_key_first_or_last_arg` —
/// `ScalarArgTarget` has no static-property variant (tracked as S19), so
/// extract it call-site-locally instead, mirroring the str_contains-family
/// recipe.
pub(crate) fn extract_array_key_first_or_last_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("array_key_first")
            || bare.eq_ignore_ascii_case("array_key_last")
        {
            if let Some(arg) = call.args.first() {
                return extract_static_prop_access(&arg.value, ctx, db, file);
            }
        }
    }
    None
}

/// `array_key_first($arr) !== null` / `array_key_last($arr) !== null` — a common
/// non-empty-array idiom, equivalent to `count($arr) > 0`. Both functions return
/// `null` only when the array is empty, so `!== null` proves it's non-empty and
/// `=== null` proves it's empty.
pub(crate) fn narrow_array_key_first_or_last_null(
    ctx: &mut FlowState,
    arr_var: &str,
    is_null: bool,
) {
    let current = ctx.get_var(arr_var);
    if current.is_mixed() {
        return;
    }
    // array_key_first()/array_key_last() throw a TypeError for a null
    // argument, so reaching either result already proves the array wasn't
    // null.
    let non_null = current.remove_null();
    let narrowed = if is_null {
        non_null.narrow_to_empty_collection()
    } else {
        non_null.narrow_to_non_empty_collection()
    };
    // `narrow_to_empty_collection` can filter every atom away when `current` is
    // already known to be exclusively non-empty (a provably-dead branch); leave
    // the type as-is rather than collapsing the variable to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(arr_var, narrowed);
    }
}

/// Property-access counterpart of `narrow_array_key_first_or_last_null`.
pub(crate) fn narrow_prop_array_key_first_or_last_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    // array_key_first()/array_key_last() on null is a TypeError, so reaching
    // either comparison result at all proves $obj->prop — and thus $obj —
    // was non-null, regardless of which direction was proven.
    narrow_receiver_non_null_on_prop_match(ctx, obj_var, true);
    if current.is_mixed() {
        return;
    }
    // The property's own value can't be null either, for the same reason.
    let non_null = current.remove_null();
    let narrowed = if is_null {
        non_null.narrow_to_empty_collection()
    } else {
        non_null.narrow_to_non_empty_collection()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Static-property counterpart of `narrow_array_key_first_or_last_null`.
pub(crate) fn narrow_static_prop_array_key_first_or_last_null(
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
    let non_null = current.remove_null();
    let narrowed = if is_null {
        non_null.narrow_to_empty_collection()
    } else {
        non_null.narrow_to_non_empty_collection()
    };
    if !narrowed.is_empty() && narrowed != current {
        apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
    }
}

/// Narrow an array variable based on `count($arr) op n` being `is_true`.
/// Promotes `array` / `list` to their non-empty variants when the comparison
/// proves the count is >= 1, or drops the non-empty variants when it proves
/// the count is exactly 0.
pub(crate) fn narrow_array_count_comparison(
    ctx: &mut FlowState,
    arr_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let current = ctx.get_var(arr_var);
    if current.is_mixed() {
        return;
    }
    // count()/sizeof()/iterator_count() throw a TypeError for a null
    // argument, so reaching ANY comparison result already proves the array
    // wasn't null — independent of whether this specific comparison also
    // proves emptiness.
    let non_null = current.remove_null();
    let narrowed = match count_or_strlen_emptiness(op, n, is_true) {
        Some(true) => non_null.narrow_to_non_empty_collection(),
        Some(false) => non_null.narrow_to_empty_collection(),
        None => non_null,
    };
    // `narrow_to_empty_collection` can filter every atom away when `current` is
    // already known to be exclusively non-empty (a provably-dead branch); leave
    // the type as-is rather than collapsing the variable to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(arr_var, narrowed);
    }
}

/// Property-access counterpart of `narrow_array_count_comparison`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn narrow_prop_array_count_comparison(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    // count() on a non-Countable (including null) is a TypeError in PHP 8, so
    // reaching either comparison result at all proves $obj->prop — and thus
    // $obj — was non-null, regardless of which direction was proven or of
    // the property's own (possibly mixed) type.
    narrow_receiver_non_null_on_prop_match(ctx, obj_var, true);
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    // The property's own value can't be null either, for the same reason.
    let non_null = current.remove_null();
    let narrowed = match count_or_strlen_emptiness(op, n, is_true) {
        Some(true) => non_null.narrow_to_non_empty_collection(),
        Some(false) => non_null.narrow_to_empty_collection(),
        None => non_null,
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Static-property counterpart of `narrow_prop_array_count_comparison`.
/// There's no nullable receiver variable for a static property, so no
/// receiver-non-null propagation is needed.
pub(crate) fn narrow_static_prop_array_count_comparison(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    // count()/sizeof()/iterator_count() throw a TypeError for a null
    // argument, so reaching ANY comparison result already proves the
    // property wasn't null.
    let non_null = current.remove_null();
    let narrowed = match count_or_strlen_emptiness(op, n, is_true) {
        Some(true) => non_null.narrow_to_non_empty_collection(),
        Some(false) => non_null.narrow_to_empty_collection(),
        None => non_null,
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// Condition-matching glue for `narrow_from_condition`'s relational
/// (`<`/`<=`/`>`/`>=`) comparison arm: recognizes `count($arr) op N` /
/// `N op count($arr)` (for a var, prop, or static-prop receiver) and
/// dispatches to the narrowing helpers above. A no-op when neither side is
/// a `count()`/`sizeof()`/`iterator_count()` call.
pub(crate) fn narrow_array_count_condition(
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    op: BinaryOp,
    is_true: bool,
) {
    // count($arr) op N  /  N op count($arr) — normalize so count call is on left.
    let count_call_on_left = extract_count_arg(left).is_some()
        || extract_count_static_prop_arg(left, ctx, db, file).is_some();
    let (count_expr, count_cmp_op, count_lit) = if count_call_on_left {
        (left, op, right)
    } else {
        (right, flip_comparison_op(op), left)
    };
    if let (Some(target), Some(n)) = (
        extract_count_arg(count_expr),
        extract_int_literal(count_lit),
    ) {
        match target {
            ScalarArgTarget::Var(arr_var) => {
                narrow_array_count_comparison(ctx, &arr_var, count_cmp_op, n, is_true)
            }
            ScalarArgTarget::Prop(obj, prop) => narrow_prop_array_count_comparison(
                ctx,
                &obj,
                &prop,
                db,
                file,
                count_cmp_op,
                n,
                is_true,
            ),
        }
    } else if let (Some((fqcn, prop)), Some(n)) = (
        extract_count_static_prop_arg(count_expr, ctx, db, file),
        extract_int_literal(count_lit),
    ) {
        narrow_static_prop_array_count_comparison(ctx, &fqcn, &prop, db, count_cmp_op, n, is_true);
    }
}
