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
