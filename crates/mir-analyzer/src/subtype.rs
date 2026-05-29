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
use mir_types::{Atomic, Type};

use crate::db::{extends_or_implements, MirDatabase};

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
                            && sub_params.iter().all(|p| p.is_mixed() || p.is_never()));
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
                // An intersection type is a subtype of C if any of its parts is a subtype of C
                // (a value satisfying A&B is also an A and also a B).
                (Atomic::TIntersection { parts }, b) => {
                    let sup_single = Type::single(b.clone());
                    parts.iter().any(|part| is_subtype(db, part, &sup_single))
                }
                _ => false,
            }
        })
    })
}
