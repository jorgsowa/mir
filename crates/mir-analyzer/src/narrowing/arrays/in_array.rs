//! `in_array()`/`array_search()` haystack-literal narrowing: extracting a
//! union of literal values from an array-literal or shape-typed haystack
//! argument, and the strict/loose comparison-safety rule shared by both
//! builtins' dispatch arms (`array_search()`'s own dispatch arm stays in
//! `mod.rs` since it also spans the string domain via
//! `strings::narrow_string_false_comparable_condition`).
use php_ast::owned::{ExprKind, FunctionCallExpr};

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::core::{
    apply_prop_narrowed, extract_any_prop_access, extract_static_prop_access, extract_var_name,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, UnionNarrowExt,
};
use super::super::literals::is_truthy_bool_literal;

/// Shared by the `Variable` and property-access arms of `extract_haystack_type`:
/// collect the TLiteralString/TLiteralInt values inside a resolved array
/// type's `TKeyedArray` properties, bailing out on any non-literal value.
fn haystack_type_from_array_type(var_ty: &Type) -> Option<Type> {
    if var_ty.is_mixed() || var_ty.is_empty() {
        return None;
    }
    let mut ty = Type::empty();
    for atomic in &var_ty.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                for prop in properties.values() {
                    match &prop.ty.types[..] {
                        [Atomic::TLiteralString(_)] | [Atomic::TLiteralInt(_)] => {
                            for a in &prop.ty.types {
                                ty.add_type(a.clone());
                            }
                        }
                        _ => return None, // non-literal value
                    }
                }
            }
            _ => return None,
        }
    }
    if ty.is_empty() {
        None
    } else {
        Some(ty)
    }
}

/// Extract a union Type from an `in_array` haystack argument.
/// Supports:
/// - Literal arrays: `['a', 'b', 1]` → union of `TLiteralString` / `TLiteralInt`
/// - Variables/property accesses: resolve the current type and collect the
///   TLiteralString/TLiteralInt values inside the TKeyedArray's properties.
pub(crate) fn extract_haystack_type(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<Type> {
    match &expr.kind {
        ExprKind::Array(elements) => {
            let mut ty = Type::empty();
            for item in elements.iter() {
                match &item.value.kind {
                    ExprKind::String(s) => {
                        ty.add_type(Atomic::TLiteralString(std::sync::Arc::from(s.as_ref())))
                    }
                    ExprKind::Int(n) => ty.add_type(Atomic::TLiteralInt(*n)),
                    _ => return None, // non-literal element — bail out
                }
            }
            if ty.is_empty() {
                None
            } else {
                Some(ty)
            }
        }
        ExprKind::Variable(name) => {
            let var_name = name.trim_start_matches('$');
            haystack_type_from_array_type(&ctx.get_var(var_name))
        }
        ExprKind::PropertyAccess(_) | ExprKind::NullsafePropertyAccess(_) => {
            let (obj, prop) = extract_any_prop_access(expr)?;
            haystack_type_from_array_type(&resolve_prop_current_type(ctx, &obj, &prop, db, file))
        }
        ExprKind::StaticPropertyAccess(_) => {
            let (fqcn, prop) = extract_static_prop_access(expr, ctx, db, file)?;
            haystack_type_from_array_type(&resolve_static_prop_current_type(ctx, &fqcn, &prop, db))
        }
        ExprKind::Parenthesized(inner) => extract_haystack_type(inner, ctx, db, file),
        _ => None,
    }
}

/// Narrow `current` to only the atomic types that overlap with `haystack` literals.
/// For each literal atom in `haystack` (TLiteralString / TLiteralInt): keep it in
/// the output if `current` could hold that value — i.e., the literal is a subtype
/// of at least one atom in `current`.
pub(crate) fn narrow_to_haystack_values(current: &Type, haystack: &Type) -> Type {
    let mut out = Type::empty();
    for hay_atom in &haystack.types {
        let lit_ty = Type::single(hay_atom.clone());
        if lit_ty.is_subtype_structural(current) {
            out.add_type(hay_atom.clone());
        }
    }
    out
}

/// Whether narrowing `current` by an `in_array()`/`!in_array()` check against
/// `haystack` is sound without a strict (third-argument) comparison. Loose
/// (`==`) comparison agrees with strict (`===`) comparison whenever both
/// sides are exclusively strings, or exclusively ints — cross-category
/// comparisons (e.g. a string needle against an int haystack) can match via
/// PHP's loose-equality coercion rules in ways a same-category narrowing
/// would incorrectly rule out.
pub(crate) fn in_array_loose_narrowing_is_safe(current: &Type, haystack: &Type) -> bool {
    fn all(ty: &Type, pred: fn(&Atomic) -> bool) -> bool {
        !ty.types.is_empty() && ty.types.iter().all(pred)
    }
    (all(current, Atomic::is_int) && all(haystack, Atomic::is_int))
        || (all(current, Atomic::is_string) && all(haystack, Atomic::is_string))
}

/// Condition-matching glue for `narrow_from_condition`'s `FunctionCall` arm:
/// handles `in_array($needle, $haystack)`, dispatching to the var/prop/
/// static-prop haystack narrowing above. Callers are expected to have
/// already checked that the function name is `in_array` before calling this.
pub(crate) fn narrow_in_array_condition(
    ctx: &mut FlowState,
    call: &FunctionCallExpr,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    // in_array($needle, ['a', 'b', 'c']) true →
    // narrow $needle to 'a'|'b'|'c'. Only safe when either the
    // 3rd (strict) argument is truthy, or $needle's current
    // type and the haystack are both exclusively string atoms
    // or both exclusively int atoms — for same-category
    // scalars, loose (==) comparison agrees with strict
    // (===). A mixed-category needle (e.g. int|string) can't
    // be narrowed under loose comparison: the string "1"
    // loosely equals the int 1, so a haystack of `[1, 2]`
    // doesn't rule out $needle being the string "1".
    let strict = call
        .args
        .get(2)
        .map(|a| is_truthy_bool_literal(&a.value))
        .unwrap_or(false);
    if let (Some(needle_arg), Some(haystack_arg)) = (call.args.first(), call.args.get(1)) {
        if let Some(var_name) = extract_var_name(&needle_arg.value) {
            if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx, db, file) {
                let current = ctx.get_var(&var_name);
                let loose_safe = strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                if !current.is_mixed() && is_true && loose_safe {
                    // intersect: keep only types that could match a haystack value
                    let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                    if !narrowed.is_empty() && narrowed != current {
                        ctx.set_var(&var_name, narrowed);
                    }
                } else if !current.is_mixed() && !is_true && loose_safe {
                    // False branch: safe only when the current type is a
                    // finite literal union — remove the matched haystack values.
                    let all_literals = !current.types.is_empty()
                        && current.types.iter().all(|a| {
                            matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                        });
                    if all_literals {
                        let narrowed =
                            current.filter(|a| !haystack_ty.types.iter().any(|h| h == a));
                        if !narrowed.is_empty() && narrowed != current {
                            ctx.set_var(&var_name, narrowed);
                        }
                    }
                }
            }
        } else if let Some((obj, prop)) = extract_any_prop_access(&needle_arg.value) {
            // Property-access counterpart of the plain-variable case
            // above, e.g. `in_array($this->status, ['a', 'b'])`.
            if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx, db, file) {
                if is_true {
                    // in_array(null, $haystack) only matches loosely
                    // when the haystack contains a falsy literal (0,
                    // "", "0"); a strict comparison can never match
                    // null at all, since our haystack extraction never
                    // includes a literal null element. So a true match
                    // proves the receiver wasn't null, except in that
                    // one loose-comparison edge case.
                    let haystack_admits_null_loosely = haystack_ty.types.iter().any(|a| {
                        matches!(a, Atomic::TLiteralInt(0))
                            || matches!(a, Atomic::TLiteralString(s) if s.as_ref() == "" || s.as_ref() == "0")
                    });
                    if strict || !haystack_admits_null_loosely {
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
                    }
                }
                let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                let loose_safe = strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                if !current.is_mixed() && is_true && loose_safe {
                    let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                    if !narrowed.is_empty() {
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                    }
                } else if !current.is_mixed() && !is_true && loose_safe {
                    let all_literals = !current.types.is_empty()
                        && current.types.iter().all(|a| {
                            matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                        });
                    if all_literals {
                        let narrowed =
                            current.filter(|a| !haystack_ty.types.iter().any(|h| h == a));
                        if !narrowed.is_empty() {
                            apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        }
                    }
                }
            }
        } else if let Some((fqcn, prop)) =
            extract_static_prop_access(&needle_arg.value, ctx, db, file)
        {
            // Static-property counterpart of the instance-property
            // case above, e.g. `in_array(self::$status, ['a', 'b'])`.
            if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx, db, file) {
                // No receiver-non-null propagation here, unlike the
                // instance-property case above: a static property
                // has no separate receiver variable whose
                // nullability this could establish (`self::`/
                // `static::` is never itself null).
                let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                let loose_safe = strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                if !current.is_mixed() && is_true && loose_safe {
                    let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                    if !narrowed.is_empty() {
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                    }
                } else if !current.is_mixed() && !is_true && loose_safe {
                    let all_literals = !current.types.is_empty()
                        && current.types.iter().all(|a| {
                            matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                        });
                    if all_literals {
                        let narrowed =
                            current.filter(|a| !haystack_ty.types.iter().any(|h| h == a));
                        if !narrowed.is_empty() {
                            apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, false);
                        }
                    }
                }
            }
        }
    }
}
