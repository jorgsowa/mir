use std::sync::Arc;

use mir_codebase::Codebase;
use mir_types::{Atomic, Union};

// ---------------------------------------------------------------------------
// Named-object return type compatibility check
// ---------------------------------------------------------------------------

/// Returns true if `actual` is compatible with `declared` considering class
/// hierarchy, self/static resolution, and short-name vs FQCN mismatches.
pub(crate) fn named_object_return_compatible(
    actual: &Union,
    declared: &Union,
    codebase: &Codebase,
    file: &str,
) -> bool {
    actual.types.iter().all(|actual_atom| {
        // Extract the actual FQCN — handles TNamedObject, TSelf, TStaticObject, TParent
        let actual_fqcn: &Arc<str> = match actual_atom {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } => fqcn,
            Atomic::TStaticObject { fqcn } => fqcn,
            Atomic::TParent { fqcn } => fqcn,
            // TNull: compatible if declared also includes null
            Atomic::TNull => return declared.types.iter().any(|d| matches!(d, Atomic::TNull)),
            // TVoid: compatible with void declared
            Atomic::TVoid => {
                return declared
                    .types
                    .iter()
                    .any(|d| matches!(d, Atomic::TVoid | Atomic::TNull))
            }
            // TNever is the bottom type — compatible with anything
            Atomic::TNever => return true,
            // class-string<X> is compatible with class-string<Y> if X extends/implements Y
            Atomic::TClassString(Some(actual_cls)) => {
                return declared.types.iter().any(|d| match d {
                    Atomic::TClassString(None) => true,
                    Atomic::TClassString(Some(declared_cls)) => {
                        actual_cls == declared_cls
                            || codebase
                                .extends_or_implements(actual_cls.as_ref(), declared_cls.as_ref())
                    }
                    Atomic::TString => true,
                    _ => false,
                });
            }
            Atomic::TClassString(None) => {
                return declared
                    .types
                    .iter()
                    .any(|d| matches!(d, Atomic::TClassString(_) | Atomic::TString));
            }
            // Non-object types: not handled here (fall through to simple subtype check)
            _ => return false,
        };

        declared.types.iter().any(|declared_atom| {
            // Extract declared FQCN — also handle self/static/parent in declared type
            let declared_fqcn: &Arc<str> = match declared_atom {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };

            let resolved_declared = codebase.resolve_class_name(file, declared_fqcn.as_ref());
            let resolved_actual = codebase.resolve_class_name(file, actual_fqcn.as_ref());

            // Self/static always compatible with the class itself
            if matches!(
                actual_atom,
                Atomic::TSelf { .. } | Atomic::TStaticObject { .. }
            ) && (resolved_actual == resolved_declared
                    || actual_fqcn.as_ref() == declared_fqcn.as_ref()
                    || actual_fqcn.as_ref() == resolved_declared.as_str()
                    || resolved_actual.as_str() == declared_fqcn.as_ref()
                    || codebase.extends_or_implements(actual_fqcn.as_ref(), &resolved_declared)
                    || codebase.extends_or_implements(actual_fqcn.as_ref(), declared_fqcn.as_ref())
                    || codebase.extends_or_implements(&resolved_actual, &resolved_declared)
                    || codebase.extends_or_implements(&resolved_actual, declared_fqcn.as_ref())
                    // static(X) is compatible with declared Y if Y extends X
                    // (because when called on Y, static = Y which satisfies declared Y)
                    || codebase.extends_or_implements(&resolved_declared, actual_fqcn.as_ref())
                    || codebase.extends_or_implements(&resolved_declared, &resolved_actual)
                    || codebase.extends_or_implements(declared_fqcn.as_ref(), actual_fqcn.as_ref()))
            {
                return true;
            }

            // Same class after resolution — check generic type params with variance
            let is_same_class = resolved_actual == resolved_declared
                || actual_fqcn.as_ref() == declared_fqcn.as_ref()
                || actual_fqcn.as_ref() == resolved_declared.as_str()
                || resolved_actual.as_str() == declared_fqcn.as_ref();

            if is_same_class {
                let actual_type_params = match actual_atom {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                let declared_type_params = match declared_atom {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                if !actual_type_params.is_empty() || !declared_type_params.is_empty() {
                    let class_tps = codebase.get_class_template_params(&resolved_declared);
                    return return_type_params_compatible(
                        actual_type_params,
                        declared_type_params,
                        &class_tps,
                    );
                }
                return true;
            }

            // Inheritance check
            codebase.extends_or_implements(actual_fqcn.as_ref(), &resolved_declared)
                || codebase.extends_or_implements(actual_fqcn.as_ref(), declared_fqcn.as_ref())
                || codebase.extends_or_implements(&resolved_actual, &resolved_declared)
                || codebase.extends_or_implements(&resolved_actual, declared_fqcn.as_ref())
        })
    })
}

/// Check whether generic return type parameters are compatible according to each parameter's
/// declared variance. Simpler than the arg-checking version — uses only structural subtyping
/// since we don't have access to ExpressionAnalyzer here.
fn return_type_params_compatible(
    actual_params: &[Union],
    declared_params: &[Union],
    template_params: &[mir_codebase::storage::TemplateParam],
) -> bool {
    if actual_params.len() != declared_params.len() {
        return true;
    }
    if actual_params.is_empty() {
        return true;
    }

    for (i, (actual_p, declared_p)) in actual_params.iter().zip(declared_params.iter()).enumerate()
    {
        let variance = template_params
            .get(i)
            .map(|tp| tp.variance)
            .unwrap_or(mir_types::Variance::Invariant);

        let compatible = match variance {
            mir_types::Variance::Covariant => {
                actual_p.is_subtype_of_simple(declared_p)
                    || declared_p.is_mixed()
                    || actual_p.is_mixed()
            }
            mir_types::Variance::Contravariant => {
                declared_p.is_subtype_of_simple(actual_p)
                    || actual_p.is_mixed()
                    || declared_p.is_mixed()
            }
            mir_types::Variance::Invariant => {
                actual_p == declared_p
                    || actual_p.is_mixed()
                    || declared_p.is_mixed()
                    || (actual_p.is_subtype_of_simple(declared_p)
                        && declared_p.is_subtype_of_simple(actual_p))
            }
        };

        if !compatible {
            return false;
        }
    }

    true
}

/// Returns true if the declared return type contains template-like types (unknown FQCNs
/// without namespace separator that don't exist in the codebase) — we can't validate
/// return types against generic type parameters without full template instantiation.
pub(super) fn declared_return_has_template(declared: &Union, codebase: &Codebase) -> bool {
    declared.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        // Generic class instantiation (e.g. Result<string, void>) — skip without full template inference.
        // Also skip when the named class doesn't exist in the codebase (e.g. type aliases
        // that were resolved to a fully-qualified name but aren't real classes).
        // Also skip when the type is an interface — concrete implementations may satisfy the
        // declared type in ways we don't track (not flagged at default error level).
        Atomic::TNamedObject { fqcn, type_params } => {
            !type_params.is_empty()
                || !codebase.type_exists(fqcn.as_ref())
                || codebase.interfaces.contains_key(fqcn.as_ref())
        }
        Atomic::TArray { value, .. }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { value, .. }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                !fqcn.contains('\\') && !codebase.type_exists(fqcn.as_ref())
            }
            _ => false,
        }),
        _ => false,
    })
}

/// Resolve all TNamedObject FQCNs in a Union using the codebase's file-level imports/namespace.
/// Used to fix up `@var` annotation types that were parsed without namespace context.
pub(super) fn resolve_union_for_file(union: Union, codebase: &Codebase, file: &str) -> Union {
    let mut result = Union::empty();
    result.possibly_undefined = union.possibly_undefined;
    result.from_docblock = union.from_docblock;
    for atomic in union.types {
        let resolved = resolve_atomic_for_file(atomic, codebase, file);
        result.types.push(resolved);
    }
    result
}

fn is_resolvable_class_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '\\')
}

fn resolve_atomic_for_file(atomic: Atomic, codebase: &Codebase, file: &str) -> Atomic {
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            if !is_resolvable_class_name(fqcn.as_ref()) {
                return Atomic::TNamedObject { fqcn, type_params };
            }
            let resolved = codebase.resolve_class_name(file, fqcn.as_ref());
            Atomic::TNamedObject {
                fqcn: resolved.into(),
                type_params,
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = codebase.resolve_class_name(file, cls.as_ref());
            Atomic::TClassString(Some(resolved.into()))
        }
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(resolve_union_for_file(*key, codebase, file)),
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TSelf { fqcn } if fqcn.is_empty() => {
            // Sentinel from docblock parser — leave as-is; caller handles it
            Atomic::TSelf { fqcn }
        }
        other => other,
    }
}

/// Returns true if both actual and declared are array/list types whose value types are
/// compatible with FQCN resolution (to avoid short-name vs FQCN mismatches in return types).
pub(super) fn return_arrays_compatible(
    actual: &Union,
    declared: &Union,
    codebase: &Codebase,
    file: &str,
) -> bool {
    actual.types.iter().all(|a_atomic| {
        let act_val: &Union = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray { .. } => return true,
            _ => return false,
        };

        declared.types.iter().any(|d_atomic| {
            let dec_val: &Union = match d_atomic {
                Atomic::TArray { value, .. }
                | Atomic::TNonEmptyArray { value, .. }
                | Atomic::TList { value }
                | Atomic::TNonEmptyList { value } => value,
                _ => return false,
            };

            act_val.types.iter().all(|av| {
                match av {
                    Atomic::TNever => return true,
                    Atomic::TClassString(Some(av_cls)) => {
                        return dec_val.types.iter().any(|dv| match dv {
                            Atomic::TClassString(None) | Atomic::TString => true,
                            Atomic::TClassString(Some(dv_cls)) => {
                                av_cls == dv_cls
                                    || codebase
                                        .extends_or_implements(av_cls.as_ref(), dv_cls.as_ref())
                            }
                            _ => false,
                        });
                    }
                    _ => {}
                }
                let av_fqcn: &Arc<str> = match av {
                    Atomic::TNamedObject { fqcn, .. } => fqcn,
                    Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => fqcn,
                    Atomic::TClosure { .. } => return true,
                    _ => return Union::single(av.clone()).is_subtype_of_simple(dec_val),
                };
                dec_val.types.iter().any(|dv| {
                    let dv_fqcn: &Arc<str> = match dv {
                        Atomic::TNamedObject { fqcn, .. } => fqcn,
                        Atomic::TClosure { .. } => return true,
                        _ => return false,
                    };
                    if !dv_fqcn.contains('\\') && !codebase.type_exists(dv_fqcn.as_ref()) {
                        return true; // template param wildcard
                    }
                    let res_dec = codebase.resolve_class_name(file, dv_fqcn.as_ref());
                    let res_act = codebase.resolve_class_name(file, av_fqcn.as_ref());
                    res_dec == res_act
                        || codebase.extends_or_implements(av_fqcn.as_ref(), &res_dec)
                        || codebase.extends_or_implements(&res_act, &res_dec)
                })
            })
        })
    })
}
