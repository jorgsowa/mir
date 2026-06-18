use mir_types::{Atomic, Name, Type};
use php_ast::ast::BinaryOp;
use php_ast::owned::{Expr, ExprKind};
use rustc_hash::FxHashSet;

use crate::subtype::is_subtype;

#[allow(dead_code)]
pub fn widen_array_with_value(current: &Type, new_value: &Type) -> Type {
    widen_array_with_value_and_key(current, new_value, &Type::mixed())
}

pub fn widen_array_with_value_and_key(current: &Type, new_value: &Type, new_key: &Type) -> Type {
    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut found_array = false;
    // Merge ALL array-like variants from current into a single accumulated TArray/TList.
    // Without this, each TArray variant in a growing union independently emits a new TArray,
    // causing unbounded union growth across salsa fixpoint iterations (infinite recursion).
    let mut acc_key: Option<Type> = None;
    let mut acc_value: Option<Type> = None;
    let mut acc_list: Option<Type> = None;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                let mut all_values = new_value.clone();
                let mut all_keys = new_key.clone();
                for prop in properties.values() {
                    all_values.merge_with(&prop.ty);
                }
                for k in properties.keys() {
                    let key_atomic = match k {
                        mir_types::ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                        mir_types::ArrayKey::Int(i) => Atomic::TLiteralInt(*i),
                    };
                    all_keys.merge_with(&Type::single(key_atomic));
                }
                fold_into(&mut acc_key, all_keys);
                fold_into(&mut acc_value, all_values);
                found_array = true;
            }
            Atomic::TArray { key, value } => {
                fold_into(&mut acc_key, Type::merge(key, new_key));
                fold_into(&mut acc_value, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                fold_into(&mut acc_list, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TNonEmptyArray { key, value } => {
                fold_into(&mut acc_key, Type::merge(key, new_key));
                fold_into(&mut acc_value, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TMixed => {
                return Type::mixed();
            }
            other => {
                result.add_type(other.clone());
            }
        }
    }
    if let (Some(key), Some(value)) = (acc_key, acc_value) {
        result.add_type(Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        });
    }
    if let Some(v) = acc_list {
        result.add_type(Atomic::TList { value: Box::new(v) });
    }
    if !found_array {
        return current.clone();
    }
    result
}

/// Widen an existing array-like type by appending `new_value` via push notation (`[]`).
/// Always produces `TList { merged_value }`, regardless of the current key type,
/// because push notation in PHP assigns the next integer index.
pub fn widen_array_as_list(current: &Type, new_value: &Type) -> Type {
    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut acc: Option<Type> = Some(new_value.clone());
    let mut found_array = false;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                for prop in properties.values() {
                    fold_into(&mut acc, prop.ty.clone());
                }
                found_array = true;
            }
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                fold_into(&mut acc, *value.clone());
                found_array = true;
            }
            Atomic::TMixed => return Type::mixed(),
            other => result.add_type(other.clone()),
        }
    }
    if !found_array {
        return current.clone();
    }
    if let Some(v) = acc {
        result.add_type(Atomic::TList { value: Box::new(v) });
    }
    result
}

fn fold_into(acc: &mut Option<Type>, new: Type) {
    match acc {
        None => *acc = Some(new),
        Some(existing) => existing.merge_with(&new),
    }
}

/// The inclusive integer bounds of `ty` when it is an integer-only type, as
/// `(min, max)` where `None` means unbounded on that side. Returns `None` when
/// any member is not an integer (so the caller falls back to scalar inference).
/// Literals are exact bounds; a general `int` is unbounded both ways.
fn int_bounds(ty: &Type) -> Option<(Option<i64>, Option<i64>)> {
    if ty.types.is_empty() {
        return None;
    }
    let mut min: Option<i64> = Some(i64::MAX);
    let mut max: Option<i64> = Some(i64::MIN);
    for a in &ty.types {
        let (lo, hi) = match a {
            Atomic::TLiteralInt(n) => (Some(*n), Some(*n)),
            Atomic::TIntRange { min, max } => (*min, *max),
            // Named int subtypes carry implicit bounds: use them so arithmetic
            // like `positive-int + 1` yields `int<2, max>` rather than bare `int`.
            Atomic::TPositiveInt => (Some(1), None),
            Atomic::TNonNegativeInt => (Some(0), None),
            Atomic::TNegativeInt => (None, Some(-1)),
            Atomic::TInt => (None, None),
            _ => return None,
        };
        // Widen the accumulated bounds to cover this member (union semantics).
        min = match (min, lo) {
            (Some(m), Some(l)) => Some(m.min(l)),
            _ => None,
        };
        max = match (max, hi) {
            (Some(m), Some(h)) => Some(m.max(h)),
            _ => None,
        };
    }
    Some((min, max))
}

/// Whether `ty` carries an explicit integer range or a named int subtype with
/// known implicit bounds (positive-int, non-negative-int, negative-int).
fn contains_int_range(ty: &Type) -> bool {
    ty.types.iter().any(|a| {
        matches!(
            a,
            Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNonNegativeInt
                | Atomic::TNegativeInt
        )
    })
}

/// Range-aware integer arithmetic for `+` and `-`: when at least one operand is
/// an integer range (e.g. a `count()` result), propagate faithful bounds so
/// `count($a) + 1` is `int<1, max>` and `count($a) - 1` is `int<-1, max>`.
/// Returns `None` for anything else (including literal-only arithmetic, left to
/// [`infer_arithmetic`] so it is not perturbed).
fn as_single_literal_int(ty: &Type) -> Option<i64> {
    if ty.types.len() == 1 {
        if let Atomic::TLiteralInt(n) = &ty.types[0] {
            return Some(*n);
        }
    }
    None
}

pub fn infer_int_range_arithmetic(left: &Type, right: &Type, op: BinaryOp) -> Option<Type> {
    // Fast path: both operands are known literal ints — fold at analysis time.
    if let (Some(l), Some(r)) = (as_single_literal_int(left), as_single_literal_int(right)) {
        let result = match op {
            BinaryOp::Add => l.checked_add(r),
            BinaryOp::Sub => l.checked_sub(r),
            BinaryOp::Mul => l.checked_mul(r),
            // Integer division only when divisor is nonzero and result is exact.
            BinaryOp::Div if r != 0 && l % r == 0 => Some(l / r),
            BinaryOp::Mod if r != 0 => Some(l % r),
            _ => None,
        };
        if let Some(n) = result {
            return Some(Type::single(Atomic::TLiteralInt(n)));
        }
    }

    // Only engage when a genuine range is in play; plain int/literal operands
    // keep the existing scalar inference.
    if !contains_int_range(left) && !contains_int_range(right) {
        return None;
    }
    let (lmin, lmax) = int_bounds(left)?;
    let (rmin, rmax) = int_bounds(right)?;
    let add = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_add(b),
        _ => None,
    };
    let sub = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_sub(b),
        _ => None,
    };
    let mul_opt = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_mul(b),
        _ => None,
    };
    let (min, max) = match op {
        BinaryOp::Add => (add(lmin, rmin), add(lmax, rmax)),
        // [lmin,lmax] - [rmin,rmax] = [lmin - rmax, lmax - rmin]
        BinaryOp::Sub => (sub(lmin, rmax), sub(lmax, rmin)),
        // Multiplication: only handle the case where both operands are non-negative,
        // which is the common case (`count * stride`, `width * height`, etc.).
        // lmin/rmin must be Some(>=0) — None means unbounded below, i.e., can be negative.
        // For mixed-sign operands the four-corner product is complex; defer to infer_arithmetic.
        BinaryOp::Mul if lmin.is_some_and(|m| m >= 0) && rmin.is_some_and(|m| m >= 0) => {
            (mul_opt(lmin, rmin), mul_opt(lmax, rmax))
        }
        _ => return None,
    };
    Some(Type::single(Atomic::TIntRange { min, max }))
}

pub fn infer_arithmetic(left: &Type, right: &Type) -> Type {
    if left.is_mixed() || right.is_mixed() {
        return Type::mixed();
    }

    let left_is_array = left.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    let right_is_array = right.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    if left_is_array || right_is_array {
        let merged_left = if left_is_array {
            left.clone()
        } else {
            Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TMixed)),
                value: Box::new(Type::mixed()),
            })
        };
        return merged_left;
    }

    let left_is_float = left.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    let right_is_float =
        right.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    if left_is_float || right_is_float {
        Type::single(Atomic::TFloat)
    } else if left.contains(|t| t.is_int()) && right.contains(|t| t.is_int()) {
        Type::single(Atomic::TInt)
    } else {
        let mut u = Type::empty();
        u.add_type(Atomic::TInt);
        u.add_type(Atomic::TFloat);
        u
    }
}

/// Extract the string representation of a single scalar literal for concat folding.
/// Returns `None` for unions or non-literal types.
pub fn as_concat_str(ty: &Type) -> Option<String> {
    if ty.types.len() != 1 {
        return None;
    }
    match &ty.types[0] {
        Atomic::TLiteralString(s) => Some(s.as_ref().to_string()),
        Atomic::TLiteralInt(n) => Some(n.to_string()),
        Atomic::TTrue => Some("1".to_string()),
        Atomic::TFalse => Some(String::new()),
        _ => None,
    }
}

pub fn extract_simple_var(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_simple_var(inner),
        _ => None,
    }
}

pub fn extract_destructure_vars(expr: &Expr) -> Vec<String> {
    match &expr.kind {
        ExprKind::Array(elements) => {
            let mut vars = vec![];
            for elem in elements.iter() {
                let sub = extract_destructure_vars(&elem.value);
                if sub.is_empty() {
                    if let Some(v) = extract_simple_var(&elem.value) {
                        vars.push(v);
                    }
                } else {
                    vars.extend(sub);
                }
            }
            vars
        }
        _ => vec![],
    }
}

pub(crate) fn ast_params_to_fn_params_resolved(
    params: &[php_ast::owned::Param],
    self_fqcn: Option<&str>,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<mir_codebase::FnParam> {
    params
        .iter()
        .map(|p| {
            let name_str = p.name.as_deref().unwrap_or("").trim_start_matches('$');
            let ty = p
                .type_hint
                .as_ref()
                .map(|h| crate::parser::type_from_hint_owned(h, self_fqcn))
                .map(|u| resolve_named_objects_in_union(u, db, file));
            mir_codebase::FnParam {
                name: Name::new(name_str),
                ty: mir_codebase::wrap_param_type(ty),
                has_default: p.default.is_some(),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            }
        })
        .collect()
}

pub(crate) fn resolve_named_objects_in_union(
    union: Type,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Type {
    let from_docblock = union.from_docblock;
    let possibly_undefined = union.possibly_undefined;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TNamedObject { fqcn, type_params } => {
                let resolved = crate::db::resolve_name(db, file, fqcn.as_ref());
                Atomic::TNamedObject {
                    fqcn: resolved.into(),
                    type_params,
                }
            }
            other => other,
        })
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

pub(crate) fn extract_string_from_expr(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(s) => Some(s.trim_start_matches('$').to_string()),
        ExprKind::Variable(_) => None,
        ExprKind::String(s) => Some(s.to_string()),
        _ => None,
    }
}

/// For a literal `switch` case / `match` arm condition, return a
/// `(dedup_key, display)` pair. The key is type-tagged so that distinct
/// literal kinds never collide (e.g. the int `0` and the string `"0"`),
/// keeping duplicate detection free of PHP's loose-comparison surprises.
///
/// Returns `None` for any non-literal (variables, calls, negation, floats,
/// …) so dynamic conditions are never flagged — duplicate detection stays at
/// zero false positives.
fn literal_condition_key(expr: &Expr) -> Option<(String, String)> {
    match &expr.kind {
        ExprKind::Int(n) => Some((format!("int:{n}"), n.to_string())),
        ExprKind::String(s) => Some((format!("str:{s}"), format!("\"{s}\""))),
        ExprKind::Bool(b) => Some((format!("bool:{b}"), b.to_string())),
        ExprKind::Null => Some(("null".to_string(), "null".to_string())),
        _ => None,
    }
}

/// Given `switch`/`match` condition expressions in source order, return the
/// `(span, display)` of each literal whose value repeats an earlier one — the
/// duplicate branch can never be reached. Non-literal conditions are ignored,
/// so dynamic arms are never flagged.
pub fn duplicate_literal_conditions<'e>(
    conditions: impl Iterator<Item = &'e Expr>,
) -> Vec<(php_ast::Span, String)> {
    let mut seen = FxHashSet::default();
    let mut duplicates = Vec::new();
    for cond in conditions {
        if let Some((key, display)) = literal_condition_key(cond) {
            if !seen.insert(key) {
                duplicates.push((cond.span, display));
            }
        }
    }
    duplicates
}

/// Returns true if `ty` contains any reference to a template param name from `names`,
/// including names nested inside generic type arguments (e.g. `R` inside `Result<Throwable, R>`).
/// Handles both `TTemplateParam` and the docblock-parser workaround where bare unqualified names
/// are emitted as `TNamedObject { fqcn: "T", type_params: [] }`.
pub(crate) fn type_refs_any_template(ty: &Type, names: &FxHashSet<Name>) -> bool {
    fn check_atomic(a: &Atomic, names: &FxHashSet<Name>) -> bool {
        match a {
            Atomic::TTemplateParam { name, .. } => names.contains(name),
            Atomic::TNamedObject { fqcn, type_params } => {
                if type_params.is_empty() && !fqcn.contains('\\') && names.contains(fqcn) {
                    return true;
                }
                type_params
                    .iter()
                    .any(|tp| tp.types.iter().any(|a| check_atomic(a, names)))
            }
            Atomic::TClassString(Some(inner)) => !inner.contains('\\') && names.contains(inner),
            _ => false,
        }
    }
    ty.types.iter().any(|a| check_atomic(a, names))
}

fn scalar_types_compatible(value_ty: &Type, prop_ty: &Type) -> bool {
    value_ty.is_subtype_structural(prop_ty)
}

pub(crate) fn property_assign_compatible(
    value_ty: &Type,
    prop_ty: &Type,
    db: &dyn crate::db::MirDatabase,
) -> bool {
    if scalar_types_compatible(value_ty, prop_ty) {
        return true;
    }
    if is_subtype(db, value_ty, prop_ty) {
        return true;
    }
    value_ty.types.iter().all(|a| match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TClosure { .. } | Atomic::TCallable { .. } => prop_ty.types.iter().any(|p| {
            matches!(p, Atomic::TClosure { .. } | Atomic::TCallable { .. })
                || matches!(p, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Closure")
        }),
        Atomic::TNever => true,
        Atomic::TNull => prop_ty.is_nullable(),
        _ => false,
    })
}

pub(crate) fn is_property_type_coercion(
    value_ty: &Type,
    prop_ty: &Type,
    db: &dyn crate::db::MirDatabase,
) -> bool {
    if value_ty.is_mixed() || prop_ty.is_mixed() {
        return false;
    }
    let value_core = value_ty.core_type();
    if value_core.types.is_empty() || !value_core.is_single() {
        return false;
    }
    let val_fqcn = match value_core.types.first().unwrap() {
        Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => *fqcn,
        _ => return false,
    };
    prop_ty.types.iter().any(|p| {
        let prop_fqcn = match p {
            Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => fqcn,
            _ => return false,
        };
        crate::db::extends_or_implements(db, prop_fqcn.as_ref(), val_fqcn.as_ref())
    })
}

#[cfg(test)]
mod range_arithmetic_tests {
    use super::*;

    fn range(min: Option<i64>, max: Option<i64>) -> Type {
        Type::single(Atomic::TIntRange { min, max })
    }

    fn lit(n: i64) -> Type {
        Type::single(Atomic::TLiteralInt(n))
    }

    #[test]
    fn add_shifts_both_bounds() {
        // int<0, 4> + 5  =>  int<5, 9>
        let r =
            infer_int_range_arithmetic(&range(Some(0), Some(4)), &lit(5), BinaryOp::Add).unwrap();
        assert_eq!(r.to_string(), "int<5, 9>");
    }

    #[test]
    fn add_keeps_unbounded_upper() {
        // int<0, max> + 5  =>  int<5, max>
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(5), BinaryOp::Add).unwrap();
        assert_eq!(r.to_string(), "int<5, max>");
    }

    #[test]
    fn sub_lowers_min_to_negative() {
        // int<0, max> - 1  =>  int<-1, max>   (lmin - rmax, lmax - rmin)
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(1), BinaryOp::Sub).unwrap();
        assert_eq!(r.to_string(), "int<-1, max>");
    }

    #[test]
    fn add_overflow_saturates_to_unbounded() {
        // int<i64::MAX, i64::MAX> + 1  =>  both bounds overflow to unbounded,
        // which renders as the bare `int`.
        let r = infer_int_range_arithmetic(
            &range(Some(i64::MAX), Some(i64::MAX)),
            &lit(1),
            BinaryOp::Add,
        )
        .unwrap();
        assert_eq!(r.to_string(), "int");
    }

    #[test]
    fn no_range_operand_returns_none() {
        // plain int + literal: no explicit range, so range arithmetic abstains
        assert!(
            infer_int_range_arithmetic(&Type::single(Atomic::TInt), &lit(3), BinaryOp::Add)
                .is_none()
        );
    }

    #[test]
    fn non_integer_operand_returns_none() {
        // range + string: not integer-only, abstain
        assert!(infer_int_range_arithmetic(
            &range(Some(0), None),
            &Type::single(Atomic::TString),
            BinaryOp::Add
        )
        .is_none());
    }

    #[test]
    fn mul_non_negative_ranges() {
        // non-negative × literal positive → int<0, max> (unbounded above)
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(2), BinaryOp::Mul).unwrap();
        assert_eq!(r, range(Some(0), None));

        // bounded × bounded → bounded product
        let r = infer_int_range_arithmetic(
            &range(Some(2), Some(4)),
            &range(Some(3), Some(6)),
            BinaryOp::Mul,
        )
        .unwrap();
        assert_eq!(r, range(Some(6), Some(24)));

        // mixed-sign operand: defer to infer_arithmetic
        assert!(
            infer_int_range_arithmetic(&range(None, Some(-1)), &lit(2), BinaryOp::Mul).is_none()
        );
    }
}
