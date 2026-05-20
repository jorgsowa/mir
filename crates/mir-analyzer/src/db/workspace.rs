//! Pull-path workspace enumeration.
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
//!
//! ## Incremental edit performance
//!
//! Two mechanisms together keep `workspace_symbol_index` cheap on project-file
//! edits:
//!
//! 1. **Salsa durability short-circuit** — vendor and built-in stub files are
//!    registered with `Durability::HIGH`.  When a LOW-durability project file
//!    changes, salsa's per-durability revision counter proves that every HIGH-
//!    durability dep is still valid without walking each one, reducing O(N)
//!    dep-verification to O(project_files_only).
//!
//! 2. **Name-only intermediary** — `workspace_symbol_index` calls
//!    `collect_file_declarations` (not `collect_file_definitions` directly).
//!    `collect_file_declarations` has a name-only `PartialEq`: body-only edits
//!    (method implementations, docblocks, whitespace) do NOT propagate to
//!    `workspace_symbol_index`, so it is not re-run unless declared names change.

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
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
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
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
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

// ---------------------------------------------------------------------------
// FileDeclarations — name-only intermediary for workspace_symbol_index
// ---------------------------------------------------------------------------

/// Name-only summary of the declarations in one source file.
///
/// `PartialEq` compares only the declared names (not body content), so salsa
/// skips re-running `workspace_symbol_index` when a file's method bodies
/// change but its set of class / function / constant names is unchanged.
#[derive(Clone)]
pub struct FileDeclarations {
    /// `(lowercased_fqcn, SymbolLoc)` for every class-like symbol.
    pub class_like: Vec<(String, SymbolLoc)>,
    /// `(lowercased_fqn, SymbolLoc)` for every function.
    pub functions: Vec<(String, SymbolLoc)>,
    /// `(name, SymbolLoc)` for every constant (case-sensitive key).
    pub constants: Vec<(String, SymbolLoc)>,
}

impl PartialEq for FileDeclarations {
    fn eq(&self, other: &Self) -> bool {
        self.class_like.len() == other.class_like.len()
            && self
                .class_like
                .iter()
                .zip(&other.class_like)
                .all(|(a, b)| a.0 == b.0)
            && self.functions.len() == other.functions.len()
            && self
                .functions
                .iter()
                .zip(&other.functions)
                .all(|(a, b)| a.0 == b.0)
            && self.constants.len() == other.constants.len()
            && self
                .constants
                .iter()
                .zip(&other.constants)
                .all(|(a, b)| a.0 == b.0)
    }
}

unsafe impl salsa::Update for FileDeclarations {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

/// Extract the declared names from one source file without exposing body
/// content.  Used as the input to `workspace_symbol_index` so that body-only
/// edits don't propagate to the workspace-wide FQCN index.
#[salsa::tracked]
pub fn collect_file_declarations(db: &dyn MirDatabase, file: SourceFile) -> FileDeclarations {
    let defs = collect_file_definitions(db, file);
    let mut class_like = Vec::new();
    let mut functions = Vec::new();
    let mut constants = Vec::new();

    for (idx, c) in defs.slice.classes.iter().enumerate() {
        class_like.push((c.fqcn.to_ascii_lowercase(), SymbolLoc::Class { file, idx }));
    }
    for (idx, i) in defs.slice.interfaces.iter().enumerate() {
        class_like.push((
            i.fqcn.to_ascii_lowercase(),
            SymbolLoc::Interface { file, idx },
        ));
    }
    for (idx, t) in defs.slice.traits.iter().enumerate() {
        class_like.push((t.fqcn.to_ascii_lowercase(), SymbolLoc::Trait { file, idx }));
    }
    for (idx, e) in defs.slice.enums.iter().enumerate() {
        class_like.push((e.fqcn.to_ascii_lowercase(), SymbolLoc::Enum { file, idx }));
    }
    for (idx, f) in defs.slice.functions.iter().enumerate() {
        functions.push((
            f.fqn.to_ascii_lowercase(),
            SymbolLoc::Function { file, idx },
        ));
    }
    for (idx, (name, _)) in defs.slice.constants.iter().enumerate() {
        constants.push((name.to_string(), SymbolLoc::Constant { file, idx }));
    }

    FileDeclarations {
        class_like,
        functions,
        constants,
    }
}

/// Symbol kind tag + slice index. Building one is a single integer tag
/// (no storage cloning). Resolution via `collect_file_definitions(file)`
/// goes through a salsa-memoized query → direct slice access.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SymbolLoc {
    Class { file: SourceFile, idx: usize },
    Interface { file: SourceFile, idx: usize },
    Trait { file: SourceFile, idx: usize },
    Enum { file: SourceFile, idx: usize },
    Function { file: SourceFile, idx: usize },
    Constant { file: SourceFile, idx: usize },
}

/// Salsa input singleton holding the pre-built [`WorkspaceSymbolIndex`].
///
/// Written imperatively by `MirDb::rebuild_workspace_symbol_index` after
/// batch file loads and after incremental edits that change declared names.
/// Reading `singleton.index(db)` inside a tracked query creates exactly
/// ONE tracked dep (this input field) with `Durability::HIGH`, so on
/// project-file body edits (LOW durability) salsa short-circuits in O(1)
/// instead of walking the O(N_files) dep list that `workspace_symbol_index`
/// (the tracked fn) accumulates.
///
/// Falls back to `workspace_symbol_index(db)` when the singleton has not
/// yet been populated (e.g. in unit tests that never call rebuild).
#[salsa::input]
pub struct WorkspaceSymbolIndexSingleton {
    pub index: WorkspaceSymbolIndex,
}

/// Lightweight FQCN→location index. Built lazily per workspace revision;
/// holds *no* storage data — just (file, slice_index) tags.
///
/// Replaces the 3-deep `resolve_fqcn_to_path → lookup_source_file →
/// class_in_file` query stack with one O(1) map lookup. Storage is fetched
/// on-demand via the already-memoized `collect_file_definitions(file)`.
#[derive(Clone, Default)]
pub struct WorkspaceSymbolIndex {
    /// Class / interface / trait / enum FQCN (lowered) → location.
    pub class_like: Arc<FxHashMap<String, SymbolLoc>>,
    /// Function FQN (lowered) → location.
    pub functions: Arc<FxHashMap<String, SymbolLoc>>,
    /// Constant FQN (case-sensitive) → location.
    pub constants: Arc<FxHashMap<String, SymbolLoc>>,
}

impl PartialEq for WorkspaceSymbolIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.class_like, &other.class_like)
            && Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.constants, &other.constants)
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
    // workspace_revision() is always Some — init_workspace_revision() is called
    // at SharedDb::new() so this query always reads the revision and salsa can
    // properly invalidate it when files are added or removed.
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut class_like: FxHashMap<String, SymbolLoc> = FxHashMap::default();
    let mut functions: FxHashMap<String, SymbolLoc> = FxHashMap::default();
    let mut constants: FxHashMap<String, SymbolLoc> = FxHashMap::default();

    // First pass: all files with or_insert (first-write-wins for native stubs).
    // collect_file_declarations has a name-only PartialEq so body-only edits
    // don't propagate to this index.
    for file in files.iter() {
        let decls = collect_file_declarations(db, *file);
        for (key, loc) in &decls.class_like {
            class_like.entry(key.clone()).or_insert(*loc);
        }
        for (key, loc) in &decls.functions {
            functions.entry(key.clone()).or_insert(*loc);
        }
        for (key, loc) in &decls.constants {
            constants.entry(key.clone()).or_insert(*loc);
        }
    }

    // Second pass: user stubs overwrite native stubs for the same symbol.
    for file in db.user_stub_source_files().iter() {
        let decls = collect_file_declarations(db, *file);
        for (key, loc) in decls.class_like {
            class_like.insert(key, loc);
        }
        for (key, loc) in decls.functions {
            functions.insert(key, loc);
        }
        for (key, loc) in decls.constants {
            constants.insert(key, loc);
        }
    }

    WorkspaceSymbolIndex {
        class_like: Arc::new(class_like),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
    }
}

#[salsa::tracked]
pub fn workspace_fqcn_index(db: &dyn MirDatabase) -> FqcnIndex {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
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
    // User stubs override native stubs for the same symbol.
    for file in db.user_stub_source_files().iter() {
        let defs = collect_file_definitions(db, *file);
        for c in defs.slice.classes.iter() {
            classes.insert(c.fqcn.to_ascii_lowercase(), *file);
        }
        for i in defs.slice.interfaces.iter() {
            classes.insert(i.fqcn.to_ascii_lowercase(), *file);
        }
        for t in defs.slice.traits.iter() {
            classes.insert(t.fqcn.to_ascii_lowercase(), *file);
        }
        for e in defs.slice.enums.iter() {
            classes.insert(e.fqcn.to_ascii_lowercase(), *file);
        }
        for f in defs.slice.functions.iter() {
            functions.insert(f.fqn.to_ascii_lowercase(), *file);
        }
        for (name, _) in defs.slice.constants.iter() {
            constants.insert(name.to_string(), *file);
        }
    }
    FqcnIndex {
        classes: Arc::new(classes),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
    }
}

// ---------------------------------------------------------------------------
// workspace_global_vars
// ---------------------------------------------------------------------------

/// Name → type map for every PHP global variable defined across all
/// registered source files.  Built from `global_vars` entries in each
/// file's `StubSlice`; the PHP standard stubs contribute the predefined
/// superglobals (`$_SERVER`, `$_GET`, …).
///
/// `Arc::ptr_eq` is used for change detection so salsa skips re-running
/// dependents when the same map is produced across revisions.
#[derive(Clone, Default, Debug)]
pub struct GlobalVarMap(pub Arc<FxHashMap<Arc<str>, mir_types::Union>>);

impl PartialEq for GlobalVarMap {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

unsafe impl salsa::Update for GlobalVarMap {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

/// Aggregate all `global_vars` entries from every registered `SourceFile`.
/// Tracked so salsa invalidates it when any file's text changes.
#[salsa::tracked]
pub fn workspace_global_vars(db: &dyn MirDatabase) -> GlobalVarMap {
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);

    let files = db.all_source_files();
    let mut out: FxHashMap<Arc<str>, mir_types::Union> = FxHashMap::default();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for (name, ty) in &defs.slice.global_vars {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.entry(gname).or_insert_with(|| ty.clone());
        }
    }
    GlobalVarMap(Arc::new(out))
}
