//! `in_array()`/`array_search()` haystack-literal narrowing: extracting a
//! union of literal values from an array-literal or shape-typed haystack
//! argument, and the strict/loose comparison-safety rule shared by both
//! builtins' dispatch arms (which stay in `mod.rs` since they also span the
//! string domain via `strings::narrow_string_false_comparable_condition`).
use php_ast::owned::ExprKind;

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::core::{
    extract_any_prop_access, extract_static_prop_access, resolve_prop_current_type,
    resolve_static_prop_current_type,
};

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
