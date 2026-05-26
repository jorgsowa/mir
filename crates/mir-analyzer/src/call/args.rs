use rustc_hash::FxHashMap;
use std::sync::Arc;

use php_ast::Span;

use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;

// Pure-db lookups, named without the `_db_or_codebase` suffix now that
// the codebase fallback is gone (S5-PR12).
fn type_exists(ea: &ExpressionAnalyzer<'_>, fqcn: &str) -> bool {
    crate::db::class_exists(ea.db, fqcn)
}

fn is_interface(ea: &ExpressionAnalyzer<'_>, fqcn: &str) -> bool {
    crate::db::class_kind(ea.db, fqcn).is_some_and(|k| k.is_interface)
}

fn class_template_params(ea: &ExpressionAnalyzer<'_>, fqcn: &str) -> Vec<TemplateParam> {
    crate::db::class_template_params(ea.db, fqcn)
        .map(|tps| tps.to_vec())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Public types and helpers
// ---------------------------------------------------------------------------

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
    match visibility {
        Visibility::Private => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            let from_trait =
                crate::db::class_kind(ea.db, owner_fqcn.as_ref()).is_some_and(|k| k.is_trait);
            let allowed = caller_fqcn == owner_fqcn.as_ref()
                || (from_trait
                    && crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref()));
            if !allowed {
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
        Visibility::Protected => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            if caller_fqcn.is_empty() {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: owner_fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            } else {
                let allowed = caller_fqcn == owner_fqcn.as_ref()
                    || crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref());
                if !allowed {
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
        }
        Visibility::Public => {}
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
        template_params,
    } = p;

    let variadic_index = params.iter().position(|p| p.is_variadic);
    let max_positional = variadic_index.unwrap_or(params.len());
    let mut param_to_arg: Vec<Option<(Type, Span, usize)>> = vec![None; params.len()];
    let mut arg_bindings: Vec<(usize, Type, Span, usize)> = Vec::new();
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

            // Check typed callable signature compatibility when param type is callable(T1,T2,...):R
            for param_atomic in &param_ty.types {
                if let Atomic::TCallable {
                    params: Some(expected_params),
                    ..
                } = param_atomic
                {
                    super::callable::check_typed_callable_arg(
                        ea,
                        &arg_ty,
                        expected_params,
                        arg_span,
                    );
                }
            }

            // Validate callable and class-string arguments
            // Skip validation for call_user_func/call_user_func_array first argument
            // since it may be a runtime callable name that doesn't exist at compile time
            let skip_validation =
                matches!(fn_name, "call_user_func" | "call_user_func_array") && arg_idx == 0;
            if !skip_validation {
                validate_callable_argument(ea, param_ty, &arg_ty, arg_span);
            }
            validate_class_string_argument(ea, param_ty, &arg_ty, arg_span);
            validate_callable_type(ea, param_ty, &arg_ty, arg_span);

            if !param_ty.is_nullable()
                && !param_ty.is_mixed()
                && !param_contains_template_or_unknown(param_ty, &arg_ty, ea, template_params)
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
            } else if !param_ty.is_nullable()
                && !param_ty.is_mixed()
                && !param_contains_template_or_unknown(param_ty, &arg_ty, ea, template_params)
                && arg_ty.is_nullable()
            {
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
                let arg_core = arg_ty.core_type();
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

            // Check for float → int implicit coercion
            if arg_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
                && param_ty.is_single()
                && param_ty.contains(|t| t.is_int())
            {
                ea.emit(
                    IssueKind::ImplicitFloatToIntCast {
                        from: arg_ty.to_string(),
                    },
                    Severity::Warning,
                    arg_span,
                );
            }

            let arg_core = arg_ty.core_type();
            if !arg_ty.is_subtype_of_simple(param_ty)
                && !param_ty.is_mixed()
                && !arg_ty.is_mixed()
                && !named_object_subtype(&arg_ty, param_ty, ea)
                && !param_contains_template_or_unknown(param_ty, &arg_ty, ea, template_params)
                && !param_contains_template_or_unknown(&arg_ty, &arg_ty, ea, template_params)
                && !array_list_compatible(&arg_ty, param_ty, ea)
                && !(arg_ty.is_single() && param_ty.is_subtype_of_simple(&arg_ty))
                && !(arg_ty.is_single() && param_ty.remove_null().is_subtype_of_simple(&arg_ty))
                && !(arg_ty.is_single()
                    && param_ty
                        .types
                        .iter()
                        .any(|p| Type::single(p.clone()).is_subtype_of_simple(&arg_ty)))
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
    arg_ty: &Type,
    param_ty: &Type,
    ea: &ExpressionAnalyzer<'_>,
) -> String {
    if let Some(projected) = project_generic_ancestor_type(arg_ty, param_ty, ea) {
        return format!("{projected}");
    }
    format!("{arg_ty}")
}

fn project_generic_ancestor_type(
    arg_ty: &Type,
    param_ty: &Type,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Type> {
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
    let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, arg_fqcn.as_ref());

    for param_atomic in &param_ty.types {
        let (param_fqcn, param_type_params) = match param_atomic {
            Atomic::TNamedObject { fqcn, type_params } => (fqcn, type_params),
            _ => continue,
        };
        if param_type_params.is_empty() {
            continue;
        }

        let resolved_param = crate::db::resolve_name(ea.db, &ea.file, param_fqcn.as_ref());
        let ancestor_args = generic_ancestor_type_args(arg_fqcn.as_ref(), &resolved_param, ea)
            .or_else(|| generic_ancestor_type_args(&resolved_arg, &resolved_param, ea))
            .or_else(|| generic_ancestor_type_args(arg_fqcn.as_ref(), param_fqcn.as_ref(), ea))
            .or_else(|| generic_ancestor_type_args(&resolved_arg, param_fqcn.as_ref(), ea))?;
        if ancestor_args.is_empty() {
            continue;
        }

        return Some(Type::single(Atomic::TNamedObject {
            fqcn: *param_fqcn,
            type_params: mir_types::union::vec_to_type_params(ancestor_args),
        }));
    }

    None
}

/// Returns true if every atomic in `arg` can be assigned to some atomic in `param`
/// using codebase-aware class hierarchy checks.
fn named_object_subtype(arg: &Type, param: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg.types.iter().all(|a_atomic| {
        let arg_fqcn: &Name = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => {
                let is_trait =
                    crate::db::class_kind(ea.db, fqcn.as_ref()).is_some_and(|k| k.is_trait);
                if is_trait {
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
                            || crate::db::extends_or_implements(
                                ea.db,
                                arg_cls.as_ref(),
                                param_cls.as_ref(),
                            )
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
            let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, arg_fqcn.as_ref());
            if crate::db::has_method_in_chain(ea.db, &resolved_arg, "__invoke")
                || crate::db::has_method_in_chain(ea.db, arg_fqcn.as_ref(), "__invoke")
            {
                return true;
            }
        }

        param.types.iter().any(|p_atomic| {
            // Handle intersection bounds: arg must satisfy every part
            if let Atomic::TIntersection { parts } = p_atomic {
                return parts.iter().all(|part| {
                    part.types.iter().any(|part_atomic| {
                        let part_fqcn = match part_atomic {
                            Atomic::TNamedObject { fqcn, .. } => fqcn,
                            _ => return false,
                        };
                        let resolved_part =
                            crate::db::resolve_name(ea.db, &ea.file, part_fqcn.as_ref());
                        crate::db::extends_or_implements(ea.db, arg_fqcn.as_ref(), &resolved_part)
                            || crate::db::extends_or_implements(
                                ea.db,
                                arg_fqcn.as_ref(),
                                part_fqcn.as_ref(),
                            )
                    })
                });
            }

            let param_fqcn: &Name = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            let resolved_param = crate::db::resolve_name(ea.db, &ea.file, param_fqcn.as_ref());
            let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, arg_fqcn.as_ref());

            let is_same_class = resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref();

            if is_same_class {
                let arg_type_params = match a_atomic {
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
                    _ => &[],
                };
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
                    _ => &[],
                };
                if !arg_type_params.is_empty() || !param_type_params.is_empty() {
                    let class_tps = class_template_params(ea, &resolved_param);
                    return generic_type_params_compatible(
                        arg_type_params,
                        param_type_params,
                        &class_tps,
                        ea,
                    );
                }
                return true;
            }

            let arg_extends_param =
                crate::db::extends_or_implements(ea.db, arg_fqcn.as_ref(), &resolved_param)
                    || crate::db::extends_or_implements(
                        ea.db,
                        arg_fqcn.as_ref(),
                        param_fqcn.as_ref(),
                    )
                    || crate::db::extends_or_implements(ea.db, &resolved_arg, &resolved_param);

            if arg_extends_param {
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
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
                        let class_tps = class_template_params(ea, &resolved_param);
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

            if crate::db::extends_or_implements(ea.db, param_fqcn.as_ref(), &resolved_arg)
                || crate::db::extends_or_implements(ea.db, param_fqcn.as_ref(), arg_fqcn.as_ref())
                || crate::db::extends_or_implements(ea.db, &resolved_param, &resolved_arg)
            {
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
                    _ => &[],
                };
                if param_type_params.is_empty() {
                    return true;
                }
            }

            if !arg_fqcn.contains('\\') && !type_exists(ea, &resolved_arg) {
                let target = arg_fqcn.as_ref();
                for fqcn in crate::db::workspace_classes(ea.db).iter() {
                    let here = crate::db::Fqcn::from_str(ea.db, fqcn.as_ref());
                    let is_class =
                        crate::db::find_class_like(ea.db, here).is_some_and(|c| c.is_class());
                    if !is_class {
                        continue;
                    }
                    let short_name = fqcn.rsplit('\\').next().unwrap_or(fqcn.as_ref());
                    if short_name == target
                        && (crate::db::extends_or_implements(ea.db, fqcn.as_ref(), &resolved_param)
                            || crate::db::extends_or_implements(
                                ea.db,
                                fqcn.as_ref(),
                                param_fqcn.as_ref(),
                            ))
                    {
                        return true;
                    }
                }
            }

            let iface_key = if is_interface(ea, arg_fqcn.as_ref()) {
                Some(arg_fqcn.as_ref())
            } else if is_interface(ea, resolved_arg.as_str()) {
                Some(resolved_arg.as_str())
            } else {
                None
            };
            if let Some(iface_fqcn) = iface_key {
                let class_fqcns: Vec<std::sync::Arc<str>> = crate::db::workspace_classes(ea.db)
                    .iter()
                    .filter(|fqcn| {
                        let here = crate::db::Fqcn::from_str(ea.db, fqcn.as_ref());
                        crate::db::find_class_like(ea.db, here).is_some_and(|c| c.is_class())
                    })
                    .cloned()
                    .collect();
                let compatible = class_fqcns.iter().any(|cls_fqcn| {
                    crate::db::extends_or_implements(ea.db, cls_fqcn.as_ref(), iface_fqcn)
                        && (crate::db::extends_or_implements(
                            ea.db,
                            cls_fqcn.as_ref(),
                            param_fqcn.as_ref(),
                        ) || crate::db::extends_or_implements(
                            ea.db,
                            cls_fqcn.as_ref(),
                            &resolved_param,
                        ))
                });
                if compatible {
                    return true;
                }
            }

            if arg_fqcn.contains('\\')
                && !type_exists(ea, arg_fqcn.as_ref())
                && !type_exists(ea, &resolved_arg)
            {
                return true;
            }

            if param_fqcn.contains('\\')
                && !type_exists(ea, param_fqcn.as_ref())
                && !type_exists(ea, &resolved_param)
            {
                return true;
            }

            false
        })
    })
}

/// Strict subtype check for generic type parameter positions (no coercion direction).
fn strict_named_object_subtype(arg: &Type, param: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg.types.iter().all(|a_atomic| {
        let arg_fqcn: &Name = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TNever => return true,
            _ => return false,
        };
        param.types.iter().any(|p_atomic| {
            let param_fqcn: &Name = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                _ => return false,
            };
            let resolved_param = crate::db::resolve_name(ea.db, &ea.file, param_fqcn.as_ref());
            let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, arg_fqcn.as_ref());
            resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref()
                || crate::db::extends_or_implements(ea.db, arg_fqcn.as_ref(), &resolved_param)
                || crate::db::extends_or_implements(ea.db, arg_fqcn.as_ref(), param_fqcn.as_ref())
                || crate::db::extends_or_implements(ea.db, &resolved_arg, &resolved_param)
        })
    })
}

/// Check generic type parameter compatibility according to declared variance.
fn generic_type_params_compatible(
    arg_params: &[Type],
    param_params: &[Type],
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
) -> Option<Vec<Type>> {
    let mut seen = std::collections::HashSet::new();
    generic_ancestor_type_args_inner(child, ancestor, ea, &mut seen)
}

fn generic_ancestor_type_args_inner(
    child: &str,
    ancestor: &str,
    ea: &ExpressionAnalyzer<'_>,
    seen: &mut std::collections::HashSet<String>,
) -> Option<Vec<Type>> {
    if child == ancestor {
        return Some(vec![]);
    }
    if !seen.insert(child.to_string()) {
        return None;
    }

    let here = crate::db::Fqcn::from_str(ea.db, child);
    let cl = crate::db::find_class_like(ea.db, here)?;
    let parent = cl.parent().cloned();
    let extends_type_args: Vec<Type> = cl.extends_type_args().to_vec();
    let implements_type_args = cl.implements_type_args();

    for (iface, args) in implements_type_args.iter() {
        if iface.as_ref() == ancestor {
            return Some(args.to_vec());
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

    let parent_template_params = class_template_params(ea, parent.as_ref());
    let bindings: FxHashMap<Name, Type> = parent_template_params
        .iter()
        .zip(extends_type_args.iter())
        .map(|(tp, ty)| (Name::from(tp.name.as_ref()), ty.clone()))
        .collect();

    Some(
        parent_args
            .into_iter()
            .map(|ty| ty.substitute_templates(&bindings))
            .collect(),
    )
}

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
            if !fqcn.contains('\\') && !type_exists(ea, fqcn.as_ref()) {
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
            !inner.contains('\\') && !type_exists(ea, inner.as_ref())
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
                !fqcn.contains('\\') && !type_exists(ea, fqcn.as_ref())
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

fn union_compatible(arg_ty: &Type, param_ty: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|av| {
        let av_fqcn: &Name = match av {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } | Atomic::TParent { fqcn } => {
                fqcn
            }
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                return param_ty.types.iter().any(|pv| {
                    let pv_val: &Type = match pv {
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
            _ => return Type::single(av.clone()).is_subtype_of_simple(param_ty),
        };

        param_ty.types.iter().any(|pv| {
            let pv_fqcn: &Name = match pv {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            if !pv_fqcn.contains('\\') && !type_exists(ea, pv_fqcn.as_ref()) {
                return true;
            }
            let resolved_param = crate::db::resolve_name(ea.db, &ea.file, pv_fqcn.as_ref());
            let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, av_fqcn.as_ref());
            resolved_param == resolved_arg
                || crate::db::extends_or_implements(ea.db, av_fqcn.as_ref(), &resolved_param)
                || crate::db::extends_or_implements(ea.db, &resolved_arg, &resolved_param)
                || crate::db::extends_or_implements(ea.db, pv_fqcn.as_ref(), &resolved_arg)
                || crate::db::extends_or_implements(ea.db, &resolved_param, &resolved_arg)
        })
    })
}

fn array_list_compatible(arg_ty: &Type, param_ty: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|a_atomic| {
        let arg_value: &Type = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray { .. } => return true,
            _ => return false,
        };

        param_ty.types.iter().any(|p_atomic| {
            let param_value: &Type = match p_atomic {
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

/// Validate callable arguments: check that string callables reference existing functions/methods
fn validate_callable_argument(
    ea: &mut ExpressionAnalyzer<'_>,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
) {
    // Only validate if parameter is callable or documented as callable-string
    if !param_ty.contains(|t| matches!(t, Atomic::TCallable { .. } | Atomic::TCallableString)) {
        return;
    }

    if let Some(Atomic::TLiteralString(s)) = arg_ty.types.first() {
        // Check for "ClassName::methodName" format
        if let Some((class_name, method_name)) = s.split_once("::") {
            let resolved_class = crate::db::resolve_name(ea.db, &ea.file, class_name);
            if !crate::db::class_exists(ea.db, &resolved_class) {
                ea.emit(
                    IssueKind::UndefinedClass {
                        name: resolved_class,
                    },
                    Severity::Error,
                    arg_span,
                );
            } else {
                // Class exists, check if method exists
                let here = crate::db::Fqcn::new(ea.db, Name::from(resolved_class.as_str()));
                if crate::db::find_method_in_chain(ea.db, here, method_name).is_none() {
                    ea.emit(
                        IssueKind::UndefinedMethod {
                            class: resolved_class.clone(),
                            method: method_name.to_string(),
                        },
                        Severity::Error,
                        arg_span,
                    );
                }
            }
        } else {
            // Check if it's a function name
            let here = crate::db::Fqcn::from_str(ea.db, s.as_ref());
            if crate::db::find_function(ea.db, here).is_none() {
                ea.emit(
                    IssueKind::UndefinedFunction {
                        name: s.to_string(),
                    },
                    Severity::Error,
                    arg_span,
                );
            }
        }
    }
}

/// Validate class-string arguments: check that string references existing classes
fn validate_class_string_argument(
    ea: &mut ExpressionAnalyzer<'_>,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
) {
    // Only validate if parameter is class-string
    let has_class_string = param_ty
        .types
        .iter()
        .any(|t| matches!(t, Atomic::TClassString(_)));
    if !has_class_string {
        return;
    }

    if let Some(Atomic::TLiteralString(s)) = arg_ty.types.first() {
        let resolved = crate::db::resolve_name(ea.db, &ea.file, s.as_ref());
        if !crate::db::class_exists(ea.db, &resolved) {
            ea.emit(
                IssueKind::UndefinedClass { name: resolved },
                Severity::Error,
                arg_span,
            );
        }
    }
}

/// Validate callable type arguments: check that arrays are in valid [obj/class, "method"] format
fn validate_callable_type(
    ea: &mut ExpressionAnalyzer<'_>,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
) {
    // Only validate if parameter expects callable
    let is_callable = param_ty.contains(|t| matches!(t, Atomic::TCallable { .. }));
    if !is_callable {
        return;
    }

    // Check if argument is a keyed array (should be [obj/class, "method"] format)
    for atomic in &arg_ty.types {
        if let Atomic::TKeyedArray { properties, .. } = atomic {
            // Valid callable arrays should have exactly 2 elements: [0] => object/class, [1] => string
            if properties.len() != 2 {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: "callback".to_string(),
                        fn_name: "callable".to_string(),
                        expected: "callable (string or [object, \"method\"])".to_string(),
                        actual: arg_ty.to_string(),
                    },
                    Severity::Error,
                    arg_span,
                );
                continue;
            }

            // Validate [$obj/class, "method"] format
            let obj_prop = properties.values().next();
            let method_prop = properties.values().nth(1);
            if let (Some(obj_prop), Some(method_prop)) = (obj_prop, method_prop) {
                // Check if second element is a string (method name)
                if let Some(Atomic::TLiteralString(method_name)) = method_prop.ty.types.first() {
                    // Get the class from the object/class reference
                    for obj_atomic in &obj_prop.ty.types {
                        if let Atomic::TNamedObject { fqcn, .. } = obj_atomic {
                            let resolved_class =
                                crate::db::resolve_name(ea.db, &ea.file, fqcn.as_ref());
                            let here =
                                crate::db::Fqcn::new(ea.db, Name::from(resolved_class.as_str()));
                            if crate::db::find_method_in_chain(ea.db, here, method_name).is_none() {
                                ea.emit(
                                    IssueKind::UndefinedMethod {
                                        class: resolved_class.clone(),
                                        method: method_name.to_string(),
                                    },
                                    Severity::Error,
                                    arg_span,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
