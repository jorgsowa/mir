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

use mir_codebase::definitions::StubSlice;
use mir_types::{atomic::Atomic, Type};
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

    let defs = crate::db::collect_file_definitions(db, file);
    collect_structural_dep_targets(&defs.slice, &mut add_target);

    let mut sorted: Vec<Arc<str>> = targets.into_iter().collect();
    sorted.sort();
    sorted.into()
}

fn collect_structural_dep_targets(slice: &StubSlice, mut add_target: impl FnMut(&str)) {
    for c in slice.classes.iter() {
        if let Some(parent) = &c.parent {
            add_target(parent.as_ref());
        }
        for interface in c.interfaces.iter() {
            add_target(interface.as_ref());
        }
        for trait_fqcn in c.traits.iter() {
            add_target(trait_fqcn.as_ref());
        }
        for prop in c.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_target);
            }
        }
        for method in c.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_target);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_target);
            }
        }
    }

    for i in slice.interfaces.iter() {
        for extended in i.extends.iter() {
            add_target(extended.as_ref());
        }
        for prop in i.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_target);
            }
        }
        for method in i.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_target);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_target);
            }
        }
    }

    for t in slice.traits.iter() {
        for trait_fqcn in t.traits.iter() {
            add_target(trait_fqcn.as_ref());
        }
        for prop in t.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_target);
            }
        }
        for method in t.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_target);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_target);
            }
        }
    }

    for e in slice.enums.iter() {
        for interface in e.interfaces.iter() {
            add_target(interface.as_ref());
        }
        for trait_fqcn in e.traits.iter() {
            add_target(trait_fqcn.as_ref());
        }
        for method in e.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_target);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_target);
            }
        }
    }

    for f in slice.functions.iter() {
        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                collect_named_object_atoms(ty.as_ref(), &mut add_target);
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            collect_named_object_atoms(rt, &mut add_target);
        }
    }

    for fqcn in slice.imports.values() {
        add_target(fqcn.as_str());
    }
}

fn collect_named_object_atoms(ty: &Type, add_target: &mut impl FnMut(&str)) {
    for atomic in ty.types.iter() {
        if let Atomic::TNamedObject { fqcn, .. } = atomic {
            add_target(fqcn.as_str());
        }
    }
}
