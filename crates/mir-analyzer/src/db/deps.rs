//! Structural file-dependency facts as tracked queries.
//!
//! [`file_structural_symbols`] memoizes the set of symbols `file` depends on
//! through its *declarations*: `use` imports, parent / interface / trait
//! FQCNs, and named-object type hints on properties, params and return types.
//! Body-level bare-FQN references are deliberately excluded — they live in the
//! reference index and are merged in by `AnalysisSession::dependency_graph`.
//!
//! Keeping the tracked structural fact at symbol level avoids making Salsa
//! cache file-path projections that really belong to the workspace symbol
//! index. [`file_structural_deps`] remains as a compatibility projection for
//! callers that want paths directly.

use std::sync::Arc;

use mir_codebase::definitions::StubSlice;
use mir_types::{atomic::Atomic, Name, Type};
use rustc_hash::FxHashSet;

use super::*;

/// Symbols that `file`'s declarations mention. Sorted for deterministic memo
/// equality. These are lookup keys for [`MirDatabase::symbol_defining_file`].
#[salsa::tracked]
pub fn file_structural_symbols(db: &dyn MirDatabase, file: SourceFile) -> Arc<[Name]> {
    let defs = crate::db::collect_file_definitions(db, file);
    let mut symbols: FxHashSet<Name> = FxHashSet::default();
    collect_structural_dep_symbols(&defs.slice, &mut |symbol| {
        symbols.insert(symbol);
    });

    let mut sorted: Vec<Name> = symbols.into_iter().collect();
    sorted.sort();
    sorted.into()
}

/// Files that `file`'s declarations depend on. Sorted for deterministic memo
/// equality. Self-edges are excluded.
#[salsa::tracked]
pub fn file_structural_deps(db: &dyn MirDatabase, file: SourceFile) -> Arc<[Arc<str>]> {
    let path = file.path(db);
    let mut targets: FxHashSet<Arc<str>> = FxHashSet::default();

    for symbol in file_structural_symbols(db, file).iter() {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            if defining_file.as_ref() != path.as_ref() {
                targets.insert(defining_file);
            }
        }
    }

    let mut sorted: Vec<Arc<str>> = targets.into_iter().collect();
    sorted.sort();
    sorted.into()
}

fn collect_structural_dep_symbols(slice: &StubSlice, mut add_symbol: impl FnMut(Name)) {
    for c in slice.classes.iter() {
        if let Some(parent) = &c.parent {
            add_symbol(Name::from(parent.clone()));
        }
        for interface in c.interfaces.iter() {
            add_symbol(Name::from(interface.clone()));
        }
        for trait_fqcn in c.traits.iter() {
            add_symbol(Name::from(trait_fqcn.clone()));
        }
        for prop in c.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_symbol);
            }
        }
        for method in c.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_symbol);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_symbol);
            }
        }
    }

    for i in slice.interfaces.iter() {
        for extended in i.extends.iter() {
            add_symbol(Name::from(extended.clone()));
        }
        for prop in i.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_symbol);
            }
        }
        for method in i.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_symbol);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_symbol);
            }
        }
    }

    for t in slice.traits.iter() {
        for trait_fqcn in t.traits.iter() {
            add_symbol(Name::from(trait_fqcn.clone()));
        }
        for prop in t.own_properties.values() {
            if let Some(ty) = &prop.ty {
                collect_named_object_atoms(ty, &mut add_symbol);
            }
        }
        for method in t.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_symbol);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_symbol);
            }
        }
    }

    for e in slice.enums.iter() {
        for interface in e.interfaces.iter() {
            add_symbol(Name::from(interface.clone()));
        }
        for trait_fqcn in e.traits.iter() {
            add_symbol(Name::from(trait_fqcn.clone()));
        }
        for method in e.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    collect_named_object_atoms(ty.as_ref(), &mut add_symbol);
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                collect_named_object_atoms(rt, &mut add_symbol);
            }
        }
    }

    for f in slice.functions.iter() {
        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                collect_named_object_atoms(ty.as_ref(), &mut add_symbol);
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            collect_named_object_atoms(rt, &mut add_symbol);
        }
    }

    for fqcn in slice.imports.values() {
        add_symbol(*fqcn);
    }
}

fn collect_named_object_atoms(ty: &Type, add_symbol: &mut impl FnMut(Name)) {
    for atomic in ty.types.iter() {
        if let Atomic::TNamedObject { fqcn, .. } = atomic {
            add_symbol(*fqcn);
        }
    }
}
