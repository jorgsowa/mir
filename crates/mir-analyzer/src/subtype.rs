//! Codebase-aware subtype check.
//!
//! `mir_types::Type::is_subtype_structural` is structural only — it never walks
//! `extends` / `implements`. Within `mir-analyzer`, whenever a `db` is in
//! scope, prefer [`is_subtype`] here. It falls back to the structural check
//! for scalars and exact matches, then resolves class hierarchies through the
//! Salsa database for named-object pairs and named-object/intersection pairs.
//!
//! Callers that already combine `is_subtype_structural` with their own
//! ad-hoc inheritance check (`named_object_subtype`, `named_object_return_compatible`)
//! don't need to switch — but new call sites should reach for this function
//! first.
use std::borrow::Cow;

use rustc_hash::FxHashMap;

use mir_types::{Atomic, Name, Type, Variance};

use crate::db::{
    class_template_params, extends_or_implements, inherited_template_bindings, MirDatabase,
};

/// A supertype type-parameter that's effectively wildcarded — an unbound
/// template var or `mixed`. When the supertype's params are all free, we
/// treat the supertype as "any instantiation" for subtype matching.
pub(crate) fn sup_param_is_free(ty: &Type) -> bool {
    ty.is_mixed()
        || ty
            .types
            .iter()
            .all(|a| matches!(a, Atomic::TTemplateParam { .. }))
}

/// Per-position variance check for two parameterizations of the SAME class
/// (`Box<Dog>` vs `Box<Animal>`): a `@template-covariant`/`-contravariant`
/// param may differ in the declared direction; invariant params must match
/// by mutual subtyping (the `sub_params == sup_params` fast path in `is_subtype` already
/// covers the all-invariant case, so a mismatch here only survives when at
/// least one param is variant).
pub(crate) fn variance_compatible(
    db: &dyn MirDatabase,
    fqcn: &str,
    sub_params: &[Type],
    sup_params: &[Type],
) -> bool {
    if sub_params.len() != sup_params.len() {
        return false;
    }
    // A bare subclass that doesn't redeclare `@template` (`class IntBox
    // extends Box {}`) still carries its type args positioned against the
    // nearest ancestor that actually declares them — walk up to that
    // ancestor via `class_template_params` instead of finding zero
    // templates on `fqcn` itself. An empty result here (no template-declaring
    // ancestor at all) can't vacuously pass a caller-supplied non-empty
    // `sub_params`/`sup_params` pair (already known to differ, since the
    // `sub_params == sup_params` fast path in `is_subtype` would have short-
    // circuited otherwise) — there's no variance info to justify treating
    // them as compatible, so this must return false, not the previous
    // `tps.iter().zip(..)` vacuous-empty-iterator `true`.
    let tps = class_template_params(db, fqcn).unwrap_or_default();
    if tps.len() != sub_params.len() {
        return false;
    }
    tps.iter()
        .zip(sub_params)
        .zip(sup_params)
        .all(|((tp, sub_p), sup_p)| match tp.variance {
            Variance::Covariant => is_subtype(db, sub_p, sup_p),
            Variance::Contravariant => is_subtype(db, sup_p, sub_p),
            Variance::Invariant => {
                // `mixed` is lenient in both directions, so it never counts as equivalent.
                sub_p == sup_p
                    // A bare `array` argument (a `: array` closure return) carries no
                    // key/value info, so any array-typed parameter may refine it.
                    || (matches!(sub_p.types.as_slice(), [a] if crate::expr::helpers::is_untyped_array(a))
                        && is_subtype(db, sup_p, sub_p))
                    || (!sub_p.is_mixed()
                        && !sup_p.is_mixed()
                        && is_subtype(db, sub_p, sup_p)
                        && is_subtype(db, sup_p, sub_p))
            }
        })
}

/// Per-position variance check across an inheritance/`@implements` chain:
/// `sub_fqcn<sub_params>` may satisfy `sup_fqcn<sup_params>` — a DIFFERENT
/// class/interface — when `sup_fqcn`'s own template params are declared
/// covariant/contravariant and the type args `sub_fqcn` actually supplies for
/// `sup_fqcn` (resolved through its `@extends`/`@implements` chain) satisfy
/// them in the declared direction. Without this, `variance_compatible`'s
/// `sub_fqcn == sup_fqcn` gate only ever matches two instantiations of the
/// SAME class, so e.g. `TypedList<Dog> implements Collection<T>` could never
/// satisfy a `Collection<Animal>` parameter even though `Collection`'s `T` is
/// `@template-covariant`.
pub(crate) fn variance_compatible_across_hierarchy(
    db: &dyn MirDatabase,
    sub_fqcn: &str,
    sub_params: &[Type],
    sup_fqcn: &str,
    sup_params: &[Type],
) -> bool {
    if sub_fqcn == sup_fqcn {
        return false;
    }
    // A bare subclass that doesn't redeclare `@template` (`class IntBox
    // extends Box {}`) still carries its type args positioned against the
    // nearest ancestor that actually declares them — walk up to that
    // ancestor instead of finding zero templates on `sub_fqcn` itself and
    // discarding every bound type param (see `variance_compatible` above for
    // the same fix, and `call/method.rs`/`call/static_call.rs`/
    // `expr/objects.rs` for the established pattern elsewhere). `None`/empty
    // (no template-declaring ancestor at all — a genuinely concrete,
    // non-generic class) has no OWN bindings to contribute, but it can still
    // fix an ancestor's template argument via `@implements Collection<int>` —
    // `inherited_template_bindings` below resolves that directly from the
    // `@implements` clause, so an empty `own_bindings` is fine. Only bail
    // when the sub class DOES declare templates but the caller supplied a
    // mismatched arity — a malformed receiver, not "nothing to check".
    let sub_tps = class_template_params(db, sub_fqcn).unwrap_or_default();
    if !sub_tps.is_empty() && !sub_params.is_empty() && sub_tps.len() != sub_params.len() {
        return false;
    }
    // A bare instantiation leaves its own templates unbound, i.e. `mixed`.
    let own_bindings: FxHashMap<Name, Type> = if sub_params.is_empty() {
        sub_tps.iter().map(|tp| (tp.name, Type::mixed())).collect()
    } else {
        sub_tps
            .iter()
            .zip(sub_params)
            .map(|(tp, ty)| (tp.name, ty.clone()))
            .collect()
    };
    let ancestor_bindings = inherited_template_bindings(db, sub_fqcn, &own_bindings);
    let sup_tps = class_template_params(db, sup_fqcn).unwrap_or_default();
    let resolved_sup_params: Vec<Type> = sup_tps
        .iter()
        .map(|tp| {
            ancestor_bindings
                .get(&tp.name)
                .cloned()
                .unwrap_or_else(Type::mixed)
        })
        .collect();
    if sub_params.is_empty() && resolved_sup_params.iter().all(Type::is_mixed) {
        return true;
    }
    variance_compatible(db, sup_fqcn, &resolved_sup_params, sup_params)
}

/// Whether `sub_fqcn<sub_params>`'s type arguments are compatible with a
/// required `sup_fqcn<sup_params>` — shared by the `(TNamedObject,
/// TNamedObject)` and `(TNamedObject, TIntersection)` arms of `is_subtype`
/// (an intersection part is itself just a named-object requirement). Does
/// NOT check `sub_fqcn`/`sup_fqcn`'s class-hierarchy relationship — callers
/// combine this with their own `extends_or_implements` check.
pub(crate) fn named_object_type_params_ok(
    db: &dyn MirDatabase,
    sub_fqcn: &Name,
    sub_params: &[Type],
    sup_fqcn: &Name,
    sup_params: &[Type],
) -> bool {
    // For parameterized classes we can only reason about the hierarchy when
    // the supertype is bare (no `<...>`), the supertype's params are all
    // unbound template vars (e.g. `Base<K, V>` where `K`/`V` are free), both
    // sides match exactly, or the sub's params are free:
    // - `mixed` explicitly opts out of type-param checking (mirrors
    //   Psalm/PHPStan behaviour for `mixed` args)
    // - `never` is the bottom type and a subtype of every type
    sup_params.is_empty()
        || sub_params == sup_params
        || sup_params.iter().all(sup_param_is_free)
        || (!sub_params.is_empty() && sub_params.iter().all(|p| p.is_mixed() || p.is_never()))
        // A bare instantiation (`new Gen()` with nothing to infer from) leaves
        // every template unbound, i.e. `Gen<mixed, ...>`.
        || (sub_params.is_empty() && sub_fqcn == sup_fqcn)
        || (sub_fqcn == sup_fqcn
            && variance_compatible(db, sub_fqcn.as_ref(), sub_params, sup_params))
        || variance_compatible_across_hierarchy(
            db,
            sub_fqcn.as_ref(),
            sub_params,
            sup_fqcn.as_ref(),
            sup_params,
        )
}

/// `is_subtype`, but a template atom is judged by its bound (an unbound one is `mixed`).
fn shape_value_fits(db: &dyn MirDatabase, sub: &Type, sup: &Type) -> bool {
    is_subtype(db, sub, sup)
        || (sub.contains(|a| matches!(a, Atomic::TTemplateParam { .. }))
            && sub.types.iter().all(|a| match a {
                Atomic::TTemplateParam { as_type, .. } => shape_value_fits(db, as_type, sup),
                _ => is_subtype(db, &Type::single(a.clone()), sup),
            }))
}

/// Widest bounded `int<min, max>` expanded into literal ints for subtype checks.
const MAX_EXPANDED_INT_RANGE_SPAN: i64 = 16;

/// Rewrites small bounded int ranges as literal ints so `int<-1, 1>` fits `-1|0|1`.
fn expand_small_int_ranges(ty: &Type) -> Option<Type> {
    let small = |a: &Atomic| match a {
        Atomic::TIntRange {
            min: Some(lo),
            max: Some(hi),
        } => hi
            .checked_sub(*lo)
            .is_some_and(|d| (0..MAX_EXPANDED_INT_RANGE_SPAN).contains(&d)),
        _ => false,
    };
    if !ty.types.iter().any(small) {
        return None;
    }
    let mut out = Type::empty();
    for a in &ty.types {
        match a {
            Atomic::TIntRange {
                min: Some(lo),
                max: Some(hi),
            } if small(a) => (*lo..=*hi).for_each(|n| out.add_type(Atomic::TLiteralInt(n))),
            _ => out.add_type(a.clone()),
        }
    }
    Some(out)
}

/// Returns true if `sub` is a subtype of `sup`, considering the codebase's
/// class-hierarchy graph (`extends` / `implements`) on top of structural
/// matches.
pub(crate) fn is_subtype(db: &dyn MirDatabase, sub: &Type, sup: &Type) -> bool {
    if sub.is_subtype_structural(sup) {
        return true;
    }
    let (sub_norm, sup_norm) = (canonical_enum_cases(db, sub), canonical_enum_cases(db, sup));
    if let (Cow::Owned(_), _) | (_, Cow::Owned(_)) = (&sub_norm, &sup_norm) {
        return is_subtype(db, &sub_norm, &sup_norm);
    }
    if sup.is_mixed() {
        return true;
    }
    if sub.is_never() {
        return true;
    }
    if let Some(expanded) = expand_small_int_ranges(sub) {
        return is_subtype(db, &expanded, sup);
    }

    sub.types.iter().all(|a| {
        // A trait-typed value only arises as `$this` inside a trait body
        // (analyzed standalone). Its concrete runtime type is the unknown using
        // class, which may extend/implement anything — so treat it as a subtype
        // of any target rather than rejecting it against the trait's own (empty)
        // hierarchy.
        if let Atomic::TNamedObject { fqcn: sub_fqcn, .. } = a {
            if crate::db::class_kind(db, sub_fqcn.as_ref()).is_some_and(|k| k.is_trait) {
                return true;
            }
        }
        sup.types.iter().any(|b| {
            // Per-pair structural check: handles scalars (string, int, etc.) when
            // sub is a union — is_subtype_structural above failed because another
            // arm didn't match structurally, but this pair may still match.
            if mir_types::union::atomic_subtype(a, b) {
                return true;
            }
            match (a, b) {
                (
                    Atomic::TNamedObject {
                        fqcn: sub_fqcn,
                        type_params: sub_params,
                    },
                    Atomic::TNamedObject {
                        fqcn: sup_fqcn,
                        type_params: sup_params,
                    },
                ) => {
                    named_object_type_params_ok(db, sub_fqcn, sub_params, sup_fqcn, sup_params)
                        && extends_or_implements(db, sub_fqcn.as_ref(), sup_fqcn.as_ref())
                }
                // An invokable object (one with an __invoke() method) satisfies a
                // callable(...): R / Closure(...): R target — mirrors the same
                // leniency `call/args/types.rs::named_object_subtype` already
                // applies for callable-typed ARGUMENTS via `has_method_in_chain`,
                // just not signature-checked here either (matching that existing
                // precedent, not a stricter new bar).
                (
                    Atomic::TNamedObject { fqcn: sub_fqcn, .. },
                    Atomic::TCallable { .. } | Atomic::TClosure { .. },
                ) => crate::db::has_method_in_chain(db, sub_fqcn.as_ref(), "__invoke"),
                (
                    Atomic::TNamedObject {
                        fqcn: sub_fqcn,
                        type_params: sub_params,
                    },
                    Atomic::TIntersection { parts },
                ) => {
                    // sub satisfies intersection bound iff it satisfies every part —
                    // same type-param variance check as the plain (TNamedObject,
                    // TNamedObject) arm above, applied per intersection part instead
                    // of dropped: `Collection<int>&Countable` must reject a
                    // `Collection<string>&Countable` sub just as strictly as a bare
                    // `Collection<int>` supertype would.
                    parts.iter().all(|part| {
                        part.types.iter().any(|part_atomic| match part_atomic {
                            Atomic::TNamedObject {
                                fqcn: part_fqcn,
                                type_params: part_params,
                            } => {
                                named_object_type_params_ok(
                                    db,
                                    sub_fqcn,
                                    sub_params,
                                    part_fqcn,
                                    part_params,
                                ) && extends_or_implements(
                                    db,
                                    sub_fqcn.as_ref(),
                                    part_fqcn.as_ref(),
                                )
                            }
                            _ => false,
                        })
                    })
                }
                // TKeyedArray (array shape) satisfies TIntersection iff it satisfies every
                // intersection part. This mirrors the atomic_subtype fix and handles
                // the Psalm idiom of using intersection types for "shape plus extra keys
                // allowed" patterns like:
                // @psalm-type Context = array<string,mixed> & array{actor:...,target?:...,outcome:...}
                (
                    Atomic::TKeyedArray {
                        properties,
                        is_open,
                        ..
                    },
                    Atomic::TIntersection { parts },
                ) => parts.iter().all(|part| {
                    part.types.iter().any(|part_atomic| match part_atomic {
                        Atomic::TKeyedArray {
                            properties: sup_props,
                            is_open: sup_open,
                            ..
                        } => {
                            let keys_satisfied =
                                sup_props
                                    .iter()
                                    .all(|(key, sup_prop)| match properties.get(key) {
                                        Some(sub_prop) => {
                                            if !sup_prop.optional && sub_prop.optional {
                                                return false;
                                            }
                                            let has_named_obj = sup_prop.ty.types.iter().any(|a| {
                                                matches!(
                                                    a,
                                                    Atomic::TNamedObject { .. }
                                                        | Atomic::TSelf { .. }
                                                        | Atomic::TStaticObject { .. }
                                                        | Atomic::TClosure { .. }
                                                        | Atomic::TTemplateParam { .. }
                                                )
                                            });
                                            has_named_obj
                                                || shape_value_fits(db, &sub_prop.ty, &sup_prop.ty)
                                        }
                                        None => *is_open || sup_prop.optional,
                                    });
                            let has_array_part = parts.iter().any(|part| {
                                part.types
                                    .iter()
                                    .any(|t| matches!(t, Atomic::TArray { .. }))
                            });
                            let has_keyed_array_part = parts.iter().any(|part| {
                                part.types
                                    .iter()
                                    .any(|t| matches!(t, Atomic::TKeyedArray { .. }))
                            });
                            let keys_allowed_by_some_part = if has_array_part {
                                true
                            } else if has_keyed_array_part {
                                properties.keys().all(|k| {
                                    parts.iter().any(|part| {
                                        part.types.iter().any(|t| {
                                            if let Atomic::TKeyedArray {
                                                properties: part_props,
                                                ..
                                            } = t
                                            {
                                                part_props.contains_key(k)
                                            } else {
                                                false
                                            }
                                        })
                                    })
                                })
                            } else {
                                true
                            };
                            let no_undeclared_extras = *sup_open || keys_allowed_by_some_part;
                            keys_satisfied && no_undeclared_extras
                        }
                        Atomic::TArray { key, value } => {
                            properties.iter().all(|(prop_key, prop)| {
                                let key_atomic = match prop_key {
                                    mir_types::atomic::ArrayKey::String(s) => {
                                        Atomic::TLiteralString(s.clone())
                                    }
                                    mir_types::atomic::ArrayKey::Int(n) => Atomic::TLiteralInt(*n),
                                };
                                if !is_subtype(db, &Type::single(key_atomic), key) {
                                    return false;
                                }
                                let has_named_obj = prop.ty.types.iter().any(|a| {
                                    matches!(
                                        a,
                                        Atomic::TNamedObject { .. }
                                            | Atomic::TSelf { .. }
                                            | Atomic::TStaticObject { .. }
                                            | Atomic::TClosure { .. }
                                            | Atomic::TTemplateParam { .. }
                                    )
                                });
                                has_named_obj || is_subtype(db, &prop.ty, value)
                            })
                        }
                        Atomic::TNamedObject { .. }
                        | Atomic::TSelf { .. }
                        | Atomic::TStaticObject { .. }
                        | Atomic::TClosure { .. }
                        | Atomic::TTemplateParam { .. } => true,
                        _ => false,
                    })
                }),
                // A&B&C satisfies a required X&Y iff every required part is
                // covered by some part of sub — a value with MORE capabilities
                // than required still satisfies the narrower requirement.
                (
                    Atomic::TIntersection { parts: sub_parts },
                    Atomic::TIntersection { parts: sup_parts },
                ) => sup_parts.iter().all(|sup_part| {
                    sub_parts
                        .iter()
                        .any(|sub_part| is_subtype(db, sub_part, sup_part))
                }),
                // An intersection type is a subtype of C if any of its parts is a subtype of C
                // (a value satisfying A&B is also an A and also a B).
                (Atomic::TIntersection { parts }, b) => {
                    let sup_single = Type::single(b.clone());
                    parts.iter().any(|part| is_subtype(db, part, &sup_single))
                }
                // A list-shaped keyed array (array{0:A,1:B}) satisfies list<T> when every
                // element is a subtype of T — using the codebase-aware check so subclasses
                // (CommandArgument extends Argument) are accepted.
                (
                    Atomic::TKeyedArray {
                        properties,
                        is_list,
                        ..
                    },
                    Atomic::TList { value: lv },
                ) => *is_list && properties.values().all(|p| is_subtype(db, &p.ty, lv)),
                (
                    Atomic::TKeyedArray {
                        properties,
                        is_list,
                        ..
                    },
                    Atomic::TNonEmptyList { value: lv },
                ) => {
                    *is_list
                        && properties.values().any(|p| !p.optional)
                        && properties.values().all(|p| is_subtype(db, &p.ty, lv))
                }
                // array<K1,V1>/non-empty-array<K1,V1> satisfies array<K2,V2> when the key
                // and value types do (codebase-aware, so array<int,Cat> satisfies
                // array<int,Animal>) — mir_types::union::atomic_subtype's structural
                // check has no class-hierarchy awareness for these pairs at all.
                (
                    Atomic::TArray { key: sk, value: sv }
                    | Atomic::TNonEmptyArray { key: sk, value: sv },
                    Atomic::TArray { key: dk, value: dv },
                ) => is_subtype(db, sk, dk) && is_subtype(db, sv, dv),
                (
                    Atomic::TNonEmptyArray { key: sk, value: sv },
                    Atomic::TNonEmptyArray { key: dk, value: dv },
                ) => is_subtype(db, sk, dk) && is_subtype(db, sv, dv),
                // list<V1>/non-empty-list<V1> satisfies list<V2> the same way.
                (
                    Atomic::TList { value: sv } | Atomic::TNonEmptyList { value: sv },
                    Atomic::TList { value: dv },
                ) => is_subtype(db, sv, dv),
                (Atomic::TNonEmptyList { value: sv }, Atomic::TNonEmptyList { value: dv }) => {
                    is_subtype(db, sv, dv)
                }
                // PHP implicitly coerces int to float in all numeric contexts.
                (
                    Atomic::TInt
                    | Atomic::TLiteralInt(_)
                    | Atomic::TPositiveInt
                    | Atomic::TNegativeInt
                    | Atomic::TNonNegativeInt
                    | Atomic::TIntRange { .. },
                    Atomic::TFloat,
                ) => true,
                (Atomic::TIntegralFloat, Atomic::TFloat) => true,
                // class-string<X> is a subtype of class-string<Y> (or interface-string<Y>,
                // provided X actually names an interface) when X extends/implements Y —
                // structural equality alone (checked above) misses the inheritance case.
                (Atomic::TClassString(Some(sub_cls)), Atomic::TClassString(Some(sup_cls))) => {
                    sub_cls == sup_cls
                        || extends_or_implements(db, sub_cls.as_ref(), sup_cls.as_ref())
                }
                (Atomic::TClassString(Some(sub_cls)), Atomic::TInterfaceString(None)) => {
                    is_interface(db, sub_cls.as_ref())
                }
                (
                    Atomic::TClassString(Some(sub_cls)),
                    Atomic::TInterfaceString(Some(sup_iface)),
                ) => {
                    is_interface(db, sub_cls.as_ref())
                        && (sub_cls == sup_iface
                            || extends_or_implements(db, sub_cls.as_ref(), sup_iface.as_ref()))
                }
                // An unresolved class-string could name an interface — stay permissive
                // rather than definitely reject, matching the None-vs-Some convention above.
                (Atomic::TClassString(None), Atomic::TInterfaceString(_)) => true,
                (
                    Atomic::TInterfaceString(Some(sub_iface)),
                    Atomic::TInterfaceString(Some(sup_iface)),
                ) => {
                    sub_iface == sup_iface
                        || extends_or_implements(db, sub_iface.as_ref(), sup_iface.as_ref())
                }
                (
                    Atomic::TInterfaceString(Some(sub_iface)),
                    Atomic::TClassString(Some(sup_cls)),
                ) => {
                    sub_iface == sup_cls
                        || extends_or_implements(db, sub_iface.as_ref(), sup_cls.as_ref())
                }
                // An enum-case literal satisfies a supertype named by any interface
                // (or, since enums can't extend a class, any other named type) its
                // declaring enum implements — e.g. a `HasLabel`-typed param/property
                // accepting `Status::Active` when `Status implements HasLabel`.
                // Exact-fqcn equality is already covered by the structural check above.
                (
                    Atomic::TLiteralEnumCase { enum_fqcn, .. },
                    Atomic::TNamedObject { fqcn: sup_fqcn, .. },
                ) => extends_or_implements(db, enum_fqcn.as_ref(), sup_fqcn.as_ref()),
                _ => false,
            }
        })
    })
}

fn is_interface(db: &dyn MirDatabase, fqcn: &str) -> bool {
    crate::db::class_kind(db, fqcn).is_some_and(|k| k.is_interface)
}

fn atomic_mentions_enum_case(a: &Atomic) -> bool {
    match a {
        Atomic::TLiteralEnumCase { .. } | Atomic::TValueOf { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            fqcn.contains("::") || type_params.iter().any(mentions_enum_case)
        }
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            mentions_enum_case(key) || mentions_enum_case(value)
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => mentions_enum_case(value),
        Atomic::TKeyedArray { properties, .. } => {
            properties.values().any(|p| mentions_enum_case(&p.ty))
        }
        _ => false,
    }
}

/// A single case of `E` fits an invariant `E` slot: an inferred `Box<E::Case>` stays usable where
/// `Box<E>` is declared, as it was when case expressions were typed as the bare enum.
pub(crate) fn single_case_fits_bare_enum(actual: &Type, declared: &Type) -> bool {
    match (actual.types.as_slice(), declared.types.as_slice()) {
        (
            [Atomic::TLiteralEnumCase { enum_fqcn, .. }],
            [Atomic::TNamedObject { fqcn, type_params }],
        ) => type_params.is_empty() && fqcn == enum_fqcn,
        _ => false,
    }
}

fn mentions_enum_case(ty: &Type) -> bool {
    ty.types.iter().any(atomic_mentions_enum_case)
}

/// [`normalize_enum_cases`], borrowing when `ty` mentions no enum case.
pub(crate) fn canonical_enum_cases<'a>(db: &dyn MirDatabase, ty: &'a Type) -> Cow<'a, Type> {
    if !mentions_enum_case(ty) {
        return Cow::Borrowed(ty);
    }
    let normalized = normalize_enum_cases(db, ty);
    if normalized == *ty {
        Cow::Borrowed(ty)
    } else {
        Cow::Owned(normalized)
    }
}

/// Canonical form for enum cases: docblock `Enum::Case` refs become case
/// literals, and a union covering every case of an enum collapses to the enum.
fn normalize_enum_cases(db: &dyn MirDatabase, ty: &Type) -> Type {
    let norm = |t: &Type| Box::new(normalize_enum_cases(db, t));
    let mut out = Type::empty();
    out.possibly_undefined = ty.possibly_undefined;
    out.from_docblock = ty.from_docblock;
    for atomic in &ty.types {
        if let Atomic::TNamedObject { fqcn, .. } = atomic {
            if let Some(constants) = class_constant_ref_type(db, fqcn) {
                for a in constants.types {
                    out.add_type(a);
                }
                continue;
            }
        }
        if let Atomic::TValueOf { target } = atomic {
            if let Some(values) = enum_backing_values(db, target) {
                for a in values.types {
                    out.add_type(a);
                }
                continue;
            }
        }
        out.add_type(match atomic {
            Atomic::TNamedObject { fqcn, .. } if fqcn.contains("::") => {
                docblock_case_literal(db, fqcn).unwrap_or_else(|| atomic.clone())
            }
            Atomic::TNamedObject { fqcn, type_params } if !type_params.is_empty() => {
                Atomic::TNamedObject {
                    fqcn: *fqcn,
                    type_params: mir_types::union::vec_to_type_params(
                        type_params
                            .iter()
                            .map(|t| normalize_enum_cases(db, t))
                            .collect(),
                    ),
                }
            }
            Atomic::TArray { key, value } => Atomic::TArray {
                key: norm(key),
                value: norm(value),
            },
            Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
                key: norm(key),
                value: norm(value),
            },
            Atomic::TList { value } => Atomic::TList { value: norm(value) },
            Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList { value: norm(value) },
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => Atomic::TKeyedArray {
                properties: Box::new(
                    properties
                        .iter()
                        .map(|(k, p)| {
                            let mut p = p.clone();
                            p.ty = normalize_enum_cases(db, &p.ty);
                            (k.clone(), p)
                        })
                        .collect(),
                ),
                is_open: *is_open,
                is_list: *is_list,
            },
            other => other.clone(),
        });
    }
    collapse_complete_enums(db, out)
}

/// `value-of<E>` for backed enums: the case values, or the backing scalar when a case value is
/// not a literal. `None` unless every atom of `target` is a backed enum.
pub(crate) fn enum_backing_values(db: &dyn MirDatabase, target: &Type) -> Option<Type> {
    let mut result = Type::empty();
    for atomic in &target.types {
        let Atomic::TNamedObject { fqcn, type_params } = atomic else {
            return None;
        };
        if !type_params.is_empty() {
            return None;
        }
        let crate::db::ClassLike::Enum(e) =
            crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, fqcn))?
        else {
            return None;
        };
        let scalar = e.scalar_type.as_ref()?;
        let literals: Option<Vec<Atomic>> = e
            .cases
            .values()
            .map(|c| {
                let v = c.value.as_ref()?;
                v.types
                    .iter()
                    .all(|a| matches!(a, Atomic::TLiteralInt(_) | Atomic::TLiteralString(_)))
                    .then(|| v.types.clone())
            })
            .collect::<Option<Vec<_>>>()
            .map(|vs| vs.into_iter().flatten().collect());
        match literals {
            Some(atoms) if !atoms.is_empty() => atoms.into_iter().for_each(|a| result.add_type(a)),
            _ => scalar.types.iter().for_each(|a| result.add_type(a.clone())),
        }
    }
    (!result.types.is_empty()).then_some(result)
}

/// Union of the constant types a docblock `Cls::NAME` / `Cls::PREFIX*` names. `None` for enums
/// (their cases are handled separately) and when no constant matches.
fn class_constant_ref_type(db: &dyn MirDatabase, name: &str) -> Option<Type> {
    let (class_name, pattern) = name.split_once("::")?;
    let fqcn = crate::db::Fqcn::from_str(db, class_name);
    if matches!(
        crate::db::find_class_like(db, fqcn)?,
        crate::db::ClassLike::Enum(_)
    ) {
        return None;
    }
    let prefix = pattern.strip_suffix('*');
    let mut seen: Vec<std::sync::Arc<str>> = Vec::new();
    let mut result = Type::empty();
    for ancestor in crate::db::class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = crate::db::Fqcn::interned(db, Name::new(ancestor.as_ref()));
        let Some(class) = crate::db::find_class_like(db, here) else {
            continue;
        };
        for (const_name, def) in class.own_constants().iter() {
            let matches = prefix.map_or(const_name.as_ref() == pattern, |p| {
                const_name.starts_with(p)
            });
            if matches && !seen.contains(const_name) {
                seen.push(const_name.clone());
                for a in def.ty.types.iter() {
                    result.add_type(a.clone());
                }
            }
        }
    }
    (!result.types.is_empty()).then_some(result)
}

fn docblock_case_literal(db: &dyn MirDatabase, name: &str) -> Option<Atomic> {
    let (enum_name, case_name) = name.split_once("::")?;
    let crate::db::ClassLike::Enum(e) =
        crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, enum_name))?
    else {
        return None;
    };
    e.cases
        .contains_key(case_name)
        .then(|| Atomic::TLiteralEnumCase {
            enum_fqcn: e.fqcn.as_ref().into(),
            case_name: case_name.into(),
        })
}

fn collapse_complete_enums(db: &dyn MirDatabase, ty: Type) -> Type {
    let mut enums: Vec<Name> = Vec::new();
    for a in &ty.types {
        if let Atomic::TLiteralEnumCase { enum_fqcn, .. } = a {
            if !enums.contains(enum_fqcn) {
                enums.push(*enum_fqcn);
            }
        }
    }
    let mut result = ty;
    for enum_fqcn in enums {
        let Some(crate::db::ClassLike::Enum(e)) =
            crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, enum_fqcn.as_ref()))
        else {
            continue;
        };
        let covers_all = e.cases.keys().all(|case| {
            result.types.iter().any(|a| {
                matches!(a, Atomic::TLiteralEnumCase { enum_fqcn: f, case_name }
                    if *f == enum_fqcn && case_name.as_ref() == case.as_ref())
            })
        });
        if covers_all && !e.cases.is_empty() {
            result.types.retain(
                |a| !matches!(a, Atomic::TLiteralEnumCase { enum_fqcn: f, .. } if *f == enum_fqcn),
            );
            result.add_type(Atomic::TNamedObject {
                fqcn: enum_fqcn,
                type_params: mir_types::union::empty_type_params(),
            });
        }
    }
    result
}
