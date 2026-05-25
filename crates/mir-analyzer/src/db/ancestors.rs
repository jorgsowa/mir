use std::sync::Arc;

use mir_types::Name;

use super::*;

// class_ancestors tracked query — pull-only via `find_class_like`.
//
// Order matches `Codebase::ensure_finalized`: parent → parent's ancestors →
// implemented interfaces + their ancestors → used traits (for a class);
// extended interfaces + their ancestors (for an interface).
//
// Empty for enums and traits — by design; consumers handling those kinds
// go through `extends_or_implements` or `trait_provides_method`
// directly.

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
    Ancestors(vec![])
}

/// Salsa tracked query: compute the transitive ancestor list for a class
/// or interface, identified by FQCN.
#[salsa::tracked(cycle_fn = ancestors_cycle, cycle_initial = ancestors_initial)]
pub fn class_ancestors<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Ancestors {
    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();

    let add =
        |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut rustc_hash::FxHashSet<Arc<str>>| {
            if seen.insert(fqcn.clone()) {
                all.push(fqcn.clone());
            }
        };

    let Some(class) = find_class_like(db, fqcn) else {
        return Ancestors(vec![]);
    };
    if class.is_enum() || class.is_trait() {
        return Ancestors(vec![]);
    }
    match &class {
        ClassLike::Interface(iface) => {
            for e in iface.extends.iter() {
                add(e, &mut all, &mut seen);
                let parent_fqcn = Fqcn::new(db, Name::new(e.as_ref()));
                for a in class_ancestors(db, parent_fqcn).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        ClassLike::Class(cls) => {
            if let Some(ref p) = cls.parent {
                add(p, &mut all, &mut seen);
                let parent_fqcn = Fqcn::new(db, Name::new(p.as_ref()));
                for a in class_ancestors(db, parent_fqcn).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
            for iface in cls.interfaces.iter() {
                add(iface, &mut all, &mut seen);
                let iface_fqcn = Fqcn::new(db, Name::new(iface.as_ref()));
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
    Ancestors(all)
}
