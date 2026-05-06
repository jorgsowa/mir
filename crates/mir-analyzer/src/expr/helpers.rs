use mir_types::{Atomic, Union};
use php_ast::ast::{Expr, ExprKind};
use std::sync::Arc;

pub fn widen_array_with_value(current: &Union, new_value: &Union) -> Union {
    let mut result = Union::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut found_array = false;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                let mut all_values = new_value.clone();
                for prop in properties.values() {
                    all_values = Union::merge(&all_values, &prop.ty);
                }
                result.add_type(Atomic::TArray {
                    key: Box::new(Union::mixed()),
                    value: Box::new(all_values),
                });
                found_array = true;
            }
            Atomic::TArray { key, value } => {
                let merged = Union::merge(value, new_value);
                result.add_type(Atomic::TArray {
                    key: key.clone(),
                    value: Box::new(merged),
                });
                found_array = true;
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                let merged = Union::merge(value, new_value);
                result.add_type(Atomic::TList {
                    value: Box::new(merged),
                });
                found_array = true;
            }
            Atomic::TMixed => {
                return Union::mixed();
            }
            other => {
                result.add_type(other.clone());
            }
        }
    }
    if !found_array {
        return current.clone();
    }
    result
}

pub fn infer_arithmetic(left: &Union, right: &Union) -> Union {
    if left.is_mixed() || right.is_mixed() {
        return Union::mixed();
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
            Union::single(Atomic::TArray {
                key: Box::new(Union::single(Atomic::TMixed)),
                value: Box::new(Union::mixed()),
            })
        };
        return merged_left;
    }

    let left_is_float = left.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    let right_is_float =
        right.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)));
    if left_is_float || right_is_float {
        Union::single(Atomic::TFloat)
    } else if left.contains(|t| t.is_int()) && right.contains(|t| t.is_int()) {
        Union::single(Atomic::TInt)
    } else {
        let mut u = Union::empty();
        u.add_type(Atomic::TInt);
        u.add_type(Atomic::TFloat);
        u
    }
}

pub fn extract_simple_var<'arena, 'src>(expr: &Expr<'arena, 'src>) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.as_str().trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_simple_var(inner),
        _ => None,
    }
}

pub fn extract_destructure_vars<'arena, 'src>(expr: &Expr<'arena, 'src>) -> Vec<String> {
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

pub(crate) fn ast_params_to_fn_params_resolved<'arena, 'src>(
    params: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Param<'arena, 'src>>,
    self_fqcn: Option<&str>,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<mir_codebase::FnParam> {
    params
        .iter()
        .map(|p| {
            let ty = p
                .type_hint
                .as_ref()
                .map(|h| crate::parser::type_from_hint(h, self_fqcn))
                .map(|u| resolve_named_objects_in_union(u, db, file));
            mir_codebase::FnParam {
                name: Arc::from(p.name.to_string().trim_start_matches('$')),
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
    union: Union,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Union {
    let from_docblock = union.from_docblock;
    let possibly_undefined = union.possibly_undefined;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TNamedObject { fqcn, type_params } => {
                let resolved = crate::db::resolve_name_via_db(db, file, fqcn.as_ref());
                Atomic::TNamedObject {
                    fqcn: resolved.into(),
                    type_params,
                }
            }
            other => other,
        })
        .collect();
    let mut result = Union::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

pub(crate) fn extract_string_from_expr<'arena, 'src>(expr: &Expr<'arena, 'src>) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(s) => Some(s.trim_start_matches('$').to_string()),
        ExprKind::Variable(_) => None,
        ExprKind::String(s) => Some(s.to_string()),
        _ => None,
    }
}

pub(crate) fn property_assign_compatible(
    value_ty: &Union,
    prop_ty: &Union,
    db: &dyn crate::db::MirDatabase,
) -> bool {
    if value_ty.is_subtype_of_simple(prop_ty) {
        return true;
    }
    value_ty.types.iter().all(|a| match a {
        Atomic::TNamedObject { fqcn: arg_fqcn, .. }
        | Atomic::TSelf { fqcn: arg_fqcn }
        | Atomic::TStaticObject { fqcn: arg_fqcn }
        | Atomic::TParent { fqcn: arg_fqcn } => prop_ty.types.iter().any(|p| match p {
            Atomic::TNamedObject {
                fqcn: prop_fqcn, ..
            } => {
                arg_fqcn == prop_fqcn
                    || crate::db::extends_or_implements_via_db(
                        db,
                        arg_fqcn.as_ref(),
                        prop_fqcn.as_ref(),
                    )
            }
            Atomic::TObject | Atomic::TMixed => true,
            _ => false,
        }),
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
