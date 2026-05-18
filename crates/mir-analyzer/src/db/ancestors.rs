use std::sync::Arc;

use super::*;

// class_ancestors tracked query — Phase 4 H1 (full): FQCN-keyed.
//
// Previously keyed on `ClassNode` (push-path salsa input handle); the
// query body and its callers now go through `Fqcn` so the push-based
// index can be deleted in Phase 5. Semantics unchanged: DFS order,
// excludes `fqcn` itself, returns empty for enums and traits.

fn ancestors_initial<'db>(
    _db: &'db dyn MirDatabase,
    _id: salsa::Id,
    _fqcn: Fqcn<'db>,
) -> Ancestors {
    Ancestors(vec![])
}

fn ancestors_cycle<'db>(
    _db: &'db dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &Ancestors,
    _value: Ancestors,
    _fqcn: Fqcn<'db>,
) -> Ancestors {
    // PHP class cycles are a compile-time error. Break immediately with an
    // empty list so the fixpoint converges on the first iteration.
    Ancestors(vec![])
}

/// Salsa tracked query: compute the transitive ancestor list for a class
/// or interface, identified by FQCN.
///
/// Ancestors are accumulated in the same order as
/// `Codebase::ensure_finalized`:
///   parent → parent's ancestors → implemented interfaces + their ancestors
///   → used traits (class); or:
///   extended interfaces + their ancestors (interface).
///
/// Empty for enums and traits — by design; consumers handling those kinds
/// go through `extends_or_implements_via_db` or `trait_provides_method`
/// directly.
///
/// Cycle recovery returns an empty list on the first iteration, which is
/// correct because PHP forbids circular inheritance.
#[salsa::tracked(cycle_fn = ancestors_cycle, cycle_initial = ancestors_initial)]
pub fn class_ancestors<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Ancestors {
    let name = fqcn.name(db);

    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();

    let add =
        |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut rustc_hash::FxHashSet<Arc<str>>| {
            if seen.insert(fqcn.clone()) {
                all.push(fqcn.clone());
            }
        };

    // Phase 4: prefer the pull path (`find_class_like`). Fall back to the
    // push-path `ClassNode` reads for classes that aren't yet reachable
    // via a registered `SourceFile` — chiefly the direct
    // `upsert_class_node` path in tests/fixtures. Phase 5 deletes both
    // the push index and this fallback together.
    if let Some(class) = find_class_like(db, fqcn) {
        if class.is_enum() || class.is_trait() {
            return Ancestors(vec![]);
        }
        match &class {
            ClassLike::Interface(iface) => {
                for e in iface.extends.iter() {
                    add(e, &mut all, &mut seen);
                    let parent_fqcn = Fqcn::new(db, e.clone());
                    for a in class_ancestors(db, parent_fqcn).0 {
                        add(&a, &mut all, &mut seen);
                    }
                }
            }
            ClassLike::Class(cls) => {
                if let Some(ref p) = cls.parent {
                    add(p, &mut all, &mut seen);
                    let parent_fqcn = Fqcn::new(db, p.clone());
                    for a in class_ancestors(db, parent_fqcn).0 {
                        add(&a, &mut all, &mut seen);
                    }
                }
                for iface in cls.interfaces.iter() {
                    add(iface, &mut all, &mut seen);
                    let iface_fqcn = Fqcn::new(db, iface.clone());
                    for a in class_ancestors(db, iface_fqcn).0 {
                        add(&a, &mut all, &mut seen);
                    }
                }
                for t in cls.traits.iter() {
                    add(t, &mut all, &mut seen);
                }
            }
            _ => {}
        }
        return Ancestors(all);
    }

    // Push-path fallback.
    let node = match db.lookup_class_node(name.as_ref()).filter(|n| n.active(db)) {
        Some(n) => n,
        None => return Ancestors(vec![]),
    };
    if node.is_enum(db) || node.is_trait(db) {
        return Ancestors(vec![]);
    }
    if node.is_interface(db) {
        for e in node.extends(db).iter() {
            add(e, &mut all, &mut seen);
            let parent_fqcn = Fqcn::new(db, e.clone());
            for a in class_ancestors(db, parent_fqcn).0 {
                add(&a, &mut all, &mut seen);
            }
        }
    } else {
        if let Some(ref p) = node.parent(db) {
            add(p, &mut all, &mut seen);
            let parent_fqcn = Fqcn::new(db, p.clone());
            for a in class_ancestors(db, parent_fqcn).0 {
                add(&a, &mut all, &mut seen);
            }
        }
        for iface in node.interfaces(db).iter() {
            add(iface, &mut all, &mut seen);
            let iface_fqcn = Fqcn::new(db, iface.clone());
            for a in class_ancestors(db, iface_fqcn).0 {
                add(&a, &mut all, &mut seen);
            }
        }
        for t in node.traits(db).iter() {
            add(t, &mut all, &mut seen);
        }
    }
    Ancestors(all)
}
