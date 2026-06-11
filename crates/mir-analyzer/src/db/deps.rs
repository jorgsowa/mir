//! Structural file-dependency edges as a tracked query.
//!
//! [`file_structural_deps`] memoizes the set of files `file` depends on
//! through its *declarations*: defining files of its `use` imports, parent /
//! interface / trait FQCNs, and named-object type hints on properties,
//! params and return types. Body-level bare-FQN references are deliberately
//! excluded — they live in the reference index (a side channel salsa cannot
//! observe) and are merged in by `AnalysisSession::dependency_graph`.
//!
//! Memoization makes the warm `dependency_graph()` rebuild cheap: the query
//! depends on the file's definitions and on the workspace symbol index
//! revision (via `symbol_defining_file`), so it re-runs only when the file's
//! declarations or the symbol→file mapping actually change.

use std::sync::Arc;

use rustc_hash::FxHashSet;

use super::*;

/// Files that `file`'s declarations depend on. Sorted for deterministic
/// memo equality. Self-edges are excluded.
#[salsa::tracked]
pub fn file_structural_deps(db: &dyn MirDatabase, file: SourceFile) -> Arc<[Arc<str>]> {
    let path = file.path(db);
    let mut targets: FxHashSet<Arc<str>> = FxHashSet::default();

    let mut add_target = |symbol: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            if defining_file.as_ref() != path.as_ref() {
                targets.insert(defining_file);
            }
        }
    };

    let extract_named_objects = |union: &mir_types::Type| {
        union
            .types
            .iter()
            .filter_map(|atomic| match atomic {
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(*fqcn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let defs = crate::db::collect_file_definitions(db, file);

    for fqcn in defs.slice.imports.values() {
        add_target(fqcn.as_str());
    }

    for c in defs.slice.classes.iter() {
        if let Some(p) = &c.parent {
            add_target(p);
        }
        for iface in c.interfaces.iter() {
            add_target(iface);
        }
        for tr in c.traits.iter() {
            add_target(tr);
        }
        for prop in c.own_properties.values() {
            if let Some(ty) = &prop.ty {
                for named in extract_named_objects(ty) {
                    add_target(named.as_ref());
                }
            }
        }
        for method in c.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_target(named.as_ref());
                    }
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                for named in extract_named_objects(rt) {
                    add_target(named.as_ref());
                }
            }
        }
    }
    for i in defs.slice.interfaces.iter() {
        for ext in i.extends.iter() {
            add_target(ext);
        }
        for method in i.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_target(named.as_ref());
                    }
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                for named in extract_named_objects(rt) {
                    add_target(named.as_ref());
                }
            }
        }
    }
    for t in defs.slice.traits.iter() {
        for tr in t.traits.iter() {
            add_target(tr);
        }
    }
    for f in defs.slice.functions.iter() {
        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                for named in extract_named_objects(ty.as_ref()) {
                    add_target(named.as_ref());
                }
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            for named in extract_named_objects(rt) {
                add_target(named.as_ref());
            }
        }
    }

    let mut sorted: Vec<Arc<str>> = targets.into_iter().collect();
    sorted.sort();
    sorted.into()
}
