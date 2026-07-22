//! `strlen()`/`mb_strlen()`/`iconv_strlen()` length narrowing and the
//! `strpos()`/`stripos()`-family `!== false` idiom, for variable, property,
//! and static-property receivers.
use php_ast::ast::BinaryOp;
use php_ast::owned::ExprKind;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, count_or_strlen_emptiness, extract_static_prop_access,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, ScalarArgTarget,
};
use super::literals::{
    expr_is_nonempty_string_literal, narrow_string_to_empty, narrow_string_to_non_empty,
};

/// Extract the variable/property target from `strlen($var)` /
/// `mb_strlen($var, ...)` / `iconv_strlen($var, ...)`.
pub(super) fn extract_strlen_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("strlen")
            || bare.eq_ignore_ascii_case("mb_strlen")
            || bare.eq_ignore_ascii_case("iconv_strlen")
        {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// Static-property counterpart of `extract_strlen_arg`.
pub(super) fn extract_strlen_static_prop_arg(
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
        if bare.eq_ignore_ascii_case("strlen")
            || bare.eq_ignore_ascii_case("mb_strlen")
            || bare.eq_ignore_ascii_case("iconv_strlen")
        {
            if let Some(arg) = call.args.first() {
                return extract_static_prop_access(&arg.value, ctx, db, file);
            }
        }
    }
    None
}

/// Narrow a string variable based on `strlen($str) op n` being `is_true`.
/// Promotes `string` to `non-empty-string` when the comparison proves length
/// >= 1, or drops `non-empty-string` when it proves length is exactly 0.
pub(super) fn narrow_string_strlen_comparison(
    ctx: &mut FlowState,
    str_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = ctx.get_var(str_var);
    if current.is_mixed() {
        return;
    }
    // strlen(null) doesn't throw (returns 0, deprecation notice only), so
    // only a proven-non-empty result excludes a null value — the empty
    // direction is also satisfiable by the string being null.
    let base = if non_empty {
        current.remove_null()
    } else {
        current.clone()
    };
    let narrowed = if non_empty {
        narrow_string_to_non_empty(&base)
    } else {
        narrow_string_to_empty(&base)
    };
    // Same rationale as the array case above: don't collapse to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(str_var, narrowed);
    }
}

/// Property-access counterpart of `narrow_string_strlen_comparison`.
#[allow(clippy::too_many_arguments)]
pub(super) fn narrow_prop_string_strlen_comparison(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    // strlen(null) returns 0 (no TypeError), so only a proven-non-empty
    // result excludes a null receiver — the empty direction is also
    // satisfiable by $obj being null. Independent of the property's own
    // (possibly mixed) type, so apply it before the mixed bail below.
    if non_empty {
        narrow_receiver_non_null_on_prop_match(ctx, obj_var, true);
    }
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    // Same reasoning as the receiver above: only the non-empty direction
    // proves the property's own value wasn't null.
    let base = if non_empty {
        current.remove_null()
    } else {
        current.clone()
    };
    let narrowed = if non_empty {
        narrow_string_to_non_empty(&base)
    } else {
        narrow_string_to_empty(&base)
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Static-property counterpart of `narrow_prop_string_strlen_comparison`.
pub(super) fn narrow_static_prop_string_strlen_comparison(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    // Only the non-empty direction proves the property's own value wasn't null.
    let base = if non_empty {
        current.remove_null()
    } else {
        current.clone()
    };
    let narrowed = if non_empty {
        narrow_string_to_non_empty(&base)
    } else {
        narrow_string_to_empty(&base)
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// The `strpos()`/`stripos()`-family (`!== false`) arm of
/// `narrow_from_false_comparable_call` — split out of that function (see
/// `mod.rs`'s remaining `array_search()` arm) because the two builtin
/// families narrow entirely different domains (string vs. array) and share
/// no state beyond the dispatch already done by the caller.
///
/// Found (`result != false`) proves the haystack is non-empty, mirroring
/// `str_contains()`'s true-branch narrowing — only sound for a non-empty
/// literal needle (an empty needle is "found" at offset 0 vacuously).
pub(super) fn narrow_string_false_comparable_condition(
    call: &php_ast::owned::FunctionCallExpr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    is_false: bool,
) {
    if is_false {
        return;
    }
    let (Some(haystack_arg), Some(needle_arg)) = (call.args.first(), call.args.get(1)) else {
        return;
    };
    let needle_non_empty = expr_is_nonempty_string_literal(&needle_arg.value, ctx, db, file);
    if !needle_non_empty {
        return;
    }
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
            // Found proves the haystack read wasn't null-derived (PHP
            // coerces null to "", and a non-empty needle can't match at
            // any offset in ""), which proves the receiver was non-null.
            narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
            let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
            if !current.is_mixed() {
                let narrowed = narrow_string_to_non_empty(&current);
                apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
            }
        }
        None => {
            // ScalarArgTarget has no static-property variant (tracked
            // as S19) — extract it call-site-locally instead, mirroring
            // the str_contains-family recipe.
            if let Some((fqcn, prop)) =
                extract_static_prop_access(&haystack_arg.value, ctx, db, file)
            {
                let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                if !current.is_mixed() {
                    let narrowed = narrow_string_to_non_empty(&current);
                    apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                }
            }
        }
    }
}
