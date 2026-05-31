use mir_types::{Atomic, Name, Type};
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
