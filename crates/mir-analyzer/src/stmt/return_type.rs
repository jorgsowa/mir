use mir_types::{Atomic, Name, Type};

use crate::db::{extends_or_implements, MirDatabase};

// ---------------------------------------------------------------------------
// Named-object return type compatibility check
// ---------------------------------------------------------------------------

/// Returns true if `actual` is compatible with `declared` considering class
/// hierarchy, self/static resolution, and short-name vs FQCN mismatches.
pub(crate) fn named_object_return_compatible(
    actual: &Type,
    declared: &Type,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    actual.types.iter().all(|actual_atom| {
        // Extract the actual FQCN — handles TNamedObject, TSelf, TStaticObject, TParent
        let actual_fqcn: &Name = match actual_atom {
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
                            || extends_or_implements(db, actual_cls.as_ref(), declared_cls.as_ref())
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
            // interface-string<X> is compatible with interface-string<Y> (or
            // class-string<Y>, since every interface-string is a valid class-string)
            // if X extends/implements Y.
            Atomic::TInterfaceString(Some(actual_iface)) => {
                return declared.types.iter().any(|d| match d {
                    Atomic::TInterfaceString(None) | Atomic::TClassString(None) => true,
                    Atomic::TInterfaceString(Some(declared_iface))
                    | Atomic::TClassString(Some(declared_iface)) => {
                        actual_iface == declared_iface
                            || extends_or_implements(
                                db,
                                actual_iface.as_ref(),
                                declared_iface.as_ref(),
                            )
                    }
                    Atomic::TString => true,
                    _ => false,
                });
            }
            Atomic::TInterfaceString(None) => {
                return declared.types.iter().any(|d| {
                    matches!(
                        d,
                        Atomic::TInterfaceString(_) | Atomic::TClassString(_) | Atomic::TString
                    )
                });
            }
            // Non-object atom (scalar, array, closure, …): this function only
            // resolves the object-inheritance dimension, so check this atom
            // structurally against the declared union. Splitting the check
            // per-atom lets unions that mix objects and scalars (e.g.
            // `string|MyClass`) validate correctly — object atoms take the
            // inheritance path below, scalar atoms are decided here. (G5)
            other => return Type::single(other.clone()).is_subtype_structural(declared),
        };

        declared.types.iter().any(|declared_atom| {
            // Extract declared FQCN — also handle self/static/parent in declared type
            let declared_fqcn: &Name = match declared_atom {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };

            let resolved_declared = crate::db::resolve_name(db, file, declared_fqcn.as_ref());
            let resolved_actual = crate::db::resolve_name(db, file, actual_fqcn.as_ref());

            // Self/static always compatible with the class itself
            if matches!(
                actual_atom,
                Atomic::TSelf { .. } | Atomic::TStaticObject { .. }
            ) && (resolved_actual == resolved_declared
                    || actual_fqcn.as_ref() == declared_fqcn.as_ref()
                    || actual_fqcn.as_ref() == resolved_declared.as_str()
                    || resolved_actual.as_str() == declared_fqcn.as_ref()
                    || extends_or_implements(db, actual_fqcn.as_ref(), &resolved_declared)
                    || extends_or_implements(db, actual_fqcn.as_ref(), declared_fqcn.as_ref())
                    || extends_or_implements(db, &resolved_actual, &resolved_declared)
                    || extends_or_implements(db, &resolved_actual, declared_fqcn.as_ref())
                    // static(X) is compatible with declared Y if Y extends X
                    // (because when called on Y, static = Y which satisfies declared Y)
                    || extends_or_implements(db, &resolved_declared, actual_fqcn.as_ref())
                    || extends_or_implements(db, &resolved_declared, &resolved_actual)
                    || extends_or_implements(db, declared_fqcn.as_ref(), actual_fqcn.as_ref()))
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
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
                    _ => &[],
                };
                let declared_type_params = match declared_atom {
                    Atomic::TNamedObject { type_params, .. } => &type_params[..],
                    _ => &[],
                };
                if !actual_type_params.is_empty() || !declared_type_params.is_empty() {
                    let class_tps = crate::db::class_template_params(db, &resolved_declared)
                        .map(|tps| tps.to_vec())
                        .unwrap_or_default();
                    return return_type_params_compatible(
                        actual_type_params,
                        declared_type_params,
                        &class_tps,
                        db,
                    );
                }
                return true;
            }

            // Inheritance check
            extends_or_implements(db, actual_fqcn.as_ref(), &resolved_declared)
                || extends_or_implements(db, actual_fqcn.as_ref(), declared_fqcn.as_ref())
                || extends_or_implements(db, &resolved_actual, &resolved_declared)
                || extends_or_implements(db, &resolved_actual, declared_fqcn.as_ref())
        })
    })
}

/// Check whether generic return type parameters are compatible according to each parameter's
/// declared variance. Uses codebase-aware subtyping so user-defined class hierarchies
/// (e.g. `Box<Cat>` accepted as `Box<Animal>` when `T` is covariant) are recognized.
fn return_type_params_compatible(
    actual_params: &[Type],
    declared_params: &[Type],
    template_params: &[mir_codebase::storage::TemplateParam],
    db: &dyn MirDatabase,
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
                crate::subtype::is_subtype(db, actual_p, declared_p)
                    || declared_p.is_mixed()
                    || actual_p.is_mixed()
            }
            mir_types::Variance::Contravariant => {
                crate::subtype::is_subtype(db, declared_p, actual_p)
                    || actual_p.is_mixed()
                    || declared_p.is_mixed()
            }
            mir_types::Variance::Invariant => {
                actual_p == declared_p
                    || actual_p.is_mixed()
                    || declared_p.is_mixed()
                    || (crate::subtype::is_subtype(db, actual_p, declared_p)
                        && crate::subtype::is_subtype(db, declared_p, actual_p))
            }
        };

        if !compatible {
            return false;
        }
    }

    true
}

/// Returns true if the union recursively contains a `TTemplateParam` anywhere.
fn union_contains_template(u: &Type) -> bool {
    u.types.iter().any(|a| match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { type_params, .. } => type_params.iter().any(union_contains_template),
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            union_contains_template(key) || union_contains_template(value)
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => union_contains_template(value),
        _ => false,
    })
}

/// Returns true when the declared return type cannot be validated without full template
/// instantiation (bare template params, unknown types, or interfaces whose implementations
/// satisfy the type in ways we don't track).
///
/// Concrete generic instantiations like `Result<string, void>` are NOT bailed on — their
/// type arguments are concrete and `named_object_return_compatible` handles them.
pub(super) fn declared_return_has_template(declared: &Type, db: &dyn MirDatabase) -> bool {
    declared.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        // Skip when the named class doesn't exist in the codebase (e.g. type aliases
        // resolved to a fully-qualified name that isn't a real class).
        // Skip when the type is an interface — concrete implementations may satisfy the
        // declared type in ways we don't track (not flagged at default error level).
        // Skip when any type argument itself contains a template param — those require
        // substitution context we don't have at the return-site.
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.iter().any(union_contains_template)
                || !crate::db::class_exists(db, fqcn.as_ref())
                || crate::db::class_kind(db, fqcn.as_ref()).is_some_and(|k| k.is_interface)
        }
        Atomic::TArray { value, .. }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { value, .. }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                !fqcn.contains('\\') && !crate::db::class_exists(db, fqcn.as_ref())
            }
            _ => false,
        }),
        _ => false,
    })
}

/// Erase template parameters to their declared bounds (`@template T of Foo` → `Foo`,
/// unbounded `T` → `mixed`), recursing into generic type arguments and array/list
/// element types.
///
/// This is the substitution step that makes a template-bearing return type checkable
/// at a `return` site: a declared `@return T` (stored as `TTemplateParam { as_type }`)
/// becomes its bound, and the body-erased actual type (parameters typed `T` are already
/// resolved to their bound during analysis) can be compared against it. Returning a
/// value that is not even a subtype of the bound is a genuine error; returning a
/// subtype-of-bound stays compatible (we cannot prove it is the *specific* `T`, so we
/// stay lenient and never emit a false positive). (G1)
pub(super) fn erase_templates_to_bounds(ty: &Type) -> Type {
    let mut out = Type::empty();
    out.possibly_undefined = ty.possibly_undefined;
    out.from_docblock = ty.from_docblock;
    for atomic in &ty.types {
        match atomic {
            Atomic::TTemplateParam { as_type, .. } => {
                for b in erase_templates_to_bounds(as_type).types {
                    out.add_type(b);
                }
            }
            Atomic::TNamedObject { fqcn, type_params } => {
                let new_params: Vec<Type> =
                    type_params.iter().map(erase_templates_to_bounds).collect();
                out.add_type(Atomic::TNamedObject {
                    fqcn: *fqcn,
                    type_params: mir_types::union::vec_to_type_params(new_params),
                });
            }
            Atomic::TArray { key, value } => out.add_type(Atomic::TArray {
                key: Box::new(erase_templates_to_bounds(key)),
                value: Box::new(erase_templates_to_bounds(value)),
            }),
            Atomic::TNonEmptyArray { key, value } => out.add_type(Atomic::TNonEmptyArray {
                key: Box::new(erase_templates_to_bounds(key)),
                value: Box::new(erase_templates_to_bounds(value)),
            }),
            Atomic::TList { value } => out.add_type(Atomic::TList {
                value: Box::new(erase_templates_to_bounds(value)),
            }),
            Atomic::TNonEmptyList { value } => out.add_type(Atomic::TNonEmptyList {
                value: Box::new(erase_templates_to_bounds(value)),
            }),
            other => out.add_type(other.clone()),
        }
    }
    out
}

/// Returns true when every atom is a plain, non-parameterized object or scalar —
/// no arrays, lists, keyed arrays, or generic (`Foo<…>`) instantiations.
///
/// Gates the G1 strict recheck (see `return_type_is_invalid`): template erasure on
/// generic/collection returns loses the element-inference and empty-array special
/// cases handled elsewhere, so the strict comparison is only trusted for plain types
/// like a bare `@return T of Animal` (erased to `Animal`).
pub(super) fn is_plain_checkable(ty: &Type) -> bool {
    ty.types.iter().all(|a| match a {
        Atomic::TNamedObject { type_params, .. } => type_params.is_empty(),
        Atomic::TArray { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. }
        | Atomic::TTemplateParam { .. } => false,
        _ => true,
    })
}

/// Resolve all TNamedObject FQCNs in a Type using the codebase's file-level imports/namespace.
/// Used to fix up `@var` annotation types that were parsed without namespace context.
pub(super) fn resolve_union_for_file(union: Type, db: &dyn MirDatabase, file: &str) -> Type {
    let mut result = Type::empty();
    result.possibly_undefined = union.possibly_undefined;
    result.from_docblock = union.from_docblock;
    for atomic in union.types {
        let resolved = resolve_atomic_for_file(atomic, db, file);
        result.types.push(resolved);
    }
    result
}

fn is_resolvable_class_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '\\')
}

fn resolve_atomic_for_file(atomic: Atomic, db: &dyn MirDatabase, file: &str) -> Atomic {
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            if !is_resolvable_class_name(fqcn.as_ref()) {
                return Atomic::TNamedObject { fqcn, type_params };
            }
            let resolved = crate::db::resolve_name(db, file, fqcn.as_ref());
            if type_params.is_empty() {
                Atomic::TNamedObject {
                    fqcn: resolved.into(),
                    type_params,
                }
            } else {
                let new_params: Vec<mir_types::Type> = type_params
                    .iter()
                    .map(|p| resolve_union_for_file(p.clone(), db, file))
                    .collect();
                Atomic::TNamedObject {
                    fqcn: resolved.into(),
                    type_params: mir_types::union::vec_to_type_params(new_params),
                }
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = crate::db::resolve_name(db, file, cls.as_ref());
            Atomic::TClassString(Some(resolved.into()))
        }
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(resolve_union_for_file(*value, db, file)),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(resolve_union_for_file(*value, db, file)),
        },
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(resolve_union_for_file(*key, db, file)),
            value: Box::new(resolve_union_for_file(*value, db, file)),
        },
        Atomic::TSelf { fqcn } if fqcn.is_empty() => {
            // Sentinel from docblock parser — leave as-is; caller handles it
            Atomic::TSelf { fqcn }
        }
        other => other,
    }
}

/// Returns true when a scalar (non-object) atom in an array's value type is structurally
/// compatible with the declared value type.  Only called after the named-object and
/// class-string branches of the match have already been handled above.
fn scalar_array_element_compatible(av: &Atomic, dec_val: &Type) -> bool {
    Type::single(av.clone()).is_subtype_structural(dec_val)
}

/// Returns true if both actual and declared are array/list types whose value types are
/// compatible with FQCN resolution (to avoid short-name vs FQCN mismatches in return types).
pub(super) fn return_arrays_compatible(
    actual: &Type,
    declared: &Type,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    actual.types.iter().all(|a_atomic| {
        let act_val: &Type = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } => {
                // TKeyedArray compatibility:
                // - TKeyedArray <: TKeyedArray: permissive (returning [] from a shaped
                //   function is a common pattern not flagged at default error level).
                // - TKeyedArray <: TArray / TNonEmptyArray: check keys + values with
                //   class hierarchy awareness (structural check deferred named-object values).
                return declared.types.iter().any(|d_atomic| match d_atomic {
                    Atomic::TKeyedArray { .. } => true,
                    Atomic::TArray { key: dk, value: dv }
                    | Atomic::TNonEmptyArray { key: dk, value: dv } => {
                        if *is_open {
                            return true;
                        }
                        properties.iter().all(|(prop_key, prop)| {
                            let key_atomic = match prop_key {
                                mir_types::atomic::ArrayKey::String(s) => {
                                    Atomic::TLiteralString(s.clone())
                                }
                                mir_types::atomic::ArrayKey::Int(n) => Atomic::TLiteralInt(*n),
                            };
                            Type::single(key_atomic).is_subtype_structural(dk)
                                && (prop.ty.is_subtype_structural(dv)
                                    || named_object_return_compatible(&prop.ty, dv, db, file))
                        })
                    }
                    _ => false,
                });
            }
            _ => return false,
        };

        declared.types.iter().any(|d_atomic| {
            let dec_val: &Type = match d_atomic {
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
                                    || extends_or_implements(db, av_cls.as_ref(), dv_cls.as_ref())
                            }
                            _ => false,
                        });
                    }
                    _ => {}
                }
                let av_fqcn: &Name = match av {
                    Atomic::TNamedObject { fqcn, .. } => fqcn,
                    Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => fqcn,
                    Atomic::TClosure { .. } => return true,
                    _ => return scalar_array_element_compatible(av, dec_val),
                };
                dec_val.types.iter().any(|dv| {
                    let dv_fqcn: &Name = match dv {
                        Atomic::TNamedObject { fqcn, .. } => fqcn,
                        Atomic::TClosure { .. } => return true,
                        _ => return false,
                    };
                    if !dv_fqcn.contains('\\') && !crate::db::class_exists(db, dv_fqcn.as_ref()) {
                        return true; // template param wildcard
                    }
                    let res_dec = crate::db::resolve_name(db, file, dv_fqcn.as_ref());
                    let res_act = crate::db::resolve_name(db, file, av_fqcn.as_ref());
                    res_dec == res_act
                        || extends_or_implements(db, av_fqcn.as_ref(), &res_dec)
                        || extends_or_implements(db, &res_act, &res_dec)
                })
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Scalar return-type coercion (non-strict PHP)
// ---------------------------------------------------------------------------

/// Returns true if `actual` is a scalar type that PHP will silently coerce to
/// `declared` in non-strict mode, so `InvalidReturnType` should not fire.
///
/// Narrower than the argument-side `scalar_coercion_ok`: for return types, mir
/// deliberately keeps `int → string` and `float → string` as `InvalidReturnType`
/// even in non-strict PHP (Psalm-parity and user expectation). Only the
/// `int|false → bool` case is suppressed here, which covers the idiomatic
/// `return preg_match(...)` pattern where the PHP function is explicitly typed
/// as `int|false` but used in boolean context.
pub(crate) fn scalar_return_coercion_ok(actual: &Type, declared: &Type) -> bool {
    let declared_is_bool = declared
        .types
        .iter()
        .any(|d| matches!(d, Atomic::TBool | Atomic::TTrue | Atomic::TFalse));

    if !declared_is_bool {
        return false;
    }

    actual.types.iter().all(|a| match a {
        // int/false/bool → bool: PHP coerces 0 = false, non-zero = true
        Atomic::TInt
        | Atomic::TLiteralInt(_)
        | Atomic::TIntRange { .. }
        | Atomic::TPositiveInt
        | Atomic::TNegativeInt
        | Atomic::TNonNegativeInt
        | Atomic::TFalse
        | Atomic::TTrue
        | Atomic::TBool => true,
        _ => false,
    })
}
