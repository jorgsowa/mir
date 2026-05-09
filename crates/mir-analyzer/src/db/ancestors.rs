use std::sync::Arc;

use super::*;

// class_ancestors tracked query (S2)

fn ancestors_initial(_db: &dyn MirDatabase, _id: salsa::Id, _node: ClassNode) -> Ancestors {
    Ancestors(vec![])
}

fn ancestors_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &Ancestors,
    _value: Ancestors,
    _node: ClassNode,
) -> Ancestors {
    // PHP class cycles are a compile-time error.  Break immediately with an
    // empty list so the fixpoint converges on the first iteration.
    Ancestors(vec![])
}

/// Salsa tracked query: compute the transitive ancestor list for a class or
/// interface.
///
/// Ancestors are accumulated in the same order as `Codebase::ensure_finalized`:
/// parent → parent's ancestors → implemented interfaces + their ancestors →
/// used traits (class); or: extended interfaces + their ancestors (interface).
///
/// Cycle recovery returns an empty list on the first iteration, which is
/// correct because PHP forbids circular inheritance.
#[salsa::tracked(cycle_fn = ancestors_cycle, cycle_initial = ancestors_initial)]
pub fn class_ancestors(db: &dyn MirDatabase, node: ClassNode) -> Ancestors {
    if !node.active(db) {
        return Ancestors(vec![]);
    }
    // Invariant: enums and traits always return empty here.
    // - Enums: enum membership questions go through
    //   `extends_or_implements_via_db`, which reads `interfaces` /
    //   `is_backed_enum` directly.
    // - Traits: trait-of-trait walking is handled by
    //   `method_is_concretely_implemented` / `trait_provides_method`
    //   directly via the `traits` field.
    // Do not lift either short-circuit without also auditing every caller
    // of `class_ancestors`.
    if node.is_enum(db) || node.is_trait(db) {
        return Ancestors(vec![]);
    }

    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();

    let add =
        |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut rustc_hash::FxHashSet<Arc<str>>| {
            if seen.insert(fqcn.clone()) {
                all.push(fqcn.clone());
            }
        };

    if node.is_interface(db) {
        for e in node.extends(db).iter() {
            add(e, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(e) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
    } else {
        if let Some(ref p) = node.parent(db) {
            add(p, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(p) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for iface in node.interfaces(db).iter() {
            add(iface, &mut all, &mut seen);
            if let Some(iface_node) = db.lookup_class_node(iface) {
                for a in class_ancestors(db, iface_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for t in node.traits(db).iter() {
            add(t, &mut all, &mut seen);
        }
    }

    Ancestors(all)
}
