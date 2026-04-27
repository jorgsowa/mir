use std::sync::Arc;

use php_ast::ast::{Expr, ExprKind};
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
    pub arg_can_be_byref: &'a [bool],
    pub call_span: Span,
    pub has_spread: bool,
}

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

pub(crate) fn expr_can_be_passed_by_reference(expr: &Expr<'_, '_>) -> bool {
    matches!(
        expr.kind,
        ExprKind::Variable(_)
            | ExprKind::ArrayAccess(_)
            | ExprKind::PropertyAccess(_)
            | ExprKind::NullsafePropertyAccess(_)
            | ExprKind::StaticPropertyAccess(_)
            | ExprKind::StaticPropertyAccessDynamic { .. }
    )
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
        arg_can_be_byref,
        call_span,
        has_spread,
    } = p;

    let variadic_index = params.iter().position(|p| p.is_variadic);
    let max_positional = variadic_index.unwrap_or(params.len());
    let mut param_to_arg: Vec<Option<(Union, Span, usize)>> = vec![None; params.len()];
    let mut arg_bindings: Vec<(usize, Union, Span, usize)> = Vec::new();
    let mut positional = 0usize;
    let mut seen_named = false;
    let mut has_shape_error = false;

    for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
        if has_spread && i > 0 {
            break;
        }

        if let Some(Some(name)) = arg_names.get(i) {
            seen_named = true;
            if let Some(pi) = params.iter().position(|p| p.name.as_ref() == name.as_str()) {
                if param_to_arg[pi].is_some() {
                    has_shape_error = true;
                    ea.emit(
                        IssueKind::InvalidNamedArgument {
                            fn_name: fn_name.to_string(),
                            name: name.to_string(),
                        },
                        Severity::Error,
                        *span,
                    );
                    continue;
                }
                param_to_arg[pi] = Some((ty.clone(), *span, i));
                arg_bindings.push((pi, ty.clone(), *span, i));
            } else if let Some(vi) = variadic_index {
                arg_bindings.push((vi, ty.clone(), *span, i));
            } else {
                has_shape_error = true;
                ea.emit(
                    IssueKind::InvalidNamedArgument {
                        fn_name: fn_name.to_string(),
                        name: name.to_string(),
                    },
                    Severity::Error,
                    *span,
                );
            }
            continue;
        }

        if seen_named && !has_spread {
            has_shape_error = true;
            ea.emit(
                IssueKind::InvalidNamedArgument {
                    fn_name: fn_name.to_string(),
                    name: format!("#{}", i + 1),
                },
                Severity::Error,
                *span,
            );
            continue;
        }

        while positional < max_positional && param_to_arg[positional].is_some() {
            positional += 1;
        }

        let Some(pi) = (if positional < max_positional {
            Some(positional)
        } else {
            variadic_index
        }) else {
            continue;
        };

        if pi < max_positional {
            param_to_arg[pi] = Some((ty.clone(), *span, i));
            positional += 1;
        }
        arg_bindings.push((pi, ty.clone(), *span, i));
    }

    let required_count = params
        .iter()
        .filter(|p| !p.is_optional && !p.is_variadic)
        .count();
    let provided_count = param_to_arg
        .iter()
        .take(required_count)
        .filter(|slot| slot.is_some())
        .count();

    if provided_count < required_count && !has_spread && !has_shape_error {
        ea.emit(
            IssueKind::TooFewArguments {
                fn_name: fn_name.to_string(),
                expected: required_count,
                actual: arg_types.len(),
            },
            Severity::Error,
            call_span,
        );
    }

    if variadic_index.is_none() && arg_types.len() > params.len() && !has_spread && !has_shape_error
    {
        ea.emit(
            IssueKind::TooManyArguments {
                fn_name: fn_name.to_string(),
                expected: params.len(),
                actual: arg_types.len(),
            },
            Severity::Error,
            arg_spans.get(params.len()).copied().unwrap_or(call_span),
        );
    }

    for (param_idx, arg_ty, arg_span, arg_idx) in arg_bindings {
        let param = &params[param_idx];

        if param.is_byref && !arg_can_be_byref.get(arg_idx).copied().unwrap_or(false) {
            ea.emit(
                IssueKind::InvalidPassByReference {
                    fn_name: fn_name.to_string(),
                    param: param.name.to_string(),
                },
                Severity::Error,
                arg_span,
            );
        }

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
                    IssueKind::NullArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                    },
                    Severity::Warning,
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

            let param_accepts_false =
                param_ty.contains(|t| matches!(t, Atomic::TFalse | Atomic::TBool));
            if !param_accepts_false
                && !param_ty.is_mixed()
                && !arg_ty.is_mixed()
                && !arg_ty.is_single()
                && arg_ty.contains(|t| matches!(t, Atomic::TFalse | Atomic::TBool))
            {
                let arg_without_false = arg_ty.remove_false();
                // Strip null too: handles int|null|false → int (alongside PossiblyNullArgument)
                let arg_core = arg_without_false.remove_null();
                if !arg_core.types.is_empty()
                    && (arg_without_false.is_subtype_of_simple(param_ty)
                        || arg_core.is_subtype_of_simple(param_ty)
                        || named_object_subtype(&arg_without_false, param_ty, ea)
                        || named_object_subtype(&arg_core, param_ty, ea))
                {
                    ea.emit(
                        IssueKind::PossiblyInvalidArgument {
                            param: param.name.to_string(),
                            fn_name: fn_name.to_string(),
                            expected: format!("{param_ty}"),
                            actual: format!("{arg_ty}"),
                        },
                        Severity::Info,
                        arg_span,
                    );
                }
            }

            let arg_core = arg_ty.remove_null().remove_false();
            if !arg_ty.is_subtype_of_simple(param_ty)
                && !param_ty.is_mixed()
                && !arg_ty.is_mixed()
                && !named_object_subtype(&arg_ty, param_ty, ea)
                && !param_contains_template_or_unknown(param_ty, ea)
                && !param_contains_template_or_unknown(&arg_ty, ea)
                && !array_list_compatible(&arg_ty, param_ty, ea)
                && !(arg_ty.is_single() && param_ty.is_subtype_of_simple(&arg_ty))
                && !(arg_ty.is_single() && param_ty.remove_null().is_subtype_of_simple(&arg_ty))
                && !(arg_ty.is_single()
                    && param_ty
                        .types
                        .iter()
                        .any(|p| Union::single(p.clone()).is_subtype_of_simple(&arg_ty)))
                && !arg_ty.remove_null().is_subtype_of_simple(param_ty)
                && (arg_ty.remove_false().types.is_empty()
                    || !arg_ty.remove_false().is_subtype_of_simple(param_ty))
                && (arg_core.types.is_empty() || !arg_core.is_subtype_of_simple(param_ty))
                && !named_object_subtype(&arg_ty.remove_null(), param_ty, ea)
                && (arg_ty.remove_false().types.is_empty()
                    || !named_object_subtype(&arg_ty.remove_false(), param_ty, ea))
                && (arg_core.types.is_empty() || !named_object_subtype(&arg_core, param_ty, ea))
            {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                        expected: format!("{param_ty}"),
                        actual: invalid_argument_actual_type(&arg_ty, param_ty, ea),
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

fn invalid_argument_actual_type(
    arg_ty: &Union,
    param_ty: &Union,
    ea: &ExpressionAnalyzer<'_>,
) -> String {
    if let Some(projected) = project_generic_ancestor_type(arg_ty, param_ty, ea) {
        return format!("{projected}");
    }
    format!("{arg_ty}")
}

fn project_generic_ancestor_type(
    arg_ty: &Union,
    param_ty: &Union,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Union> {
    if !arg_ty.is_single() {
        return None;
    }
    let arg_fqcn = match arg_ty.types.first()? {
        Atomic::TNamedObject { fqcn, type_params } => {
            if !type_params.is_empty() {
                return None;
            }
            fqcn
        }
        Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } | Atomic::TParent { fqcn } => fqcn,
        _ => return None,
    };
    let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());

    for param_atomic in &param_ty.types {
        let (param_fqcn, param_type_params) = match param_atomic {
            Atomic::TNamedObject { fqcn, type_params } => (fqcn, type_params),
            _ => continue,
        };
        if param_type_params.is_empty() {
            continue;
        }

        let resolved_param = ea
            .codebase
            .resolve_class_name(&ea.file, param_fqcn.as_ref());
        let ancestor_args = generic_ancestor_type_args(arg_fqcn.as_ref(), &resolved_param, ea)
            .or_else(|| generic_ancestor_type_args(&resolved_arg, &resolved_param, ea))
            .or_else(|| generic_ancestor_type_args(arg_fqcn.as_ref(), param_fqcn.as_ref(), ea))
            .or_else(|| generic_ancestor_type_args(&resolved_arg, param_fqcn.as_ref(), ea))?;
        if ancestor_args.is_empty() {
            continue;
        }

        return Some(Union::single(Atomic::TNamedObject {
            fqcn: param_fqcn.clone(),
            type_params: ancestor_args,
        }));
    }

    None
}

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

            let arg_extends_param = ea
                .codebase
                .extends_or_implements(arg_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), param_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param);

            if arg_extends_param {
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                if !param_type_params.is_empty() {
                    let ancestor_args =
                        generic_ancestor_type_args(arg_fqcn.as_ref(), &resolved_param, ea)
                            .or_else(|| {
                                generic_ancestor_type_args(&resolved_arg, &resolved_param, ea)
                            })
                            .or_else(|| {
                                generic_ancestor_type_args(
                                    arg_fqcn.as_ref(),
                                    param_fqcn.as_ref(),
                                    ea,
                                )
                            })
                            .or_else(|| {
                                generic_ancestor_type_args(&resolved_arg, param_fqcn.as_ref(), ea)
                            });
                    if let Some(arg_as_param_params) = ancestor_args {
                        let class_tps = ea.codebase.get_class_template_params(&resolved_param);
                        return generic_type_params_compatible(
                            &arg_as_param_params,
                            param_type_params,
                            &class_tps,
                            ea,
                        );
                    }
                }
                return true;
            }

            if ea
                .codebase
                .extends_or_implements(param_fqcn.as_ref(), &resolved_arg)
                || ea
                    .codebase
                    .extends_or_implements(param_fqcn.as_ref(), arg_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_param, &resolved_arg)
            {
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                if param_type_params.is_empty() {
                    return true;
                }
            }

            if !arg_fqcn.contains('\\') && !ea.codebase.type_exists(&resolved_arg) {
                for entry in ea.codebase.classes.iter() {
                    if entry.value().short_name.as_ref() == arg_fqcn.as_ref() {
                        let actual_fqcn = entry.key().clone();
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

fn generic_ancestor_type_args(
    child: &str,
    ancestor: &str,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Vec<Union>> {
    let mut seen = std::collections::HashSet::new();
    generic_ancestor_type_args_inner(child, ancestor, ea, &mut seen)
}

fn generic_ancestor_type_args_inner(
    child: &str,
    ancestor: &str,
    ea: &ExpressionAnalyzer<'_>,
    seen: &mut std::collections::HashSet<String>,
) -> Option<Vec<Union>> {
    if child == ancestor {
        return Some(vec![]);
    }
    if !seen.insert(child.to_string()) {
        return None;
    }

    let cls = ea.codebase.classes.get(child)?;
    let parent = cls.parent.clone();
    let extends_type_args = cls.extends_type_args.clone();
    let implements_type_args = cls.implements_type_args.clone();
    drop(cls);

    for (iface, args) in implements_type_args {
        if iface.as_ref() == ancestor {
            return Some(args);
        }
    }

    let parent = parent?;
    if parent.as_ref() == ancestor {
        return Some(extends_type_args);
    }

    let parent_args = generic_ancestor_type_args_inner(parent.as_ref(), ancestor, ea, seen)?;
    if parent_args.is_empty() {
        return Some(parent_args);
    }

    let parent_template_params = ea.codebase.get_class_template_params(parent.as_ref());
    let bindings: std::collections::HashMap<Arc<str>, Union> = parent_template_params
        .iter()
        .zip(extends_type_args.iter())
        .map(|(tp, ty)| (tp.name.clone(), ty.clone()))
        .collect();

    Some(
        parent_args
            .into_iter()
            .map(|ty| ty.substitute_templates(&bindings))
            .collect(),
    )
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
