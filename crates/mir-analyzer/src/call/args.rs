use std::sync::Arc;

use php_ast::Span;

use mir_codebase::storage::{FnParam, MethodStorage, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};

use crate::expr::ExpressionAnalyzer;

// ---------------------------------------------------------------------------
// Public types and helpers
// ---------------------------------------------------------------------------

pub struct CheckArgsParams<'a> {
    pub fn_name: &'a str,
    pub params: &'a [FnParam],
    pub arg_types: &'a [Union],
    pub arg_spans: &'a [Span],
    pub arg_names: &'a [Option<String>],
    pub call_span: Span,
    pub has_spread: bool,
}

pub fn check_constructor_args(
    ea: &mut ExpressionAnalyzer<'_>,
    class_name: &str,
    p: CheckArgsParams<'_>,
) {
    let ctor_name = format!("{}::__construct", class_name);
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
pub fn spread_element_type(arr_ty: &Union) -> Union {
    let mut result = Union::empty();
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
            _ => return Union::mixed(),
        }
    }
    if result.types.is_empty() {
        Union::mixed()
    } else {
        result
    }
}

/// Replace `TStaticObject` / `TSelf` in a method's return type with the actual receiver FQCN.
pub(crate) fn substitute_static_in_return(ret: Union, receiver_fqcn: &Arc<str>) -> Union {
    let from_docblock = ret.from_docblock;
    let types: Vec<Atomic> = ret
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TStaticObject { .. } | Atomic::TSelf { .. } => Atomic::TNamedObject {
                fqcn: receiver_fqcn.clone(),
                type_params: vec![],
            },
            other => other,
        })
        .collect();
    let mut result = Union::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

pub(crate) fn check_method_visibility(
    ea: &mut ExpressionAnalyzer<'_>,
    method: &MethodStorage,
    ctx: &crate::context::Context,
    span: Span,
) {
    match method.visibility {
        Visibility::Private => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            let from_trait = ea.codebase.traits.contains_key(method.fqcn.as_ref());
            let allowed = caller_fqcn == method.fqcn.as_ref()
                || (from_trait
                    && ea
                        .codebase
                        .extends_or_implements(caller_fqcn, method.fqcn.as_ref()));
            if !allowed {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: method.fqcn.to_string(),
                        method: method.name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            }
        }
        Visibility::Protected => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            if caller_fqcn.is_empty() {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: method.fqcn.to_string(),
                        method: method.name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            } else {
                let allowed = caller_fqcn == method.fqcn.as_ref()
                    || ea
                        .codebase
                        .extends_or_implements(caller_fqcn, method.fqcn.as_ref());
                if !allowed {
                    ea.emit(
                        IssueKind::UndefinedMethod {
                            class: method.fqcn.to_string(),
                            method: method.name.to_string(),
                        },
                        Severity::Error,
                        span,
                    );
                }
            }
        }
        Visibility::Public => {}
    }
}

// ---------------------------------------------------------------------------
// Argument type checking
// ---------------------------------------------------------------------------

pub(crate) fn check_args(ea: &mut ExpressionAnalyzer<'_>, p: CheckArgsParams<'_>) {
    let CheckArgsParams {
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        call_span,
        has_spread,
    } = p;

    let has_named = arg_names.iter().any(|n| n.is_some());
    let mut param_to_arg: Vec<Option<(Union, Span)>> = vec![None; params.len()];

    if has_named {
        let mut positional = 0usize;
        for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
            if let Some(Some(name)) = arg_names.get(i) {
                if let Some(pi) = params.iter().position(|p| p.name.as_ref() == name.as_str()) {
                    param_to_arg[pi] = Some((ty.clone(), *span));
                }
            } else {
                while positional < params.len() && param_to_arg[positional].is_some() {
                    positional += 1;
                }
                if positional < params.len() {
                    param_to_arg[positional] = Some((ty.clone(), *span));
                    positional += 1;
                }
            }
        }
    } else {
        for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
            if i < params.len() {
                param_to_arg[i] = Some((ty.clone(), *span));
            }
        }
    }

    let required_count = params
        .iter()
        .filter(|p| !p.is_optional && !p.is_variadic)
        .count();
    let provided_count = if params.iter().any(|p| p.is_variadic) {
        arg_types.len()
    } else {
        arg_types.len().min(params.len())
    };

    if provided_count < required_count && !has_spread {
        ea.emit(
            IssueKind::InvalidArgument {
                param: format!("#{}", provided_count + 1),
                fn_name: fn_name.to_string(),
                expected: format!("{} argument(s)", required_count),
                actual: format!("{} provided", provided_count),
            },
            Severity::Error,
            call_span,
        );
        return;
    }

    for (param, slot) in params.iter().zip(param_to_arg.iter()) {
        let (arg_ty, arg_span) = match slot {
            Some(pair) => pair,
            None => continue,
        };
        let arg_span = *arg_span;

        if let Some(raw_param_ty) = &param.ty {
            let param_ty_owned;
            let param_ty: &Union = if param.is_variadic {
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

            if !param_ty.is_nullable()
                && !param_ty.is_mixed()
                && arg_ty.is_single()
                && arg_ty.contains(|t| matches!(t, Atomic::TNull))
            {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                        expected: format!("{}", param_ty),
                        actual: format!("{}", arg_ty),
                    },
                    Severity::Error,
                    arg_span,
                );
            } else if !param_ty.is_nullable() && !param_ty.is_mixed() && arg_ty.is_nullable() {
                ea.emit(
                    IssueKind::PossiblyNullArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                    },
                    Severity::Info,
                    arg_span,
                );
            }

            if !arg_ty.is_subtype_of_simple(param_ty)
                && !param_ty.is_mixed()
                && !arg_ty.is_mixed()
                && !named_object_subtype(arg_ty, param_ty, ea)
                && !param_contains_template_or_unknown(param_ty, ea)
                && !param_contains_template_or_unknown(arg_ty, ea)
                && !array_list_compatible(arg_ty, param_ty, ea)
                && !(arg_ty.is_single() && param_ty.is_subtype_of_simple(arg_ty))
                && !(arg_ty.is_single() && param_ty.remove_null().is_subtype_of_simple(arg_ty))
                && !(arg_ty.is_single()
                    && param_ty
                        .types
                        .iter()
                        .any(|p| Union::single(p.clone()).is_subtype_of_simple(arg_ty)))
                && !arg_ty.remove_null().is_subtype_of_simple(param_ty)
                && !arg_ty.remove_false().is_subtype_of_simple(param_ty)
                && !named_object_subtype(&arg_ty.remove_null(), param_ty, ea)
                && !named_object_subtype(&arg_ty.remove_false(), param_ty, ea)
            {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                        expected: format!("{}", param_ty),
                        actual: format!("{}", arg_ty),
                    },
                    Severity::Error,
                    arg_span,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Subtype helpers (private to this module)
// ---------------------------------------------------------------------------

/// Returns true if every atomic in `arg` can be assigned to some atomic in `param`
/// using codebase-aware class hierarchy checks.
fn named_object_subtype(arg: &Union, param: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg.types.iter().all(|a_atomic| {
        let arg_fqcn: &Arc<str> = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => {
                if ea.codebase.traits.contains_key(fqcn.as_ref()) {
                    return true;
                }
                fqcn
            }
            Atomic::TParent { fqcn } => fqcn,
            Atomic::TNever => return true,
            Atomic::TClosure { .. } => {
                return param.types.iter().any(|p| match p {
                    Atomic::TClosure { .. } | Atomic::TCallable { .. } => true,
                    Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref() == "Closure",
                    _ => false,
                });
            }
            Atomic::TCallable { .. } => {
                return param.types.iter().any(|p| match p {
                    Atomic::TCallable { .. } | Atomic::TClosure { .. } => true,
                    Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref() == "Closure",
                    _ => false,
                });
            }
            Atomic::TClassString(Some(arg_cls)) => {
                return param.types.iter().any(|p| match p {
                    Atomic::TClassString(None) | Atomic::TString => true,
                    Atomic::TClassString(Some(param_cls)) => {
                        arg_cls == param_cls
                            || ea
                                .codebase
                                .extends_or_implements(arg_cls.as_ref(), param_cls.as_ref())
                    }
                    _ => false,
                });
            }
            Atomic::TNull => {
                return param.types.iter().any(|p| matches!(p, Atomic::TNull));
            }
            Atomic::TFalse => {
                return param
                    .types
                    .iter()
                    .any(|p| matches!(p, Atomic::TFalse | Atomic::TBool));
            }
            _ => return false,
        };

        if param
            .types
            .iter()
            .any(|p| matches!(p, Atomic::TCallable { .. }))
        {
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());
            if ea.codebase.get_method(&resolved_arg, "__invoke").is_some()
                || ea
                    .codebase
                    .get_method(arg_fqcn.as_ref(), "__invoke")
                    .is_some()
            {
                return true;
            }
        }

        param.types.iter().any(|p_atomic| {
            let param_fqcn: &Arc<str> = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            let resolved_param = ea
                .codebase
                .resolve_class_name(&ea.file, param_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());

            let is_same_class = resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref();

            if is_same_class {
                let arg_type_params = match a_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                if !arg_type_params.is_empty() || !param_type_params.is_empty() {
                    let class_tps = ea.codebase.get_class_template_params(&resolved_param);
                    return generic_type_params_compatible(
                        arg_type_params,
                        param_type_params,
                        &class_tps,
                        ea,
                    );
                }
                return true;
            }

            if ea
                .codebase
                .extends_or_implements(arg_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), param_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(param_fqcn.as_ref(), &resolved_arg)
                || ea
                    .codebase
                    .extends_or_implements(param_fqcn.as_ref(), arg_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_param, &resolved_arg)
            {
                return true;
            }

            if !arg_fqcn.contains('\\') && !ea.codebase.type_exists(&resolved_arg) {
                if let Some(actual_fqcn) = ea.codebase.class_by_short_name.get(arg_fqcn.as_ref()) {
                    let actual_fqcn = actual_fqcn.clone();
                    if ea
                        .codebase
                        .extends_or_implements(actual_fqcn.as_ref(), &resolved_param)
                        || ea
                            .codebase
                            .extends_or_implements(actual_fqcn.as_ref(), param_fqcn.as_ref())
                    {
                        return true;
                    }
                }
            }

            let iface_key = if ea.codebase.interfaces.contains_key(arg_fqcn.as_ref()) {
                Some(arg_fqcn.as_ref())
            } else if ea.codebase.interfaces.contains_key(resolved_arg.as_str()) {
                Some(resolved_arg.as_str())
            } else {
                None
            };
            if let Some(iface_fqcn) = iface_key {
                let compatible = ea.codebase.classes.iter().any(|entry| {
                    let cls = entry.value();
                    cls.all_parents.iter().any(|p| p.as_ref() == iface_fqcn)
                        && (ea
                            .codebase
                            .extends_or_implements(entry.key().as_ref(), param_fqcn.as_ref())
                            || ea
                                .codebase
                                .extends_or_implements(entry.key().as_ref(), &resolved_param))
                });
                if compatible {
                    return true;
                }
            }

            if arg_fqcn.contains('\\')
                && !ea.codebase.type_exists(arg_fqcn.as_ref())
                && !ea.codebase.type_exists(&resolved_arg)
            {
                return true;
            }

            if param_fqcn.contains('\\')
                && !ea.codebase.type_exists(param_fqcn.as_ref())
                && !ea.codebase.type_exists(&resolved_param)
            {
                return true;
            }

            false
        })
    })
}

/// Strict subtype check for generic type parameter positions (no coercion direction).
fn strict_named_object_subtype(arg: &Union, param: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg.types.iter().all(|a_atomic| {
        let arg_fqcn: &Arc<str> = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TNever => return true,
            _ => return false,
        };
        param.types.iter().any(|p_atomic| {
            let param_fqcn: &Arc<str> = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                _ => return false,
            };
            let resolved_param = ea
                .codebase
                .resolve_class_name(&ea.file, param_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());
            resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref()
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), param_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param)
        })
    })
}

/// Check generic type parameter compatibility according to declared variance.
fn generic_type_params_compatible(
    arg_params: &[Union],
    param_params: &[Union],
    template_params: &[mir_codebase::storage::TemplateParam],
    ea: &ExpressionAnalyzer<'_>,
) -> bool {
    if arg_params.len() != param_params.len() {
        return true;
    }
    if arg_params.is_empty() {
        return true;
    }

    for (i, (arg_p, param_p)) in arg_params.iter().zip(param_params.iter()).enumerate() {
        let variance = template_params
            .get(i)
            .map(|tp| tp.variance)
            .unwrap_or(mir_types::Variance::Invariant);

        let compatible = match variance {
            mir_types::Variance::Covariant => {
                arg_p.is_subtype_of_simple(param_p)
                    || param_p.is_mixed()
                    || arg_p.is_mixed()
                    || strict_named_object_subtype(arg_p, param_p, ea)
            }
            mir_types::Variance::Contravariant => {
                param_p.is_subtype_of_simple(arg_p)
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || strict_named_object_subtype(param_p, arg_p, ea)
            }
            mir_types::Variance::Invariant => {
                arg_p == param_p
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || (arg_p.is_subtype_of_simple(param_p) && param_p.is_subtype_of_simple(arg_p))
            }
        };

        if !compatible {
            return false;
        }
    }

    true
}

fn param_contains_template_or_unknown(param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    param_ty.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, .. } => {
            !fqcn.contains('\\') && !ea.codebase.type_exists(fqcn.as_ref())
        }
        Atomic::TClassString(Some(inner)) => {
            !inner.contains('\\') && !ea.codebase.type_exists(inner.as_ref())
        }
        Atomic::TArray { key: _, value }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { key: _, value }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                !fqcn.contains('\\') && !ea.codebase.type_exists(fqcn.as_ref())
            }
            _ => false,
        }),
        _ => false,
    })
}

fn union_compatible(arg_ty: &Union, param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|av| {
        let av_fqcn: &Arc<str> = match av {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } | Atomic::TParent { fqcn } => {
                fqcn
            }
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                return param_ty.types.iter().any(|pv| {
                    let pv_val: &Union = match pv {
                        Atomic::TArray { value, .. }
                        | Atomic::TNonEmptyArray { value, .. }
                        | Atomic::TList { value }
                        | Atomic::TNonEmptyList { value } => value,
                        _ => return false,
                    };
                    union_compatible(value, pv_val, ea)
                });
            }
            Atomic::TKeyedArray { .. } => return true,
            _ => return Union::single(av.clone()).is_subtype_of_simple(param_ty),
        };

        param_ty.types.iter().any(|pv| {
            let pv_fqcn: &Arc<str> = match pv {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            if !pv_fqcn.contains('\\') && !ea.codebase.type_exists(pv_fqcn.as_ref()) {
                return true;
            }
            let resolved_param = ea.codebase.resolve_class_name(&ea.file, pv_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, av_fqcn.as_ref());
            resolved_param == resolved_arg
                || ea
                    .codebase
                    .extends_or_implements(av_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(pv_fqcn.as_ref(), &resolved_arg)
                || ea
                    .codebase
                    .extends_or_implements(&resolved_param, &resolved_arg)
        })
    })
}

fn array_list_compatible(arg_ty: &Union, param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|a_atomic| {
        let arg_value: &Union = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray { .. } => return true,
            _ => return false,
        };

        param_ty.types.iter().any(|p_atomic| {
            let param_value: &Union = match p_atomic {
                Atomic::TArray { value, .. }
                | Atomic::TNonEmptyArray { value, .. }
                | Atomic::TList { value }
                | Atomic::TNonEmptyList { value } => value,
                _ => return false,
            };

            union_compatible(arg_value, param_value, ea)
        })
    })
}
