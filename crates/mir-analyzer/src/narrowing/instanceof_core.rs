//! Core `instanceof`/`is_a()`/`is_subclass_of()` matching: narrows a type
//! against a named class for variable, property, and static-property
//! receivers, preserving/projecting template type params where possible.
use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, resolve_prop_current_type, resolve_static_prop_current_type,
    UnionNarrowExt,
};

/// When `class_name` is a (possibly indirect) subclass/subinterface of
/// `atom_fqcn` and the atom being narrowed carries concrete `type_params`
/// (e.g. `Box<int>`), project those onto `class_name`'s own template params
/// instead of discarding them: `$b instanceof IntBox` on a `Box<int>`
/// receiver should narrow to `IntBox<int>`, not a bare unparameterized
/// `IntBox` that leaves a later `IntBox` method's own `@return T` unresolved
/// and unrelated to `Box`'s binding.
///
/// Handles the two shapes real code uses to relate a subclass's own template
/// params to its ancestor's: an explicit `@extends`/`@implements
/// Ancestor<...>` clause whose args are bare references to the subclass's
/// own template param names (identity or renamed passthrough), and the
/// simpler case where the subclass declares no such clause at all but has
/// the same template arity as the ancestor, which real-world code (and this
/// analyzer's own `class_template_params`) treats as an implicit,
/// unchanged passthrough. Anything else (arity mismatch, no relationship
/// found) falls back to no type params, same as before this projection
/// existed.
pub(super) fn project_type_params_onto_subclass(
    db: &dyn MirDatabase,
    atom_fqcn: &str,
    atom_type_params: &[Type],
    class_name: &str,
) -> std::sync::Arc<[Type]> {
    let Some(class_own_tps) = crate::db::class_template_params(db, class_name) else {
        return mir_types::union::empty_type_params();
    };
    if class_own_tps.is_empty() {
        return mir_types::union::empty_type_params();
    }
    let Some(atom_own_tps) = crate::db::class_template_params(db, atom_fqcn) else {
        return mir_types::union::empty_type_params();
    };
    let here = crate::db::Fqcn::from_str(db, class_name);
    let Some(class) = crate::db::find_class_like(db, here) else {
        return mir_types::union::empty_type_params();
    };

    let explicit_args: Option<&[Type]> = if class
        .parent()
        .is_some_and(|p| p.as_ref().eq_ignore_ascii_case(atom_fqcn))
    {
        Some(class.extends_type_args())
    } else {
        class
            .implements_type_args()
            .iter()
            .chain(class.interface_extends_type_args())
            .find(|(iface, _)| iface.as_ref().eq_ignore_ascii_case(atom_fqcn))
            .map(|(_, args)| args.as_slice())
    };

    let mut result = vec![Type::mixed(); class_own_tps.len()];
    let mut any_bound = false;

    if let Some(args) = explicit_args.filter(|a| !a.is_empty()) {
        for (idx, given_ty) in atom_type_params.iter().enumerate() {
            let Some(arg_expr) = args.get(idx) else {
                continue;
            };
            let Some(bare_name) = bare_named_type(arg_expr) else {
                continue;
            };
            if let Some(pos) = class_own_tps
                .iter()
                .position(|tp| tp.name.as_str() == bare_name)
            {
                result[pos] = given_ty.clone();
                any_bound = true;
            }
        }
    } else if class_own_tps.len() == atom_own_tps.len() {
        result = atom_type_params.to_vec();
        any_bound = !result.is_empty();
    }

    if any_bound {
        mir_types::union::vec_to_type_params(result)
    } else {
        mir_types::union::empty_type_params()
    }
}

/// A `Type` consisting of exactly one bare, unqualified named-type atom
/// (e.g. a docblock's `@extends Box<U>` argument referencing a template
/// param by name) — as opposed to a real, concrete class reference or a
/// compound type. Returns that name, if so.
fn bare_named_type(ty: &Type) -> Option<&str> {
    if ty.types.len() != 1 {
        return None;
    }
    match &ty.types[0] {
        Atomic::TNamedObject { fqcn, type_params }
            if type_params.is_empty() && !fqcn.contains('\\') =>
        {
            Some(fqcn.as_ref())
        }
        // The collector stores `@extends Box<U>` args template-aware, so a
        // template-param reference arrives as a proper TTemplateParam atom.
        Atomic::TTemplateParam { name, .. } => Some(name.as_ref()),
        _ => None,
    }
}

pub(super) fn narrow_instanceof_preserving_subtypes(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let narrowed_ty = Atomic::TNamedObject {
        fqcn: class_name.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        return Type::single(narrowed_ty);
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if named_object_matches_instanceof(fqcn, class_name, db) =>
            {
                result.add_type(atomic.clone());
            }
            // Handle template parameters: if a bare unqualified name matches a template param,
            // intersect it with the checked class rather than replacing it — the value is
            // still guaranteed to be a T (e.g. for a later `@return T`), just now also
            // known to be an instance of `class_name`.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            // Handle TTemplateParam: intersect it with the instanceof check class instead
            // of discarding the template binding (see comment above).
            Atomic::TTemplateParam { .. } => {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
            // `$x instanceof C` on an `A&B`-typed value adds C to the
            // intersection rather than replacing it — the value is still
            // guaranteed to be an A and a B, so dropping them here would
            // falsely reject valid uses of the original intersection.
            Atomic::TIntersection { parts } => {
                let already_covered = parts.iter().any(|p| {
                    p.types.iter().any(|a| {
                        matches!(a, Atomic::TNamedObject { fqcn, .. }
                            if named_object_matches_instanceof(fqcn, class_name, db))
                    })
                });
                if already_covered {
                    result.add_type(atomic.clone());
                } else {
                    // Same reasoning as the non-intersection `!type_params.is_empty()`
                    // arm below: if a part is a generic `TNamedObject` that
                    // `class_name` is a subtype of, project its type params onto
                    // `class_name` instead of appending a raw, empty-type-params atom.
                    let projected_atom = parts.iter().find_map(|p| {
                        p.types.iter().find_map(|a| match a {
                            Atomic::TNamedObject { fqcn, type_params }
                                if !type_params.is_empty()
                                    && named_object_matches_instanceof(class_name, fqcn, db) =>
                            {
                                Some(Atomic::TNamedObject {
                                    fqcn: class_name.into(),
                                    type_params: project_type_params_onto_subclass(
                                        db,
                                        fqcn,
                                        type_params,
                                        class_name,
                                    ),
                                })
                            }
                            _ => None,
                        })
                    });
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(Type::single(
                        projected_atom.unwrap_or_else(|| narrowed_ty.clone()),
                    ));
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            // `class_name` is a (possibly indirect) subtype of the atom's own class
            // AND the atom carries concrete type params (e.g. `Box<int>`
            // narrowed by `instanceof IntBox`) — project them onto
            // `class_name`'s own template params rather than discarding them.
            Atomic::TNamedObject { fqcn, type_params }
                if !type_params.is_empty()
                    && named_object_matches_instanceof(class_name, fqcn, db) =>
            {
                let projected =
                    project_type_params_onto_subclass(db, fqcn, type_params, class_name);
                result.add_type(Atomic::TNamedObject {
                    fqcn: class_name.into(),
                    type_params: projected,
                });
            }
            // `class_name` is a (possibly indirect) subtype of the atom's own class
            // — e.g. atom is the `Foo` interface and class_name is `A implements
            // Foo` — so the instanceof check's result subsumes and is strictly
            // more specific than what's already known; replace outright rather
            // than forming a redundant `Foo&A` intersection.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if named_object_matches_instanceof(class_name, fqcn, db) =>
            {
                result.add_type(narrowed_ty.clone());
            }
            // A named object unrelated to `class_name` by inheritance in either
            // direction (e.g. two interfaces neither of which extends the other,
            // as in `$x instanceof A && $x instanceof B`) must not be silently
            // discarded — the instanceof check proved the value ALSO satisfies
            // class_name. Form an intersection when that's actually possible
            // (at least one side is an interface, so a single object can
            // implement both); otherwise the atom's own class and class_name are
            // both concrete classes, which PHP's single inheritance makes
            // mutually exclusive, so the atom is provably impossible here and is
            // correctly dropped.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if classes_can_coexist(fqcn, class_name, db) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            // A `Closure(...): R`-typed atom (its own dedicated atomic, not a
            // TNamedObject) genuinely IS an instance of `Closure` at runtime —
            // keep it as-is rather than falling through to the catch-all drop,
            // which would make `$x instanceof Closure` on a `Closure(): T`-typed
            // value look provably impossible.
            Atomic::TClosure { .. } if class_name.eq_ignore_ascii_case("Closure") => {
                result.add_type(atomic.clone());
            }
            _ => {}
        }
    }

    // Unlike the early-return above (truly unconstrained `mixed`/empty `current`),
    // reaching here with an empty `result` means `current` had at least one real
    // atom and NONE of them survived narrowing — every atom was proven
    // incompatible with `class_name` (e.g. two unrelated `final` classes).
    // Propagate the emptiness instead of resetting to a bare `narrowed_ty`, so
    // the caller's `mark_diverges` can correctly flag the branch as unreachable
    // instead of silently treating a provably-impossible instanceof as if
    // nothing were known about the value.
    result
}

/// Like [`narrow_instanceof_preserving_subtypes`], but for an OR-chain of
/// `instanceof` checks against several classes at once (`$x instanceof A ||
/// $x instanceof B`) — narrowing per-class-then-merging (as opposed to
/// per-atom-across-all-classes) double-counts `TIntersection` union members
/// unrelated to any single disjunct: `(A&B)|C|D` narrowed by `instanceof C ||
/// instanceof D` would otherwise produce two separate `A&B&C`/`A&B&D`
/// members instead of one `A&B&(C|D)` member, bloating the displayed type
/// and hiding it from later, more precise checks.
pub(super) fn narrow_or_instanceof_union(
    current: &Type,
    class_names: &[String],
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let class_atom = |cn: &str| Atomic::TNamedObject {
        fqcn: cn.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        let mut out = Type::empty();
        for cn in class_names {
            out.add_type(class_atom(cn));
        }
        return out;
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if class_names
                    .iter()
                    .any(|cn| named_object_matches_instanceof(fqcn, cn, db)) =>
            {
                result.add_type(atomic.clone());
            }
            // As in narrow_instanceof_preserving_subtypes, keep the template atom by
            // intersecting it with the union of checked classes rather than replacing it.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                let mut classes = Type::empty();
                for cn in class_names {
                    classes.add_type(class_atom(cn));
                }
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                });
            }
            Atomic::TTemplateParam { .. } => {
                let mut classes = Type::empty();
                for cn in class_names {
                    classes.add_type(class_atom(cn));
                }
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                });
            }
            Atomic::TObject | Atomic::TMixed => {
                for cn in class_names {
                    result.add_type(class_atom(cn));
                }
            }
            // As in narrow_instanceof_preserving_subtypes, a Closure(...): R atom
            // genuinely IS an instance of Closure at runtime — keep it when one
            // of the OR-chain's classes is Closure, instead of falling through
            // to the catch-all drop.
            Atomic::TClosure { .. }
                if class_names
                    .iter()
                    .any(|cn| cn.eq_ignore_ascii_case("Closure")) =>
            {
                result.add_type(atomic.clone());
            }
            Atomic::TIntersection { parts } => {
                let mut remaining = Type::empty();
                for cn in class_names {
                    let already_covered = parts.iter().any(|p| {
                        p.types.iter().any(|a| {
                            matches!(a, Atomic::TNamedObject { fqcn, .. }
                                if named_object_matches_instanceof(fqcn, cn, db))
                        })
                    });
                    if !already_covered {
                        // Same reasoning as the non-intersection `!type_params.is_empty()`
                        // arm below: if a part is a generic `TNamedObject` that `cn`
                        // is a subtype of, project its type params onto `cn` instead
                        // of a raw, empty-type-params atom.
                        let projected_atom = parts.iter().find_map(|p| {
                            p.types.iter().find_map(|a| match a {
                                Atomic::TNamedObject { fqcn, type_params }
                                    if !type_params.is_empty()
                                        && named_object_matches_instanceof(cn, fqcn, db) =>
                                {
                                    Some(Atomic::TNamedObject {
                                        fqcn: cn.as_str().into(),
                                        type_params: project_type_params_onto_subclass(
                                            db,
                                            fqcn,
                                            type_params,
                                            cn,
                                        ),
                                    })
                                }
                                _ => None,
                            })
                        });
                        remaining.add_type(projected_atom.unwrap_or_else(|| class_atom(cn)));
                    }
                }
                if remaining.is_empty() {
                    result.add_type(atomic.clone());
                } else {
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(remaining);
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            // Some disjunct(s) are a (possibly indirect) subtype of the atom's own
            // class AND the atom carries concrete type params — project them
            // onto each subsuming disjunct's own template params rather than
            // discarding them, mirroring narrow_instanceof_preserving_subtypes.
            Atomic::TNamedObject { fqcn, type_params }
                if !type_params.is_empty()
                    && class_names
                        .iter()
                        .any(|cn| named_object_matches_instanceof(cn, fqcn, db)) =>
            {
                for cn in class_names {
                    if named_object_matches_instanceof(cn, fqcn, db) {
                        let projected =
                            project_type_params_onto_subclass(db, fqcn, type_params, cn);
                        result.add_type(Atomic::TNamedObject {
                            fqcn: cn.as_str().into(),
                            type_params: projected,
                        });
                    }
                }
            }
            // Some disjunct(s) are a (possibly indirect) subtype of the atom's own
            // class — e.g. atom is `Foo` and one label checks `instanceof A` where
            // `A implements Foo` — so the instanceof result subsumes and is
            // strictly more specific; narrow to just the subsuming disjunct(s)
            // rather than forming a redundant `Foo&A` intersection.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if class_names
                    .iter()
                    .any(|cn| named_object_matches_instanceof(cn, fqcn, db)) =>
            {
                for cn in class_names {
                    if named_object_matches_instanceof(cn, fqcn, db) {
                        result.add_type(class_atom(cn));
                    }
                }
            }
            // A named object matching none of the disjuncts by inheritance in
            // either direction must not be silently discarded — the
            // (already-true) instanceof check proved the value ALSO satisfies
            // one of class_names. Intersect with only the disjuncts that could
            // actually coexist with this atom (at least one side an interface);
            // a disjunct that's a concrete class unrelated to this atom's own
            // concrete class is impossible under PHP's single inheritance and is
            // dropped instead. Mirrors the equivalent fix in
            // narrow_instanceof_preserving_subtypes for the single-class case.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn } => {
                let mut classes = Type::empty();
                for cn in class_names {
                    if classes_can_coexist(fqcn, cn, db) {
                        classes.add_type(class_atom(cn));
                    }
                }
                if !classes.is_empty() {
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                    });
                }
            }
            _ => {}
        }
    }

    // Unlike the early-return above (truly unconstrained `mixed`/empty
    // `current`), reaching here with an empty `result` means `current` had
    // at least one real atom and NONE of them survived narrowing against any
    // disjunct — every atom was proven incompatible with every disjunct
    // (e.g. unrelated `final` classes). Propagate the emptiness instead of
    // resetting to the disjuncts' bare union, mirroring
    // `narrow_instanceof_preserving_subtypes`, so the caller can correctly
    // flag the branch as unreachable instead of silently widening a
    // provably-impossible `instanceof` chain to `A|B`.
    result
}

/// Whether a value could simultaneously be (a subtype of) both `a` and `b` —
/// true when either is an interface (a class can implement any number of
/// interfaces), false when both are concrete classes, which PHP's single
/// inheritance makes mutually exclusive unless one already extends the other
/// (checked separately by the caller via `named_object_matches_instanceof`).
fn classes_can_coexist(a: &str, b: &str, db: &dyn MirDatabase) -> bool {
    crate::db::class_kind(db, a).is_some_and(|k| k.is_interface)
        || crate::db::class_kind(db, b).is_some_and(|k| k.is_interface)
}

pub(super) fn filter_out_instanceof_match(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
) -> Type {
    current.filter(|t| match t {
        Atomic::TNamedObject { fqcn, .. }
        | Atomic::TSelf { fqcn }
        | Atomic::TStaticObject { fqcn }
        | Atomic::TParent { fqcn } => !named_object_matches_instanceof(fqcn, class_name, db),
        // A Closure(...): R atom genuinely IS an instance of Closure at
        // runtime, so it's excluded by the false branch of `instanceof Closure`
        // just like a TNamedObject would be.
        Atomic::TClosure { .. } => !class_name.eq_ignore_ascii_case("Closure"),
        // A&B is provably excluded by `!($x instanceof C)` when EITHER part
        // alone would satisfy it — a value that's simultaneously an A and a B
        // is also a C the moment either A or B extends/implements C, so the
        // whole intersection can't survive the negation, not just its own
        // (nonexistent) direct name.
        Atomic::TIntersection { parts } => !parts.iter().any(|part| {
            part.types.iter().any(|inner| match inner {
                Atomic::TNamedObject { fqcn, .. }
                | Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => named_object_matches_instanceof(fqcn, class_name, db),
                _ => false,
            })
        }),
        _ => true,
    })
}

pub(super) fn named_object_matches_instanceof(
    fqcn: &str,
    class_name: &str,
    db: &dyn MirDatabase,
) -> bool {
    fqcn == class_name || crate::db::extends_or_implements(db, fqcn, class_name)
}

/// Partition `current`'s atoms for the `allow_string: true` true-branch of
/// `is_a($x, $class_name, true)`. A `class-string<C>` atom is dropped only
/// when `C` is provably unrelated to `class_name` in both directions AND
/// the two can't coexist on a single class (mirrors the object-atom
/// coexistence check in `narrow_instanceof_preserving_subtypes` above,
/// via `classes_can_coexist`) — a class-string naming an interface, or a
/// concrete class unrelated to `class_name` where `class_name` itself
/// names an interface, could still describe a subtype that also satisfies
/// `class_name`, so it isn't provably excluded. Any other string atom is
/// kept as-is (it might name `class_name` at runtime; there's nothing more
/// precise to narrow it to). The second element of the tuple is every
/// non-string atom, handed back separately so the caller can narrow it via
/// `instanceof` semantics.
pub(super) fn partition_is_a_string_like(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
) -> (Type, Type) {
    let mut string_part = Type::empty();
    string_part.possibly_undefined = current.possibly_undefined;
    string_part.from_docblock = current.from_docblock;
    let mut obj_part = Type::empty();
    for atom in &current.types {
        if let Atomic::TClassString(Some(name)) = atom {
            if named_object_matches_instanceof(name, class_name, db)
                || classes_can_coexist(name, class_name, db)
            {
                string_part.add_type(atom.clone());
            }
        } else if atom.is_string() {
            string_part.add_type(atom.clone());
        } else {
            obj_part.add_type(atom.clone());
        }
    }
    (string_part, obj_part)
}

/// `filter_out_instanceof_match`, extended for the `allow_string: true`
/// false-branch of `is_a()`: a `class-string<C>` atom provably matching
/// `class_name` is also excluded (mirrors the object-atom exclusion above —
/// `is_a()` being false rules out that specific class-string just as surely
/// as it rules out that specific object class).
pub(super) fn filter_out_is_a_string_match(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
) -> Type {
    filter_out_instanceof_match(current, class_name, db).filter(|t| {
        !matches!(t, Atomic::TClassString(Some(name)) if named_object_matches_instanceof(name, class_name, db))
    })
}

/// Narrow `current` for the true branch of `is_subclass_of($obj, 'ClassName')`.
///
/// Unlike `instanceof` / `is_a`, `is_subclass_of` requires a *strict* subclass:
/// the exact class itself is excluded. Atoms that are only the named class (not a
/// descendant) are dropped. Mixed/TObject are narrowed to the named class as the
/// best approximation (a value satisfying `is_subclass_of` must be some subclass,
/// and the named class is the tightest bound we can express).
pub(super) fn narrow_strict_subclass_of(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let narrowed_ty = Atomic::TNamedObject {
        fqcn: class_name.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        return Type::single(narrowed_ty);
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            // Strict subclass: keep only atoms that extend/implement without being the class itself.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if crate::db::extends_or_implements(db, fqcn.as_ref(), class_name)
                    && fqcn.as_ref() != class_name =>
            {
                result.add_type(atomic.clone());
            }
            // Template parameter — intersect with the named class rather than replacing it,
            // so the value is still known to be a T as well as a strict subclass of it.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TTemplateParam { .. } => {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
            // `is_subclass_of($x, class_name)` on an `A&B`-typed value adds
            // class_name to the intersection rather than discarding it —
            // mirrors narrow_instanceof_preserving_subtypes's TIntersection
            // handling above.
            Atomic::TIntersection { parts } => {
                let already_covered = parts.iter().any(|p| {
                    p.types.iter().any(|a| {
                        matches!(a, Atomic::TNamedObject { fqcn, .. }
                            if crate::db::extends_or_implements(db, fqcn.as_ref(), class_name)
                                && fqcn.as_ref() != class_name)
                    })
                });
                if already_covered {
                    result.add_type(atomic.clone());
                } else {
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(Type::single(narrowed_ty.clone()));
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            // A named object unrelated to `class_name` by inheritance in either
            // direction must not be silently discarded when it could still
            // coexist with class_name (at least one side is an interface, so a
            // single object can implement both) — mirrors
            // narrow_instanceof_preserving_subtypes's TIntersection handling.
            // Two unrelated concrete classes remain mutually exclusive under
            // PHP's single inheritance and are correctly dropped by the `_` arm.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if classes_can_coexist(fqcn, class_name, db) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            _ => {}
        }
    }

    result
    // Note: no fallback to Type::single(narrowed_ty) when result is empty — if the
    // current type contains no known subclasses of the named class, the narrowing
    // returns empty and the caller should NOT mark diverges (is_subclass_of may still
    // be false at runtime for the exact class, which is valid).
}

/// Narrow a static property's type when `self::$prop instanceof ClassName` /
/// `static::$prop instanceof ClassName` is proven true or false.
pub(super) fn narrow_static_prop_instanceof(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = if let Some(refined) = ctx.get_prop_refined(fqcn, prop) {
        refined.clone()
    } else {
        let here = crate::db::Fqcn::from_str(db, fqcn);
        crate::db::find_property_in_chain(db, here, prop)
            .and_then(|(_, p)| p.ty.as_deref().cloned())
            .unwrap_or_else(mir_types::Type::mixed)
    };

    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = if is_true {
        narrow_instanceof_preserving_subtypes(&current, class_name, db, &ctx.template_param_names)
    } else {
        filter_out_instanceof_match(&current, class_name, db)
    };
    if !narrowed.is_empty() {
        if narrowed != current {
            ctx.set_prop_refined(fqcn, prop, narrowed);
        }
    } else if !current.is_empty() && !current.is_mixed() {
        ctx.diverges = true;
    }
}

pub(super) fn narrow_prop_instanceof(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = if is_true {
        narrow_instanceof_preserving_subtypes(&current, class_name, db, &ctx.template_param_names)
    } else {
        filter_out_instanceof_match(&current, class_name, db)
    };
    // `!($obj->prop instanceof X)` is also true whenever $obj itself is null
    // (`null instanceof X` is always false), so a nullable receiver means an
    // empty false-branch narrowing isn't a real contradiction — same
    // reasoning as `narrow_prop_null`'s nullable-receiver gate. The true
    // branch is unaffected: `narrow_instanceof_preserving_subtypes` never
    // returns empty.
    let mark_diverges = is_true || !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_is_a`, for
/// `is_a(self::$prop, X::class, ...)` (and `static::$prop`/`Class::$prop`).
pub(super) fn narrow_static_prop_is_a(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    allow_string: bool,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed_not_template() {
        return;
    }
    if allow_string {
        let narrowed = if is_true {
            let (mut result, obj_part) = partition_is_a_string_like(&current, class_name, db);
            if !obj_part.is_empty() || current.is_mixed() {
                let obj_src = if obj_part.is_empty() {
                    &current
                } else {
                    &obj_part
                };
                let obj_narrowed = narrow_instanceof_preserving_subtypes(
                    obj_src,
                    class_name,
                    db,
                    &ctx.template_param_names,
                );
                for atom in obj_narrowed.types.iter() {
                    result.add_type(atom.clone());
                }
            }
            result
        } else {
            filter_out_is_a_string_match(&current, class_name, db)
        };
        // Same rationale as the variable case: don't mark diverges when
        // allow_string is set, since a class-string value may still pass.
        apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
    } else {
        let narrowed = if is_true {
            narrow_instanceof_preserving_subtypes(
                &current,
                class_name,
                db,
                &ctx.template_param_names,
            )
        } else {
            filter_out_instanceof_match(&current, class_name, db)
        };
        apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
    }
}

/// `is_a($obj->prop, ClassName::class)` / `is_a($obj->prop, ClassName::class, true)`
/// narrowing — same semantics as the variable-based `is_a` branch in
/// `narrow_from_condition`, applied to a property-access receiver instead.
#[allow(clippy::too_many_arguments)]
pub(super) fn narrow_prop_is_a(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    allow_string: bool,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    if allow_string {
        let narrowed = if is_true {
            let (mut result, obj_part) = partition_is_a_string_like(&current, class_name, db);
            if !obj_part.is_empty() || current.is_mixed() {
                let obj_src = if obj_part.is_empty() {
                    &current
                } else {
                    &obj_part
                };
                let obj_narrowed = narrow_instanceof_preserving_subtypes(
                    obj_src,
                    class_name,
                    db,
                    &ctx.template_param_names,
                );
                for atom in obj_narrowed.types.iter() {
                    result.add_type(atom.clone());
                }
            }
            result
        } else {
            filter_out_is_a_string_match(&current, class_name, db)
        };
        // Same rationale as the variable case: don't mark diverges when
        // allow_string is set, since a class-string value may still pass.
        if !narrowed.is_empty() && narrowed != current {
            ctx.set_prop_refined(obj_var, prop, narrowed);
        }
    } else {
        let narrowed = if is_true {
            narrow_instanceof_preserving_subtypes(
                &current,
                class_name,
                db,
                &ctx.template_param_names,
            )
        } else {
            filter_out_instanceof_match(&current, class_name, db)
        };
        // Same nullable-receiver gate as `narrow_prop_instanceof`'s false
        // branch: `is_a($obj->prop, X)` false is also true whenever $obj
        // itself is null.
        let mark_diverges = is_true || !ctx.get_var(obj_var).is_nullable();
        apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
    }
}

/// Static-property counterpart of `narrow_prop_is_subclass_of`, for
/// `is_subclass_of(self::$prop, ClassName::class)` (and `static::$prop`/
/// `Class::$prop`) — same strict-subclass-only semantics.
pub(super) fn narrow_static_prop_is_subclass_of(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    if !is_true {
        return;
    }
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = narrow_strict_subclass_of(&current, class_name, db, &ctx.template_param_names);
    // mark_diverges=false: the exact class being absent from strict-subclass
    // narrowing doesn't make the branch dead, mirroring the var/prop siblings.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// `is_subclass_of($obj->prop, ClassName::class)` narrowing — same semantics
/// as the variable-based branch (strict-subclass only; the false branch never
/// narrows since a non-subclass could still be the exact class itself).
pub(super) fn narrow_prop_is_subclass_of(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    if !is_true {
        return;
    }
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = narrow_strict_subclass_of(&current, class_name, db, &ctx.template_param_names);
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
    }
}
