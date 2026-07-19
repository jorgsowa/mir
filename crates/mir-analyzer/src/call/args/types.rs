use rustc_hash::FxHashMap;

use php_ast::Span;

use mir_codebase::definitions::TemplateParam;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;

fn type_exists(ea: &ExpressionAnalyzer<'_>, fqcn: &str) -> bool {
    crate::db::class_exists(ea.db, fqcn)
}

fn is_interface(ea: &ExpressionAnalyzer<'_>, fqcn: &str) -> bool {
    crate::db::class_kind(ea.db, fqcn).is_some_and(|k| k.is_interface)
}

fn class_template_params(
    ea: &ExpressionAnalyzer<'_>,
    fqcn: &str,
) -> Vec<mir_codebase::definitions::TemplateParam> {
    crate::db::class_template_params(ea.db, fqcn)
        .map(|tps| tps.to_vec())
        .unwrap_or_default()
}

/// Like `class_template_params`, but walks up to the nearest ancestor that
/// declares `@template` when `fqcn` itself doesn't — a bare subclass
/// (`class IntBox extends Box {}`) still inherits `Box`'s template slots
/// (and their `@template-covariant`/`-contravariant` variance), so variance
/// checking must see those, not an empty own-declarations-only list.
fn effective_class_template_params(
    ea: &ExpressionAnalyzer<'_>,
    fqcn: &str,
) -> Vec<mir_codebase::definitions::TemplateParam> {
    crate::db::effective_class_template_params(ea.db, fqcn)
        .map(|tps| tps.to_vec())
        .unwrap_or_default()
}

/// Returns true when `arg` is a structural subtype of `param` for scalar / primitive types.
/// Named-object cases (class hierarchies) are always handled separately by named_object_subtype
/// or array_list_compatible; this function is only called when those checks have already
/// been tried or when we know the types cannot be class instances.
fn scalar_arg_fits_param(arg: &Type, param: &Type) -> bool {
    arg.is_subtype_structural(param)
}

/// Returns true when `param` is structurally less specific than `arg` (a supertype),
/// meaning the call is a deliberate widening — not an error.
fn param_accepts_wider_than_arg(param: &Type, arg: &Type) -> bool {
    param.is_subtype_structural(arg)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_one(
    ea: &mut ExpressionAnalyzer<'_>,
    fn_name: &str,
    param_name: &str,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
    arg_idx: usize,
    template_params: &[TemplateParam],
) {
    // Check typed callable signature compatibility when param type is
    // `callable(T1,T2,...):R` or the equivalent `Closure(T1,T2,...):R` docblock syntax —
    // both parse to a concrete param list and must be checked the same way.
    for param_atomic in &param_ty.types {
        let (expected_params, expected_return) = match param_atomic {
            Atomic::TCallable {
                params: Some(expected_params),
                return_type,
            } => (&expected_params[..], return_type.as_deref()),
            Atomic::TClosure { data } => (&data.params[..], Some(&data.return_type)),
            _ => continue,
        };
        super::super::callable::check_typed_callable_arg(
            ea,
            fn_name,
            param_name,
            arg_ty,
            expected_params,
            expected_return,
            arg_span,
            template_params,
        );
    }

    // Validate callable and class-string arguments.
    // Skip validation for call_user_func/call_user_func_array first argument
    // since it may be a runtime callable name that doesn't exist at compile time.
    let skip_validation =
        matches!(fn_name, "call_user_func" | "call_user_func_array") && arg_idx == 0;
    if !skip_validation {
        validate_callable_argument(ea, param_ty, arg_ty, arg_span);
    }
    validate_class_string_argument(ea, param_ty, arg_ty, arg_span);
    validate_interface_string_argument(ea, param_ty, arg_ty, arg_span);
    validate_callable_type(ea, param_ty, arg_ty, arg_span);

    // A bare string literal passed where any parameter accepts `callable` is a
    // real runtime reference to the named function/method (register_shutdown_function,
    // set_error_handler, spl_autoload_register, or any user function with a `callable`
    // param) — record it so the referenced symbol isn't flagged as dead code. The
    // array_map/usort/etc-specific call sites already do this; this covers every
    // other callable-typed parameter generically. call_user_func/call_user_func_array's
    // first arg is excluded: it has its own stricter syntactic-literal-only recording
    // in function.rs (a variable that merely infers to a literal string doesn't count).
    if !skip_validation
        && param_ty.contains(|t| matches!(t, Atomic::TCallable { .. } | Atomic::TCallableString))
    {
        super::super::callable::record_callable_string_ref(ea, arg_ty, arg_span);
    }

    // Null checks run here to preserve the original emission order
    // (after callable validations but before type-compat checks).
    super::nullability::check_one(
        ea,
        fn_name,
        param_name,
        param_ty,
        arg_ty,
        arg_span,
        template_params,
    );

    // When the arg is mixed and the param expects a specific type, emit MixedArgument
    // (and skip further type checks — mixed is inherently unchecked).
    if arg_ty.is_mixed() && !param_ty.is_mixed() {
        ea.emit(
            IssueKind::MixedArgument {
                param: param_name.to_string(),
                fn_name: fn_name.to_string(),
            },
            Severity::Info,
            arg_span,
        );
        return;
    }

    let param_accepts_false = param_ty.contains(|t| matches!(t, Atomic::TFalse | Atomic::TBool));
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
            && (scalar_arg_fits_param(&arg_without_false, param_ty)
                || scalar_arg_fits_param(&arg_core, param_ty)
                || named_object_subtype(&arg_without_false, param_ty, ea)
                || named_object_subtype(&arg_core, param_ty, ea))
        {
            ea.emit(
                IssueKind::PossiblyInvalidArgument {
                    param: param_name.to_string(),
                    fn_name: fn_name.to_string(),
                    expected: format!("{param_ty}"),
                    actual: format!("{arg_ty}"),
                },
                Severity::Info,
                arg_span,
            );
        }
    }

    // TIntegralFloat (floor/ceil/round result) is always a whole-valued float, so passing it to
    // an int param is lossless — silently accept in non-strict mode with no diagnostic.
    // In strict mode PHP still rejects it (TypeError), so fall through to InvalidArgument.
    if arg_ty.contains(|t| matches!(t, Atomic::TIntegralFloat))
        && param_ty.is_single()
        && param_ty.contains(|t| t.is_int())
        && !ea.strict_types
    {
        return;
    }

    // Check for float → int implicit coercion.
    // In non-strict mode PHP silently truncates (deprecated in 8.1+), so
    // ImplicitFloatToIntCast (Warning) is the right diagnostic — InvalidArgument
    // would be a false positive because PHP accepts the call.  Return early to
    // suppress the InvalidArgument check below.
    // In strict mode PHP throws TypeError; fall through to InvalidArgument.
    if arg_ty.contains(|t| matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
        && param_ty.is_single()
        && param_ty.contains(|t| t.is_int())
        && !ea.strict_types
    {
        ea.emit(
            IssueKind::ImplicitFloatToIntCast {
                from: arg_ty.to_string(),
            },
            Severity::Warning,
            arg_span,
        );
        return;
    }

    let arg_core = arg_ty.core_type();
    if !scalar_arg_fits_param(arg_ty, param_ty)
        && !param_ty.is_mixed()
        && !arg_ty.is_mixed()
        && !named_object_subtype(arg_ty, param_ty, ea)
        && !super::param_contains_template_or_unknown(param_ty, arg_ty, ea, template_params)
        && !super::param_contains_template_or_unknown(arg_ty, arg_ty, ea, template_params)
        && !array_list_compatible(arg_ty, param_ty, ea)
        && !(arg_ty.is_single() && param_accepts_wider_than_arg(param_ty, arg_ty))
        && !(arg_ty.is_single() && param_accepts_wider_than_arg(&param_ty.remove_null(), arg_ty))
        && !(arg_ty.is_single()
            && param_ty
                .types
                .iter()
                .any(|p| param_accepts_wider_than_arg(&Type::single(p.clone()), arg_ty)))
        && !scalar_arg_fits_param(&arg_ty.remove_null(), param_ty)
        && (arg_ty.remove_false().types.is_empty()
            || !scalar_arg_fits_param(&arg_ty.remove_false(), param_ty))
        && (arg_core.types.is_empty() || !scalar_arg_fits_param(&arg_core, param_ty))
        && !named_object_subtype(&arg_ty.remove_null(), param_ty, ea)
        && (arg_ty.remove_false().types.is_empty()
            || !named_object_subtype(&arg_ty.remove_false(), param_ty, ea))
        && (arg_core.types.is_empty() || !named_object_subtype(&arg_core, param_ty, ea))
        // In PHP's coercive typing mode (no strict_types=1), an object that
        // implements \Stringable can be passed where a string is expected —
        // PHP calls __toString() implicitly. Most PHP code (including Laravel)
        // does not declare strict_types, so this is the common case.
        && !stringable_coercion_ok(arg_ty, param_ty, ea)
        // `[$obj, 'method']` / `['Class', 'method']` is a valid callable, even
        // though it types as a 2-element list shape.
        && !(param_ty.contains(|t| matches!(t, Atomic::TCallable { .. } | Atomic::TClosure { .. }))
            && super::super::callable::is_callable_array_pair(arg_ty))
    {
        // For union arg types, check if any individual atomic fits the param.
        // If some atomics fit and some don't → PossiblyInvalidArgument; otherwise → InvalidArgument.
        let any_atomic_fits = !arg_ty.is_single()
            && arg_ty.types.iter().any(|a| {
                let single = Type::single(a.clone());
                scalar_arg_fits_param(&single, param_ty)
                    || named_object_subtype(&single, param_ty, ea)
                    || array_list_compatible(&single, param_ty, ea)
                    || stringable_coercion_ok(&single, param_ty, ea)
            });
        if any_atomic_fits {
            ea.emit(
                IssueKind::PossiblyInvalidArgument {
                    param: param_name.to_string(),
                    fn_name: fn_name.to_string(),
                    expected: format!("{param_ty}"),
                    actual: format!("{arg_ty}"),
                },
                Severity::Info,
                arg_span,
            );
        } else if is_named_object_coercion(arg_ty, param_ty, ea)
            || scalar_coercion_ok(arg_ty, param_ty, ea)
        {
            ea.emit(
                IssueKind::ArgumentTypeCoercion {
                    param: param_name.to_string(),
                    fn_name: fn_name.to_string(),
                    expected: format!("{param_ty}"),
                    actual: format!("{arg_ty}"),
                },
                Severity::Info,
                arg_span,
            );
        } else {
            ea.emit(
                IssueKind::InvalidArgument {
                    param: param_name.to_string(),
                    fn_name: fn_name.to_string(),
                    expected: format!("{param_ty}"),
                    actual: invalid_argument_actual_type(arg_ty, param_ty, ea),
                },
                Severity::Error,
                arg_span,
            );
        }
    }

    // When a supertype object is passed where a subtype is expected, emit ArgumentTypeCoercion.
    // This happens when named_object_subtype returns true via the "reverse" check (param extends arg),
    // which means the call might fail at runtime if the actual object isn't the expected subtype.
    if !arg_ty.is_mixed()
        && !param_ty.is_mixed()
        && is_named_object_coercion(arg_ty, param_ty, ea)
        && !scalar_arg_fits_param(arg_ty, param_ty)
        && !array_list_compatible(arg_ty, param_ty, ea)
    {
        ea.emit(
            IssueKind::ArgumentTypeCoercion {
                param: param_name.to_string(),
                fn_name: fn_name.to_string(),
                expected: format!("{param_ty}"),
                actual: format!("{arg_ty}"),
            },
            Severity::Info,
            arg_span,
        );
    }
}

// ---------------------------------------------------------------------------
// PHP coercive-typing helpers
// ---------------------------------------------------------------------------

/// Returns `true` when passing `arg` where `string` is expected is safe via
/// PHP's coercive `__toString()` mechanism.
///
/// In PHP without `strict_types=1`, an object that implements `\Stringable`
/// (or whose class has `__toString()`) is automatically coerced to a string
/// when passed to a `string`-typed parameter. Most PHP code, including the
/// entire Laravel framework, does not declare strict_types, so this coercion
/// is the common case and should not be reported as `InvalidArgument`.
///
/// Scope: only fires when `param` contains a `string` atomic and `arg` is a
/// named-object type that implements `\Stringable`. Other parameter types
/// (int, array, …) are deliberately excluded.
fn stringable_coercion_ok(arg: &Type, param: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    use mir_types::Atomic;

    // Under strict_types=1, PHP does NOT coerce objects to string even when
    // they implement \Stringable — the runtime would throw a TypeError.
    if ea.strict_types {
        return false;
    }

    if !param.types.iter().any(|p| matches!(p, Atomic::TString)) {
        return false;
    }

    arg.types.iter().any(|a| {
        let fqcn = match a {
            Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref(),
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => fqcn.as_ref(),
            _ => return false,
        };
        let resolved = crate::db::resolve_name(ea.db, &ea.file, fqcn);
        crate::db::extends_or_implements(ea.db, &resolved, "Stringable")
            || crate::db::extends_or_implements(ea.db, fqcn, "Stringable")
            || crate::db::has_method_in_chain(ea.db, &resolved, "__toString")
            || crate::db::has_method_in_chain(ea.db, fqcn, "__toString")
    })
}

/// Returns `true` when passing `arg` where `param` is expected is a benign PHP
/// scalar coercion in non-strict mode (e.g. `int → string`).
///
/// PHP without `strict_types=1` silently coerces int/float to string at call
/// sites. These are not type errors; they should be `ArgumentTypeCoercion`
/// (Info) rather than `InvalidArgument` (Error). Bool → string is excluded
/// because passing `false` to a typed string param usually indicates a bug.
fn scalar_coercion_ok(arg: &Type, param: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    if ea.strict_types {
        return false;
    }
    // int/float → string coercion
    let param_accepts_string = param.types.iter().any(|p| {
        matches!(
            p,
            Atomic::TString | Atomic::TNonEmptyString | Atomic::TNumericString
        )
    });
    if param_accepts_string {
        return arg.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TInt
                    | Atomic::TLiteralInt(_)
                    | Atomic::TIntRange { .. }
                    | Atomic::TPositiveInt
                    | Atomic::TNegativeInt
                    | Atomic::TNonNegativeInt
                    | Atomic::TFloat
                    | Atomic::TIntegralFloat
                    | Atomic::TLiteralFloat(..)
            )
        });
    }
    false
}

// ---------------------------------------------------------------------------
// Subtype helpers
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
        // `arg_ty.is_single()` + the empty-type_params check above guarantee
        // the arg class carries no live type params of its own here.
        let ancestor_args = generic_ancestor_type_args(arg_fqcn.as_ref(), &[], &resolved_param, ea)
            .or_else(|| generic_ancestor_type_args(&resolved_arg, &[], &resolved_param, ea))
            .or_else(|| generic_ancestor_type_args(arg_fqcn.as_ref(), &[], param_fqcn.as_ref(), ea))
            .or_else(|| generic_ancestor_type_args(&resolved_arg, &[], param_fqcn.as_ref(), ea))?;
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
            Atomic::TNamedObject { fqcn, .. } => {
                // `$this` inside a trait body is typed `TNamedObject{trait}`;
                // its concrete runtime type is the unknown using class, which
                // may extend/implement the param type — treat as compatible.
                if crate::db::class_kind(ea.db, fqcn.as_ref()).is_some_and(|k| k.is_trait) {
                    return true;
                }
                fqcn
            }
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
                    // A class-string is a valid interface-string when the name it
                    // holds actually names an interface (e.g. `SomeInterface::class`
                    // types as `class-string<SomeInterface>`, not `interface-string`).
                    Atomic::TInterfaceString(None) => is_interface(ea, arg_cls.as_ref()),
                    Atomic::TInterfaceString(Some(param_cls)) => {
                        is_interface(ea, arg_cls.as_ref())
                            && (arg_cls == param_cls
                                || crate::db::extends_or_implements(
                                    ea.db,
                                    arg_cls.as_ref(),
                                    param_cls.as_ref(),
                                ))
                    }
                    _ => false,
                });
            }
            Atomic::TInterfaceString(Some(arg_cls)) => {
                return param.types.iter().any(|p| match p {
                    Atomic::TClassString(None)
                    | Atomic::TInterfaceString(None)
                    | Atomic::TString => true,
                    Atomic::TClassString(Some(param_cls))
                    | Atomic::TInterfaceString(Some(param_cls)) => {
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
            Atomic::TIntersection { parts } => {
                // An intersection of named types is an object; check if param accepts
                // `object` or if any part of the intersection satisfies the param.
                if param
                    .types
                    .iter()
                    .any(|p| matches!(p, Atomic::TObject | Atomic::TMixed))
                {
                    return true;
                }
                // If param itself requires an intersection, every required part must
                // be covered by SOME part of arg — checking any single arg part
                // against the whole param (the fallback below) wrongly rejects an
                // arg with MORE capabilities than required, e.g. passing `A&B&C`
                // where only `A&B` is needed.
                if let Some(param_parts) = param.types.iter().find_map(|p| match p {
                    Atomic::TIntersection { parts } => Some(parts),
                    _ => None,
                }) {
                    return param_parts.iter().all(|param_part| {
                        parts
                            .iter()
                            .any(|arg_part| named_object_subtype(arg_part, param_part, ea))
                    });
                }
                return parts
                    .iter()
                    .any(|part| named_object_subtype(part, param, ea));
            }
            // Bare `object` satisfies any param that accepts `object` or `mixed`.
            Atomic::TObject => {
                return param
                    .types
                    .iter()
                    .any(|p| matches!(p, Atomic::TObject | Atomic::TMixed));
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
                // Any named-object arg satisfies bare `object` or `mixed`.
                Atomic::TObject | Atomic::TMixed => return true,
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
                    let class_tps = effective_class_template_params(ea, &resolved_param);
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
                    // The arg's own declared type params (e.g. `Dog` in
                    // `TypedList<Dog>`) — needed to resolve an `@implements
                    // Collection<T>` declaration where `T` is the arg class's
                    // OWN template, not yet substituted to a concrete type.
                    let arg_own_params = match a_atomic {
                        Atomic::TNamedObject { type_params, .. } => &type_params[..],
                        _ => &[],
                    };
                    let ancestor_args = generic_ancestor_type_args(
                        arg_fqcn.as_ref(),
                        arg_own_params,
                        &resolved_param,
                        ea,
                    )
                    .or_else(|| {
                        generic_ancestor_type_args(
                            &resolved_arg,
                            arg_own_params,
                            &resolved_param,
                            ea,
                        )
                    })
                    .or_else(|| {
                        generic_ancestor_type_args(
                            arg_fqcn.as_ref(),
                            arg_own_params,
                            param_fqcn.as_ref(),
                            ea,
                        )
                    })
                    .or_else(|| {
                        generic_ancestor_type_args(
                            &resolved_arg,
                            arg_own_params,
                            param_fqcn.as_ref(),
                            ea,
                        )
                    });
                    if let Some(arg_as_param_params) = ancestor_args {
                        let class_tps = effective_class_template_params(ea, &resolved_param);
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

fn is_named_object_coercion(arg: &Type, param: &Type, ea: &ExpressionAnalyzer<'_>) -> bool {
    if !arg.is_single() {
        return false;
    }
    let arg_fqcn: &Name = match arg.types.first() {
        Some(Atomic::TNamedObject { fqcn, type_params }) if type_params.is_empty() => fqcn,
        Some(
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } | Atomic::TParent { fqcn },
        ) => fqcn,
        _ => return false,
    };
    let resolved_arg = crate::db::resolve_name(ea.db, &ea.file, arg_fqcn.as_ref());
    param.types.iter().any(|p_atomic| {
        let param_fqcn: &Name = match p_atomic {
            Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => fqcn,
            _ => return false,
        };
        let resolved_param = crate::db::resolve_name(ea.db, &ea.file, param_fqcn.as_ref());
        // param is a subtype of arg = arg is the ancestor = coercion
        crate::db::extends_or_implements(ea.db, param_fqcn.as_ref(), &resolved_arg)
            || crate::db::extends_or_implements(ea.db, param_fqcn.as_ref(), arg_fqcn.as_ref())
            || crate::db::extends_or_implements(ea.db, &resolved_param, &resolved_arg)
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
    template_params: &[mir_codebase::definitions::TemplateParam],
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
                scalar_arg_fits_param(arg_p, param_p)
                    || param_p.is_mixed()
                    || arg_p.is_mixed()
                    || strict_named_object_subtype(arg_p, param_p, ea)
            }
            mir_types::Variance::Contravariant => {
                scalar_arg_fits_param(param_p, arg_p)
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || strict_named_object_subtype(param_p, arg_p, ea)
            }
            mir_types::Variance::Invariant => {
                arg_p == param_p
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || (scalar_arg_fits_param(arg_p, param_p)
                        && scalar_arg_fits_param(param_p, arg_p))
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
    child_own_args: &[Type],
    ancestor: &str,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Vec<Type>> {
    let mut seen = rustc_hash::FxHashSet::default();
    let raw = generic_ancestor_type_args_inner(child, ancestor, ea, &mut seen)?;
    if raw.is_empty() {
        return Some(raw);
    }
    let own_tps = class_template_params(ea, child);
    if own_tps.is_empty() {
        return Some(raw);
    }
    // A `child` template param not supplied by `child_own_args` (e.g. a bare,
    // unparameterized `Generator` return with no `@return Generator<...>`
    // annotation) is effectively unbound — default it to its declared
    // `@template T = Default`, then its bound, then `mixed`, the same
    // fallback `infer_template_bindings` uses, so a bare ancestor placeholder
    // doesn't leak through unresolved and then read as a concrete mismatch
    // against the expected type.
    let own_bindings: FxHashMap<Name, Type> = own_tps
        .iter()
        .enumerate()
        .map(|(i, tp)| {
            let ty = child_own_args.get(i).cloned().unwrap_or_else(|| {
                tp.default
                    .as_deref()
                    .or(tp.bound.as_deref())
                    .cloned()
                    .unwrap_or_else(Type::mixed)
            });
            (Name::from(tp.name.as_ref()), ty)
        })
        .collect();
    Some(
        raw.into_iter()
            .map(|ty| ty.substitute_templates(&own_bindings))
            .collect(),
    )
}

fn generic_ancestor_type_args_inner(
    child: &str,
    ancestor: &str,
    ea: &ExpressionAnalyzer<'_>,
    seen: &mut rustc_hash::FxHashSet<String>,
) -> Option<Vec<Type>> {
    if child == ancestor {
        return Some(vec![]);
    }
    if !seen.insert(child.to_string()) {
        return None;
    }

    let here = crate::db::Fqcn::from_str(ea.db, child);
    let cl = crate::db::find_class_like(ea.db, here)?;

    // Resolve `ancestor`'s type args through a single typed edge — either
    // directly (`edge == ancestor`) or by recursing past it and substituting
    // `edge`'s own template params (bound by `edge_args`) into whatever the
    // recursive search found further up. Shared by every kind of typed edge
    // below (`@implements`, an interface's own `@extends`, and a class's
    // single parent `@extends`) so an ancestor declared past the FIRST hop
    // — e.g. `Collection extends GrandCollection<T>` reached through
    // `TypedList implements Collection<T>` — is not silently missed.
    let resolve_through = |edge: &str,
                           edge_args: &[Type],
                           seen: &mut rustc_hash::FxHashSet<String>|
     -> Option<Vec<Type>> {
        if edge == ancestor {
            return Some(edge_args.to_vec());
        }
        let found = generic_ancestor_type_args_inner(edge, ancestor, ea, seen)?;
        if found.is_empty() {
            return Some(found);
        }
        let edge_template_params = class_template_params(ea, edge);
        let bindings: FxHashMap<Name, Type> = edge_template_params
            .iter()
            .zip(edge_args.iter())
            .map(|(tp, ty)| (Name::from(tp.name.as_ref()), ty.clone()))
            .collect();
        Some(
            found
                .into_iter()
                .map(|ty| ty.substitute_templates(&bindings))
                .collect(),
        )
    };

    for (iface, args) in cl.implements_type_args() {
        if let Some(result) = resolve_through(iface.as_ref(), args, seen) {
            return Some(result);
        }
    }
    for (iface, args) in cl.interface_extends_type_args() {
        if let Some(result) = resolve_through(iface.as_ref(), args, seen) {
            return Some(result);
        }
    }
    if let Some(parent) = cl.parent() {
        let extends_type_args = cl.extends_type_args();
        if let Some(result) = resolve_through(parent.as_ref(), extends_type_args, seen) {
            return Some(result);
        }
    }

    None
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
            // An open shape may carry additional keys of unknown type — stay
            // permissive, matching atomic_subtype's treatment of open shapes.
            // A closed shape is only array/list-compatible when every one of its
            // property values fits the param's element type (checked per-property
            // rather than as one merged union, so a mix of a compatible and an
            // incompatible property value is correctly rejected). A TKeyedArray
            // param atom (shape-to-shape) is left permissive: `atomic_subtype` has
            // no shape-vs-shape arm at all, so this is the only path that currently
            // accepts a structurally-fine shape argument (e.g. an int literal where
            // the param property is `float`) — precise shape-vs-shape checking is
            // a separate, larger gap than the scalar/array-param one this fixes.
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } => {
                if *is_open {
                    return true;
                }
                return param_ty.types.iter().any(|pv| match pv {
                    Atomic::TArray { value, .. }
                    | Atomic::TNonEmptyArray { value, .. }
                    | Atomic::TList { value }
                    | Atomic::TNonEmptyList { value } => properties
                        .values()
                        .all(|p| union_compatible(&p.ty, value, ea)),
                    Atomic::TKeyedArray { .. } => true,
                    _ => false,
                });
            }
            _ => return scalar_arg_fits_param(&Type::single(av.clone()), param_ty),
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
            // An open shape may carry additional keys of unknown type — stay
            // permissive, matching atomic_subtype's treatment of open shapes.
            // A closed shape is only array/list-compatible when every one of its
            // property values fits the param's element type. A TKeyedArray param
            // atom (shape-to-shape) is left permissive: `atomic_subtype` has no
            // shape-vs-shape arm at all, so this is the only path that currently
            // accepts a structurally-fine shape argument (e.g. an int literal
            // where the param property is `float`) — precise shape-vs-shape
            // checking is a separate, larger gap than the one this fixes.
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } => {
                if *is_open {
                    return true;
                }
                return param_ty.types.iter().any(|p_atomic| match p_atomic {
                    Atomic::TArray { value, .. }
                    | Atomic::TNonEmptyArray { value, .. }
                    | Atomic::TList { value }
                    | Atomic::TNonEmptyList { value } => properties
                        .values()
                        .all(|p| union_compatible(&p.ty, value, ea)),
                    Atomic::TKeyedArray { .. } => true,
                    _ => false,
                });
            }
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

    // A union like `(callable(TValue): bool)|TValue|string` (Collection::max,
    // ::contains, ...) accepts a plain string through its non-callable
    // alternatives — the string is not necessarily a callable, so don't
    // validate it as a function name.
    let has_string_accepting_alternative = param_ty.types.iter().any(|t| match t {
        Atomic::TString
        | Atomic::TLiteralString(_)
        | Atomic::TNonEmptyString
        | Atomic::TNumericString
        | Atomic::TClassString(_)
        | Atomic::TMixed
        | Atomic::TScalar
        | Atomic::TTemplateParam { .. } => true,
        // A bare unresolvable name is almost certainly an unsubstituted
        // template param (e.g. `TValue`), which could be a string.
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.is_empty() && !fqcn.contains('\\') && !type_exists(ea, fqcn.as_ref())
        }
        _ => false,
    });
    if has_string_accepting_alternative {
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

    // When the parameter also accepts a plain `string` (e.g. `string|class-string<T>`),
    // a literal string satisfies the `string` alternative — don't check class existence.
    // This avoids false positives from IoC container keys like `Container::make('config')`.
    let has_plain_string = param_ty
        .types
        .iter()
        .any(|t| matches!(t, Atomic::TString | Atomic::TNonEmptyString));
    if has_plain_string {
        return;
    }

    // Check every literal-string atom, not just the first — a union arg
    // (e.g. `$cond ? 'Real' : 'Bogus'`) must have each branch validated.
    for s in arg_ty.types.iter().filter_map(|t| match t {
        Atomic::TLiteralString(s) => Some(s),
        _ => None,
    }) {
        // Skip strings that cannot be valid PHP class names (e.g. 'string[]', 'int|null').
        // A class name only contains letters, digits, underscores, and namespace separators.
        if !is_possible_class_name(s) {
            continue;
        }
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

/// Validate interface-string arguments: check that the string references an
/// existing interface (not just any class-like symbol).
fn validate_interface_string_argument(
    ea: &mut ExpressionAnalyzer<'_>,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
) {
    // Only validate if parameter is interface-string
    let has_interface_string = param_ty
        .types
        .iter()
        .any(|t| matches!(t, Atomic::TInterfaceString(_)));
    if !has_interface_string {
        return;
    }

    // When the parameter also accepts a plain `string` or bare `class-string`
    // (e.g. `class-string|interface-string<T>`), a literal string satisfies the
    // wider alternative — don't force the interface-only check on it.
    let has_wider_alternative = param_ty.types.iter().any(|t| {
        matches!(
            t,
            Atomic::TString | Atomic::TNonEmptyString | Atomic::TClassString(_)
        )
    });
    if has_wider_alternative {
        return;
    }

    // Check every literal-string atom, not just the first — see the identical
    // comment in `validate_class_string_argument`.
    for s in arg_ty.types.iter().filter_map(|t| match t {
        Atomic::TLiteralString(s) => Some(s),
        _ => None,
    }) {
        if !is_possible_class_name(s) {
            continue;
        }
        let resolved = crate::db::resolve_name(ea.db, &ea.file, s.as_ref());
        if !crate::db::class_exists(ea.db, &resolved) {
            ea.emit(
                IssueKind::UndefinedClass { name: resolved },
                Severity::Error,
                arg_span,
            );
        } else if !is_interface(ea, &resolved) {
            ea.emit(
                IssueKind::NotAnInterface { name: resolved },
                Severity::Error,
                arg_span,
            );
        }
    }
}

fn is_possible_class_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '\\' || !c.is_ascii())
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

    // A union like `callable|array|null` (Http\Client\Factory::fake) accepts
    // any array through its non-callable alternatives — don't force the
    // [object, "method"] callable shape onto it.
    let has_array_accepting_alternative = param_ty.types.iter().any(|t| match t {
        Atomic::TArray { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. }
        | Atomic::TMixed
        | Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.is_empty() && !fqcn.contains('\\') && !type_exists(ea, fqcn.as_ref())
        }
        _ => false,
    });
    if has_array_accepting_alternative {
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
                    // Get the class from the object/class reference. `[Foo::class,
                    // 'method']` evaluates the first element to TClassString, not
                    // TNamedObject — without this arm it silently skipped the
                    // UndefinedMethod check that `[$obj, 'method']` already gets.
                    for obj_atomic in &obj_prop.ty.types {
                        let resolved_class = match obj_atomic {
                            Atomic::TNamedObject { fqcn, .. } => {
                                Some(crate::db::resolve_name(ea.db, &ea.file, fqcn.as_ref()))
                            }
                            Atomic::TClassString(Some(fqcn)) => Some(fqcn.to_string()),
                            _ => None,
                        };
                        if let Some(resolved_class) = resolved_class {
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
