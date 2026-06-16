use std::sync::Arc;

use php_ast::Span;

use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;

mod counts;
mod nullability;
mod types;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

pub(crate) struct ArgBinding {
    pub(crate) param_idx: usize,
    pub(crate) arg_ty: Type,
    pub(crate) arg_span: Span,
    pub(crate) arg_idx: usize,
}

pub struct CheckArgsParams<'a> {
    pub fn_name: &'a str,
    pub params: &'a [FnParam],
    pub arg_types: &'a [Type],
    pub arg_spans: &'a [Span],
    pub arg_names: &'a [Option<String>],
    pub arg_can_be_byref: &'a [bool],
    pub call_span: Span,
    pub has_spread: bool,
    pub template_params: &'a [TemplateParam],
    /// True when the function/method is tagged `@no-named-arguments`.
    pub no_named_arguments: bool,
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

pub fn check_constructor_args(
    ea: &mut ExpressionAnalyzer<'_>,
    class_name: &str,
    p: CheckArgsParams<'_>,
) {
    let ctor_name = format!("{class_name}::__construct");
    check_args(
        ea,
        CheckArgsParams {
            fn_name: &ctor_name,
            ..p
        },
    );
}

/// For a spread (`...`) argument, return the union of value types across all array atomics.
/// E.g. `array<int, int>` → `int`, `list<string>` → `string`, `mixed` → `mixed`.
pub fn spread_element_type(arr_ty: &Type) -> Type {
    let mut result = Type::empty();
    for atomic in arr_ty.types.iter() {
        match atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                for t in value.types.iter() {
                    result.add_type(t.clone());
                }
            }
            Atomic::TKeyedArray { properties, .. } => {
                for (_key, prop) in properties.iter() {
                    for t in prop.ty.types.iter() {
                        result.add_type(t.clone());
                    }
                }
            }
            // Traversable<K, V>, Iterator<K, V>, Generator<K, V, ...> — value is param[1].
            // Only extract when the key type is int-compatible; float/other keys
            // produce undefined positional semantics so we skip (let result stay empty).
            Atomic::TNamedObject { type_params, .. } if type_params.len() >= 2 => {
                let key_is_int_compat = type_params[0].types.iter().all(|k| {
                    matches!(
                        k,
                        Atomic::TInt
                            | Atomic::TPositiveInt
                            | Atomic::TIntRange { .. }
                            | Atomic::TMixed
                    )
                });
                if key_is_int_compat {
                    for t in type_params[1].types.iter() {
                        result.add_type(t.clone());
                    }
                }
            }
            _ => return Type::mixed(),
        }
    }
    if result.types.is_empty() {
        Type::mixed()
    } else {
        result
    }
}

/// Replace `TStaticObject` / `TSelf` in a method's return type with the actual receiver FQCN.
pub(crate) fn substitute_static_in_return(ret: Type, receiver_fqcn: &Arc<str>) -> Type {
    let from_docblock = ret.from_docblock;
    let types: Vec<Atomic> = ret
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TStaticObject { .. } | Atomic::TSelf { .. } => Atomic::TNamedObject {
                fqcn: Name::from(receiver_fqcn.as_ref()),
                type_params: mir_types::union::empty_type_params(),
            },
            other => other,
        })
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

pub(crate) fn check_method_visibility(
    ea: &mut ExpressionAnalyzer<'_>,
    visibility: Visibility,
    owner_fqcn: &Arc<str>,
    method_name: &Arc<str>,
    ctx: &crate::flow_state::FlowState,
    span: Span,
) {
    let disallowed = match visibility {
        Visibility::Private => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            let from_trait =
                crate::db::class_kind(ea.db, owner_fqcn.as_ref()).is_some_and(|k| k.is_trait);
            !(caller_fqcn == owner_fqcn.as_ref()
                || (from_trait
                    && crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref())))
        }
        Visibility::Protected => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            caller_fqcn.is_empty()
                || !(caller_fqcn == owner_fqcn.as_ref()
                    || crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref()))
        }
        Visibility::Public => false,
    };
    // An inaccessible method call is dispatched to `__call` at runtime when
    // the class (chain) defines one — e.g. Laravel's Router::prefix() is
    // protected and external callers go through Macroable::__call.
    if disallowed && !crate::db::has_method_in_chain(ea.db, owner_fqcn, "__call") {
        ea.emit(
            IssueKind::UndefinedMethod {
                class: owner_fqcn.to_string(),
                method: method_name.to_string(),
            },
            Severity::Error,
            span,
        );
    }
}

pub(crate) fn expr_can_be_passed_by_reference_owned(expr: &php_ast::owned::Expr) -> bool {
    matches!(
        expr.kind,
        php_ast::owned::ExprKind::Variable(_)
            | php_ast::owned::ExprKind::ArrayAccess(_)
            | php_ast::owned::ExprKind::PropertyAccess(_)
            | php_ast::owned::ExprKind::NullsafePropertyAccess(_)
            | php_ast::owned::ExprKind::StaticPropertyAccess(_)
            | php_ast::owned::ExprKind::StaticPropertyAccessDynamic { .. }
    )
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub(crate) fn check_args(ea: &mut ExpressionAnalyzer<'_>, p: CheckArgsParams<'_>) {
    let CheckArgsParams {
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        arg_can_be_byref,
        call_span,
        has_spread,
        template_params,
        no_named_arguments,
    } = p;

    let bindings = counts::check_counts(
        ea,
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        call_span,
        has_spread,
        no_named_arguments,
    );

    for ArgBinding {
        param_idx,
        arg_ty,
        arg_span,
        arg_idx,
    } in &bindings
    {
        let param = &params[*param_idx];

        if param.is_byref && !arg_can_be_byref.get(*arg_idx).copied().unwrap_or(false) {
            ea.emit(
                IssueKind::InvalidPassByReference {
                    fn_name: fn_name.to_string(),
                    param: param.name.to_string(),
                },
                Severity::Error,
                *arg_span,
            );
        }

        if let Some(raw_param_ty) = &param.ty {
            let param_ty_owned;
            let param_ty: &Type = if param.is_variadic {
                if let Some(elem_ty) = raw_param_ty.types.iter().find_map(|a| match a {
                    Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                        Some(*value.clone())
                    }
                    _ => None,
                }) {
                    param_ty_owned = elem_ty;
                    &param_ty_owned
                } else {
                    raw_param_ty
                }
            } else {
                raw_param_ty
            };

            // types::check_one handles the full per-binding sequence: callable-sig validations,
            // null checks (via nullability::check_one), and type-compat checks.
            types::check_one(
                ea,
                fn_name,
                &param.name,
                param_ty,
                arg_ty,
                *arg_span,
                *arg_idx,
                template_params,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Shared predicate (used by types.rs and nullability.rs via super::)
// ---------------------------------------------------------------------------

fn param_contains_template_or_unknown(
    param_ty: &Type,
    arg_ty: &Type,
    ea: &ExpressionAnalyzer<'_>,
    template_params: &[TemplateParam],
) -> bool {
    let template_names: std::collections::HashSet<&str> =
        template_params.iter().map(|tp| tp.name.as_ref()).collect();

    fn has_template_param(union: &Type, template_names: &std::collections::HashSet<&str>) -> bool {
        union.types.iter().any(|atomic| match atomic {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, type_params } => {
                // Check if this name is a template parameter
                if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                    return true;
                }
                // Check nested type_params for template parameters only
                type_params
                    .iter()
                    .any(|tp| has_template_param(tp, template_names))
            }
            Atomic::TClassString(Some(inner)) => {
                !inner.contains('\\') && template_names.contains(inner.as_ref())
            }
            _ => false,
        })
    }

    param_ty.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            // Check if this name is a template parameter
            if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                return true;
            }
            // Check if this is an unknown type
            if !fqcn.contains('\\') && !crate::db::class_exists(ea.db, fqcn.as_ref()) {
                return true;
            }
            // Check nested type_params for template parameters only
            !type_params.is_empty() && has_template_param(param_ty, &template_names)
        }
        Atomic::TClassString(Some(inner)) => {
            // Check if this name is a template parameter
            if !inner.contains('\\') && template_names.contains(inner.as_ref()) {
                return true;
            }
            // Check if this is an unknown type
            !inner.contains('\\') && !crate::db::class_exists(ea.db, inner.as_ref())
        }
        Atomic::TArray { key: _, value }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { key: _, value }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                    return true;
                }
                !fqcn.contains('\\') && !crate::db::class_exists(ea.db, fqcn.as_ref())
            }
            _ => false,
        }),
        // For A&B intersections containing a template, only suppress the
        // InvalidArgument if the arg satisfies all the concrete (non-template)
        // parts. If a concrete part is violated (e.g. arg doesn't implement
        // Taggable), the error is a true positive and should still fire.
        Atomic::TIntersection { parts } => {
            let has_template = parts
                .iter()
                .any(|part| has_template_param(part, &template_names));
            if !has_template {
                return false;
            }
            // Check that every concrete (non-template) part is satisfied by arg_ty.
            parts.iter().all(|part| {
                if has_template_param(part, &template_names) {
                    return true; // template part — forgiven
                }
                // Concrete part: arg_ty must satisfy it via extends/implements.
                // Also flatten TIntersection in arg_ty (e.g. Box<string>&Taggable as arg).
                part.types.iter().all(|part_atomic| {
                    let part_fqcn = match part_atomic {
                        Atomic::TNamedObject { fqcn, .. } => fqcn,
                        _ => return true,
                    };
                    let arg_satisfies = |arg_fqcn: &Name| {
                        arg_fqcn == part_fqcn
                            || crate::db::extends_or_implements(
                                ea.db,
                                arg_fqcn.as_ref(),
                                part_fqcn.as_ref(),
                            )
                    };
                    arg_ty.types.iter().any(|arg_atomic| match arg_atomic {
                        Atomic::TNamedObject { fqcn, .. } => arg_satisfies(fqcn),
                        Atomic::TIntersection { parts: arg_parts } => arg_parts
                            .iter()
                            .any(|ap| ap.types.iter().any(|a| matches!(a, Atomic::TNamedObject { fqcn, .. } if arg_satisfies(fqcn)))),
                        _ => false,
                    })
                })
            })
        }
        _ => false,
    })
}
