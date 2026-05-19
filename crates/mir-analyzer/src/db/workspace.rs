//! Phase 4 enabler — pull-path workspace enumeration.
//!
//! Allows tracked queries to iterate everything defined across all
//! registered `SourceFile` inputs without going through the push-based
//! `MirDb::active_*_fqcns` methods. Needed by `class.rs::analyze_all`,
//! `dead_code.rs`, and `project.rs` after Phase 5 deletes those.
//!
//! ## Design
//!
//! A single `WorkspaceRevision` salsa input holds a monotonic counter
//! bumped whenever a file is added or removed (`upsert_source_file` /
//! `remove_source_file`). Edits to existing files don't bump the
//! revision; they invalidate `collect_file_definitions` directly.
//!
//! Tracked aggregators (`workspace_classes`, `workspace_functions`)
//! read `WorkspaceRevision::revision` to anchor on the set of files,
//! then enumerate via the off-salsa `source_files` registry and demand
//! `collect_file_definitions` per file. Salsa invalidates the aggregator
//! when either the file set or any file's content changes.

use std::sync::Arc;

use mir_codebase::storage::{
    ClassStorage, EnumStorage, FunctionStorage, InterfaceStorage, TraitStorage,
};
use mir_types::Union;
use rustc_hash::FxHashMap;

use crate::db::{collect_file_definitions, MirDatabase, SourceFile};

/// Singleton salsa input — revision counter for workspace add/remove
/// events. The actual list of [`crate::db::SourceFile`]s lives off-salsa
/// on `MirDb::source_files`.
#[salsa::input]
pub struct WorkspaceRevision {
    pub revision: u64,
}

/// Iterate over every class FQCN defined in any registered SourceFile.
///
/// Tracked: invalidates when the workspace file set changes
/// (`WorkspaceRevision`) or any file's text changes (via
/// `collect_file_definitions`). Result is `Arc<[Arc<str>]>` so salsa
/// can ptr_eq-compare for cheap skip.
#[salsa::tracked]
pub fn workspace_classes(db: &dyn MirDatabase) -> Arc<[Arc<str>]> {
    let Some(rev) = db.workspace_revision() else {
        return Arc::from([]);
    };
    // Anchor on the revision so file add/remove invalidates this query.
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: Vec<Arc<str>> = Vec::new();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for c in defs.slice.classes.iter() {
            out.push(c.fqcn.clone());
        }
        for i in defs.slice.interfaces.iter() {
            out.push(i.fqcn.clone());
        }
        for t in defs.slice.traits.iter() {
            out.push(t.fqcn.clone());
        }
        for e in defs.slice.enums.iter() {
            out.push(e.fqcn.clone());
        }
    }
    Arc::from(out)
}

/// Iterate over every function FQN defined in any registered SourceFile.
#[salsa::tracked]
pub fn workspace_functions(db: &dyn MirDatabase) -> Arc<[Arc<str>]> {
    let Some(rev) = db.workspace_revision() else {
        return Arc::from([]);
    };
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: Vec<Arc<str>> = Vec::new();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for f in defs.slice.functions.iter() {
            out.push(f.fqn.clone());
        }
    }
    Arc::from(out)
}

/// O(1) FQCN→SourceFile index across the workspace. Used by
/// `source_file_for_fqcn` as the resolver-miss fallback (project-only
/// classes / no-resolver test fixtures).
///
/// Class / interface / trait / enum / function FQNs are stored
/// case-insensitively (lowered keys); constant FQNs case-sensitively.
#[derive(Clone, Default)]
pub struct FqcnIndex {
    pub classes: Arc<FxHashMap<String, SourceFile>>,
    pub functions: Arc<FxHashMap<String, SourceFile>>,
    pub constants: Arc<FxHashMap<String, SourceFile>>,
}

impl PartialEq for FqcnIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.classes, &other.classes)
            && Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.constants, &other.constants)
    }
}

unsafe impl salsa::Update for FqcnIndex {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

// ---------------------------------------------------------------------------
// WorkspaceSymbolIndex — Phase 6 hot-path lookup map.
//
// One salsa-tracked query builds a comprehensive FQCN → storage map across
// every registered SourceFile. Pass-2 takes the `Arc<...>` once and reads
// O(1) thereafter, bypassing the 3-4-deep nested tracked-query stack the
// previous design paid for every method/class lookup.
//
// Keys are case-folded for class / interface / trait / enum / function
// (PHP semantics); constants stay case-sensitive.
// ---------------------------------------------------------------------------

/// Hot-path bundle holding the materialised storage for every symbol in the
/// workspace. Built once per workspace_revision × file content; Arc-shared
/// so consumers clone-and-read without locks.
#[derive(Clone, Default)]
pub struct WorkspaceSymbolIndex {
    pub classes: Arc<FxHashMap<String, Arc<ClassStorage>>>,
    pub interfaces: Arc<FxHashMap<String, Arc<InterfaceStorage>>>,
    pub traits: Arc<FxHashMap<String, Arc<TraitStorage>>>,
    pub enums: Arc<FxHashMap<String, Arc<EnumStorage>>>,
    pub functions: Arc<FxHashMap<String, Arc<FunctionStorage>>>,
    pub constants: Arc<FxHashMap<String, Arc<Union>>>,
    /// FQCN (lowered) → defining file. Lets callers route "where is X defined?"
    /// in O(1) without iterating the workspace.
    pub class_files: Arc<FxHashMap<String, SourceFile>>,
    pub function_files: Arc<FxHashMap<String, SourceFile>>,
    pub constant_files: Arc<FxHashMap<String, SourceFile>>,
}

impl PartialEq for WorkspaceSymbolIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.classes, &other.classes)
            && Arc::ptr_eq(&self.interfaces, &other.interfaces)
            && Arc::ptr_eq(&self.traits, &other.traits)
            && Arc::ptr_eq(&self.enums, &other.enums)
            && Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.constants, &other.constants)
            && Arc::ptr_eq(&self.class_files, &other.class_files)
            && Arc::ptr_eq(&self.function_files, &other.function_files)
            && Arc::ptr_eq(&self.constant_files, &other.constant_files)
    }
}

unsafe impl salsa::Update for WorkspaceSymbolIndex {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

#[salsa::tracked]
pub fn workspace_symbol_index(db: &dyn MirDatabase) -> WorkspaceSymbolIndex {
    let Some(rev) = db.workspace_revision() else {
        return WorkspaceSymbolIndex::default();
    };
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut classes: FxHashMap<String, Arc<ClassStorage>> = FxHashMap::default();
    let mut interfaces: FxHashMap<String, Arc<InterfaceStorage>> = FxHashMap::default();
    let mut traits_: FxHashMap<String, Arc<TraitStorage>> = FxHashMap::default();
    let mut enums: FxHashMap<String, Arc<EnumStorage>> = FxHashMap::default();
    let mut functions: FxHashMap<String, Arc<FunctionStorage>> = FxHashMap::default();
    let mut constants: FxHashMap<String, Arc<Union>> = FxHashMap::default();
    let mut class_files: FxHashMap<String, SourceFile> = FxHashMap::default();
    let mut function_files: FxHashMap<String, SourceFile> = FxHashMap::default();
    let mut constant_files: FxHashMap<String, SourceFile> = FxHashMap::default();

    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for c in defs.slice.classes.iter() {
            let key = c.fqcn.to_ascii_lowercase();
            classes
                .entry(key.clone())
                .or_insert_with(|| Arc::new(c.clone()));
            class_files.entry(key).or_insert(*file);
        }
        for i in defs.slice.interfaces.iter() {
            let key = i.fqcn.to_ascii_lowercase();
            interfaces
                .entry(key.clone())
                .or_insert_with(|| Arc::new(i.clone()));
            class_files.entry(key).or_insert(*file);
        }
        for t in defs.slice.traits.iter() {
            let key = t.fqcn.to_ascii_lowercase();
            traits_
                .entry(key.clone())
                .or_insert_with(|| Arc::new(t.clone()));
            class_files.entry(key).or_insert(*file);
        }
        for e in defs.slice.enums.iter() {
            let key = e.fqcn.to_ascii_lowercase();
            enums
                .entry(key.clone())
                .or_insert_with(|| Arc::new(e.clone()));
            class_files.entry(key).or_insert(*file);
        }
        for f in defs.slice.functions.iter() {
            let key = f.fqn.to_ascii_lowercase();
            functions
                .entry(key.clone())
                .or_insert_with(|| Arc::new(f.clone()));
            function_files.entry(key).or_insert(*file);
        }
        for (name, ty) in defs.slice.constants.iter() {
            let key = name.to_string();
            constants
                .entry(key.clone())
                .or_insert_with(|| Arc::new(ty.clone()));
            constant_files.entry(key).or_insert(*file);
        }
    }

    WorkspaceSymbolIndex {
        classes: Arc::new(classes),
        interfaces: Arc::new(interfaces),
        traits: Arc::new(traits_),
        enums: Arc::new(enums),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
        class_files: Arc::new(class_files),
        function_files: Arc::new(function_files),
        constant_files: Arc::new(constant_files),
    }
}

#[salsa::tracked]
pub fn workspace_fqcn_index(db: &dyn MirDatabase) -> FqcnIndex {
    let Some(rev) = db.workspace_revision() else {
        return FqcnIndex::default();
    };
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut classes: FxHashMap<String, SourceFile> = FxHashMap::default();
    let mut functions: FxHashMap<String, SourceFile> = FxHashMap::default();
    let mut constants: FxHashMap<String, SourceFile> = FxHashMap::default();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for c in defs.slice.classes.iter() {
            classes.entry(c.fqcn.to_ascii_lowercase()).or_insert(*file);
        }
        for i in defs.slice.interfaces.iter() {
            classes.entry(i.fqcn.to_ascii_lowercase()).or_insert(*file);
        }
        for t in defs.slice.traits.iter() {
            classes.entry(t.fqcn.to_ascii_lowercase()).or_insert(*file);
        }
        for e in defs.slice.enums.iter() {
            classes.entry(e.fqcn.to_ascii_lowercase()).or_insert(*file);
        }
        for f in defs.slice.functions.iter() {
            functions.entry(f.fqn.to_ascii_lowercase()).or_insert(*file);
        }
        for (name, _) in defs.slice.constants.iter() {
            constants.entry(name.to_string()).or_insert(*file);
        }
    }
    FqcnIndex {
        classes: Arc::new(classes),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
    }
}
