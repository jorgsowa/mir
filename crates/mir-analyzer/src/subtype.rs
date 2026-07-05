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
use mir_types::{Atomic, Type, Variance};

use crate::db::{class_template_params, extends_or_implements, MirDatabase};

/// A supertype type-parameter that's effectively wildcarded — an unbound
/// template var or `mixed`. When the supertype's params are all free, we
/// treat the supertype as "any instantiation" for subtype matching.
fn sup_param_is_free(ty: &Type) -> bool {
    ty.is_mixed()
        || ty
            .types
            .iter()
            .all(|a| matches!(a, Atomic::TTemplateParam { .. }))
}

/// Per-position variance check for two parameterizations of the SAME class
/// (`Box<Dog>` vs `Box<Animal>`): a `@template-covariant`/`-contravariant`
/// param may differ in the declared direction; invariant params must match
/// exactly (the `sub_params == sup_params` fast path in `is_subtype` already
/// covers the all-invariant case, so a mismatch here only survives when at
/// least one param is variant).
fn variance_compatible(
    db: &dyn MirDatabase,
    fqcn: &str,
    sub_params: &[Type],
    sup_params: &[Type],
) -> bool {
    if sub_params.len() != sup_params.len() {
        return false;
    }
    let Some(tps) = class_template_params(db, fqcn) else {
        return false;
    };
    tps.iter()
        .zip(sub_params)
        .zip(sup_params)
        .all(|((tp, sub_p), sup_p)| match tp.variance {
            Variance::Covariant => is_subtype(db, sub_p, sup_p),
            Variance::Contravariant => is_subtype(db, sup_p, sub_p),
            Variance::Invariant => sub_p == sup_p,
        })
}

/// Returns true if `sub` is a subtype of `sup`, considering the codebase's
/// class-hierarchy graph (`extends` / `implements`) on top of structural
/// matches.
pub(crate) fn is_subtype(db: &dyn MirDatabase, sub: &Type, sup: &Type) -> bool {
    if sub.is_subtype_structural(sup) {
        return true;
    }
    if sup.is_mixed() {
        return true;
    }
    if sub.is_never() {
        return true;
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
            if Type::single(a.clone()).is_subtype_structural(&Type::single(b.clone())) {
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
                    // For parameterized classes we can only reason about the
                    // hierarchy when the supertype is bare (no `<...>`), the
                    // supertype's params are all unbound template vars (e.g.
                    // `Base<K, V>` where `K`/`V` are free), both sides match
                    // exactly, or the sub's params are free:
                    // - `mixed` explicitly opts out of type-param checking
                    //   (mirrors Psalm/PHPStan behaviour for `mixed` args)
                    // - `never` is the bottom type and a subtype of every type
                    let params_ok = sup_params.is_empty()
                        || sub_params == sup_params
                        || sup_params.iter().all(sup_param_is_free)
                        || (!sub_params.is_empty()
                            && sub_params.iter().all(|p| p.is_mixed() || p.is_never()))
                        || (sub_fqcn == sup_fqcn
                            && variance_compatible(db, sub_fqcn.as_ref(), sub_params, sup_params));
                    params_ok && extends_or_implements(db, sub_fqcn.as_ref(), sup_fqcn.as_ref())
                }
                (Atomic::TNamedObject { fqcn: sub_fqcn, .. }, Atomic::TIntersection { parts }) => {
                    // sub satisfies intersection bound iff it satisfies every part
                    parts.iter().all(|part| {
                        part.types.iter().any(|part_atomic| match part_atomic {
                            Atomic::TNamedObject {
                                fqcn: part_fqcn, ..
                            } => extends_or_implements(db, sub_fqcn.as_ref(), part_fqcn.as_ref()),
                            _ => false,
                        })
                    })
                }
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
                _ => false,
            }
        })
    })
}

fn is_interface(db: &dyn MirDatabase, fqcn: &str) -> bool {
    crate::db::class_kind(db, fqcn).is_some_and(|k| k.is_interface)
}
