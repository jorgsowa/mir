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

use mir_types::Name;
use rustc_hash::FxHashMap;

use crate::db::{collect_file_definitions, MirDatabase, SourceFile};

/// Singleton salsa input — revision counter for workspace add/remove
/// events. The actual list of [`crate::db::SourceFile`]s lives off-salsa
/// on `MirDbStorage::source_files`.
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
        let decls = collect_file_declarations(db, *file);
        for decl in &decls.class_like {
            out.push(decl.symbol.clone());
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
        let decls = collect_file_declarations(db, *file);
        for decl in &decls.functions {
            out.push(decl.symbol.clone());
        }
    }
    Arc::from(out)
}

// ---------------------------------------------------------------------------
// WorkspaceSymbolIndex — Phase 6 hot-path lookup map.
//
// One salsa-tracked query builds a comprehensive FQCN → storage map across
// every registered SourceFile. body-analysis takes the `Arc<...>` once and reads
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
    /// Every class-like declaration, keyed case-insensitively for lookup while
    /// preserving the original symbol for aggregation queries.
    pub class_like: Vec<DeclaredSymbol>,
    /// Every function declaration, keyed case-insensitively for lookup while
    /// preserving the original symbol for aggregation queries.
    pub functions: Vec<DeclaredSymbol>,
    /// Every constant declaration (case-sensitive key).
    pub constants: Vec<DeclaredSymbol>,
}

impl PartialEq for FileDeclarations {
    fn eq(&self, other: &Self) -> bool {
        self.class_like.len() == other.class_like.len()
            && self
                .class_like
                .iter()
                .zip(&other.class_like)
                .all(|(a, b)| a.key == b.key)
            && self.functions.len() == other.functions.len()
            && self
                .functions
                .iter()
                .zip(&other.functions)
                .all(|(a, b)| a.key == b.key)
            && self.constants.len() == other.constants.len()
            && self
                .constants
                .iter()
                .zip(&other.constants)
                .all(|(a, b)| a.key == b.key)
    }
}

/// Compact key-only declaration snapshot for imperative workspace-index
/// maintenance. Avoids duplicating `symbol` and `loc` payloads per file while
/// preserving body-only edit no-op detection and incremental subtract logic.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub(crate) struct FileDeclSnapshot {
    pub class_like: Box<[Name]>,
    pub functions: Box<[Name]>,
    pub constants: Box<[Name]>,
}

impl FileDeclSnapshot {
    pub fn from_decls(decls: &FileDeclarations) -> Self {
        Self {
            class_like: decls.class_like.iter().map(|decl| decl.key).collect(),
            functions: decls.functions.iter().map(|decl| decl.key).collect(),
            constants: decls.constants.iter().map(|decl| decl.key).collect(),
        }
    }

    pub fn matches_decls(&self, decls: &FileDeclarations) -> bool {
        self.class_like.len() == decls.class_like.len()
            && self
                .class_like
                .iter()
                .zip(&decls.class_like)
                .all(|(key, decl)| *key == decl.key)
            && self.functions.len() == decls.functions.len()
            && self
                .functions
                .iter()
                .zip(&decls.functions)
                .all(|(key, decl)| *key == decl.key)
            && self.constants.len() == decls.constants.len()
            && self
                .constants
                .iter()
                .zip(&decls.constants)
                .all(|(key, decl)| *key == decl.key)
    }
}

#[derive(Clone)]
pub struct DeclaredSymbol {
    pub key: Name,
    pub symbol: Arc<str>,
    pub loc: SymbolLoc,
}

struct FileDeclarationProjection {
    class_like: Vec<DeclaredSymbol>,
    functions: Vec<DeclaredSymbol>,
    constants: Vec<DeclaredSymbol>,
    structural_symbols: Vec<Arc<str>>,
}

/// Extract the declared names from one source file without exposing body
/// content.  Used as the input to `workspace_symbol_index` so that body-only
/// edits don't propagate to the workspace-wide FQCN index.
///
/// Deliberately NOT `lru`-capped (unlike `collect_file_definitions`): the
/// result is a few name/loc pairs per file, and `workspace_symbol_index`
/// walks EVERY source file through this query on each rebuild. With a cap
/// smaller than the workspace (a 15K-file session vs the old `lru = 4096`),
/// each walk re-executed the evicted majority — and since the walk happens
/// after every `workspace_revision` bump (each file ingested by a reference
/// query's prepare loop), a single cold query degraded into
/// O(prepared_files × workspace) re-parsing, measured at ~28s wall on a
/// 15K-file workspace. Uncapped, a walk is a cheap memo validation.
#[salsa::tracked]
pub fn collect_file_declarations(db: &dyn MirDatabase, file: SourceFile) -> FileDeclarations {
    let defs = collect_file_definitions(db, file);
    decls_from_slice(&defs.slice, file)
}

/// The projection behind [`collect_file_declarations`], callable on a raw
/// [`StubSlice`]. The warm-start path uses this to seed the workspace symbol
/// index from disk-cached slices without pulling `collect_file_definitions`;
/// sharing one projection keeps the seeded snapshots byte-identical to what
/// the tracked query would later compute for unchanged text.
pub fn decls_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> FileDeclarations {
    let projection = declaration_projection_from_slice(slice, file);
    FileDeclarations {
        class_like: projection.class_like,
        functions: projection.functions,
        constants: projection.constants,
    }
}

pub(crate) fn structural_symbols_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> Arc<[Arc<str>]> {
    declaration_projection_from_slice(slice, file)
        .structural_symbols
        .into()
}

fn declaration_projection_from_slice(
    slice: &mir_codebase::definitions::StubSlice,
    file: SourceFile,
) -> FileDeclarationProjection {
    let mut class_like = Vec::new();
    let mut functions = Vec::new();
    let mut constants = Vec::new();
    let mut structural_symbols = Vec::new();

    let push_named_objects = |out: &mut Vec<Arc<str>>, union: &mir_types::Type| {
        out.extend(union.types.iter().filter_map(|atomic| match atomic {
            mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => {
                Some(Arc::<str>::from(fqcn.as_str()))
            }
            _ => None,
        }));
    };

    // Pre-lowercase FQCNs once at collection time and intern via Name so
    // downstream lookups (find_class_like, inferred_*_demand) can hash u64
    // pointers instead of byte-by-byte strings.
    for (idx, c) in slice.classes.iter().enumerate() {
        class_like.push(DeclaredSymbol {
            key: Name::new(c.fqcn.as_ref()).ascii_lowercase(),
            symbol: c.fqcn.clone(),
            loc: SymbolLoc::Class { file, idx },
        });
        if let Some(parent) = &c.parent {
            structural_symbols.push(parent.clone());
        }
        structural_symbols.extend(c.interfaces.iter().cloned());
        structural_symbols.extend(c.traits.iter().cloned());
        for prop in c.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in c.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, i) in slice.interfaces.iter().enumerate() {
        class_like.push(DeclaredSymbol {
            key: Name::new(i.fqcn.as_ref()).ascii_lowercase(),
            symbol: i.fqcn.clone(),
            loc: SymbolLoc::Interface { file, idx },
        });
        structural_symbols.extend(i.extends.iter().cloned());
        for prop in i.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in i.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, t) in slice.traits.iter().enumerate() {
        class_like.push(DeclaredSymbol {
            key: Name::new(t.fqcn.as_ref()).ascii_lowercase(),
            symbol: t.fqcn.clone(),
            loc: SymbolLoc::Trait { file, idx },
        });
        structural_symbols.extend(t.traits.iter().cloned());
        for prop in t.own_properties.values() {
            if let Some(ty) = &prop.ty {
                push_named_objects(&mut structural_symbols, ty);
            }
        }
        for method in t.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, e) in slice.enums.iter().enumerate() {
        class_like.push(DeclaredSymbol {
            key: Name::new(e.fqcn.as_ref()).ascii_lowercase(),
            symbol: e.fqcn.clone(),
            loc: SymbolLoc::Enum { file, idx },
        });
        structural_symbols.extend(e.interfaces.iter().cloned());
        structural_symbols.extend(e.traits.iter().cloned());
        for method in e.own_methods.values() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    push_named_objects(&mut structural_symbols, ty.as_ref());
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                push_named_objects(&mut structural_symbols, rt);
            }
        }
    }
    for (idx, f) in slice.functions.iter().enumerate() {
        functions.push(DeclaredSymbol {
            key: Name::new(f.fqn.as_ref()).ascii_lowercase(),
            symbol: f.fqn.clone(),
            loc: SymbolLoc::Function { file, idx },
        });
        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                push_named_objects(&mut structural_symbols, ty.as_ref());
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            push_named_objects(&mut structural_symbols, rt);
        }
    }
    for (idx, (name, _)) in slice.constants.iter().enumerate() {
        constants.push(DeclaredSymbol {
            key: Name::new(name.as_ref()),
            symbol: name.clone(),
            loc: SymbolLoc::Constant { file, idx },
        });
    }
    structural_symbols.extend(
        slice
            .imports
            .values()
            .map(|fqcn| Arc::<str>::from(fqcn.as_str())),
    );

    FileDeclarationProjection {
        class_like,
        functions,
        constants,
        structural_symbols,
    }
}

/// Name kind tag + slice index. Building one is a single integer tag
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

impl SymbolLoc {
    /// The `SourceFile` this symbol is declared in.
    pub fn file(&self) -> SourceFile {
        match self {
            SymbolLoc::Class { file, .. }
            | SymbolLoc::Interface { file, .. }
            | SymbolLoc::Trait { file, .. }
            | SymbolLoc::Enum { file, .. }
            | SymbolLoc::Function { file, .. }
            | SymbolLoc::Constant { file, .. } => *file,
        }
    }
}

/// Precedence tier for a symbol declaration. The imperative build paths
/// (full rebuild, seed, incremental merge) drive it through `tier_insert`;
/// the tracked `workspace_symbol_index` fallback encodes the same rule as
/// ordered passes (equivalence pinned by
/// `tracked_walk_matches_imperative_rebuild`):
///
/// 1. `NativeStub` — built-in PHP stub files (`path` starts with `stubs/`);
///    first-write-wins among themselves.
/// 2. `UserFile` — analyzed project / vendor files; overwrite native stubs.
/// 3. `UserStub` — user-provided stub files; overwrite everything.
///
/// Stored implicitly (derived from a [`SymbolLoc`]'s file) so the incremental
/// merge in `merge_precomputed_into_workspace_index` can decide precedence per-insert
/// regardless of the order chunks arrive in during background indexing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SymbolTier {
    NativeStub = 0,
    UserFile = 1,
    UserStub = 2,
}

/// Per-symbol-kind declarer counts maintained alongside the workspace symbol
/// index singleton. `counts[name]` = number of registered files that declare
/// `name`. Used by the incremental subtract path
/// ([`crate::db::MirDbStorage::update_workspace_index_for_file`]) to decide
/// whether removing a file's declaration of `name` is safe (count drops to 0)
/// or ambiguous (another file still declares it → fall back to full rebuild).
#[derive(Default, Clone)]
pub struct IndexDeclCounts {
    pub class_like: FxHashMap<Name, u32>,
    pub functions: FxHashMap<Name, u32>,
    pub constants: FxHashMap<Name, u32>,
}

/// Salsa input singleton holding the pre-built [`WorkspaceSymbolIndex`].
///
/// Written imperatively by `MirDbStorage::rebuild_workspace_symbol_index` after
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
    /// Monotonic counter bumped in lockstep with `index` (in
    /// `MirDbStorage::set_workspace_index`, the single write chokepoint).
    ///
    /// Lets the frozen-then-borrow fast path register a salsa dependency on
    /// the workspace index **without** cloning the three `Arc<FxHashMap>`s in
    /// `index`: a frozen reader reads this `Copy` field (a real salsa input
    /// read, so it joins the active query's dep set) and then borrows the
    /// pre-snapshotted maps. Tracked callers that resolve a class through the
    /// frozen path (e.g. `class_ancestors_by_fqcn`) therefore still get
    /// invalidated when the index mutates — without this, a negative memo
    /// (class-not-found) computed pre-load would never be re-run post-load.
    pub revision: u64,
}

/// Lightweight FQCN→location index. Built lazily per workspace revision;
/// holds *no* storage data — just (file, slice_index) tags.
///
/// Replaces the 3-deep `resolve_fqcn_to_path → lookup_source_file →
/// class_in_file` query stack with one O(1) map lookup. Storage is fetched
/// on-demand via the already-memoized `collect_file_definitions(file)`.
#[derive(Clone, Default)]
pub struct WorkspaceSymbolIndex {
    /// Class / interface / trait / enum FQCN (lowercased Name) → location.
    ///
    /// Keys are `Name` rather than `String` so lookups from the body-analysis hot
    /// path are u64 pointer-eq comparisons instead of byte-by-byte string
    /// hashes — and so the caller doesn't have to allocate a `String` to do
    /// the lookup. The lowercased symbol is computed once at index-build
    /// time and reused by all lookups via `Name::ascii_lowercase()` (which
    /// is itself memoized).
    pub class_like: Arc<FxHashMap<Name, SymbolLoc>>,
    /// Function FQN (lowercased Name) → location.
    pub functions: Arc<FxHashMap<Name, SymbolLoc>>,
    /// Constant FQN (case-sensitive Name) → location.
    pub constants: Arc<FxHashMap<Name, SymbolLoc>>,
    /// Short class/interface/trait/enum name (lowercased, no namespace) →
    /// every FQCN (lowercased) in `class_like` sharing it. A short name
    /// shared across namespaces (e.g. Laravel's many `Factory` classes) is
    /// otherwise invisible to a host that only has `class_like`'s FQCN keys —
    /// this is what lets a host resolve/disambiguate a bare class-name token
    /// without maintaining its own name→candidates map or falling back to a
    /// text scan. No precedence/tier concept at this level (unlike
    /// `class_like`): a bucket entry exists exactly as long as its FQCN
    /// exists in `class_like`, regardless of which tier won that FQCN.
    pub class_like_by_short_name: Arc<FxHashMap<Name, Vec<Name>>>,
}

impl PartialEq for WorkspaceSymbolIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.class_like, &other.class_like)
            && Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.constants, &other.constants)
            && Arc::ptr_eq(
                &self.class_like_by_short_name,
                &other.class_like_by_short_name,
            )
    }
}

/// Pure constructor shared by every path that materializes a fresh
/// [`WorkspaceSymbolIndex`] from already-built maps.
pub fn build_workspace_symbol_index(
    class_like: FxHashMap<Name, SymbolLoc>,
    functions: FxHashMap<Name, SymbolLoc>,
    constants: FxHashMap<Name, SymbolLoc>,
    class_like_by_short_name: FxHashMap<Name, Vec<Name>>,
) -> WorkspaceSymbolIndex {
    WorkspaceSymbolIndex {
        class_like: Arc::new(class_like),
        functions: Arc::new(functions),
        constants: Arc::new(constants),
        class_like_by_short_name: Arc::new(class_like_by_short_name),
    }
}

/// The short-name key for an already-lowercased FQCN `Name` — the part after
/// the last `\`, itself already lowercase (rsplit doesn't change case).
/// Shared by every `WorkspaceSymbolIndex` build/incremental-update path so
/// `class_like_by_short_name` stays keyed consistently everywhere.
pub fn short_name_key(fqcn_lower: Name) -> Name {
    let s = fqcn_lower.as_str().trim_start_matches('\\');
    match s.rsplit_once('\\') {
        Some((_, short)) => Name::new(short),
        None => Name::new(s),
    }
}

/// Return the workspace symbol index, preferring the imperatively-populated
/// `WorkspaceSymbolIndexSingleton` (cheap: O(1) singleton input read with
/// HIGH durability) and falling back to the salsa-tracked
/// `workspace_symbol_index` (full rebuild over every file) when no singleton
/// has been committed.
///
/// In batch mode the singleton is always populated by
/// `MirDbStorage::rebuild_workspace_symbol_index`. The fallback exists for unit
/// tests that build a db directly without going through `AnalyzerDb`.
pub fn workspace_index(db: &dyn MirDatabase) -> &WorkspaceSymbolIndex {
    if let Some(s) = db.workspace_symbol_index_singleton() {
        s.index(db)
    } else {
        workspace_symbol_index(db)
    }
}

#[salsa::tracked]
pub fn workspace_symbol_index(db: &dyn MirDatabase) -> WorkspaceSymbolIndex {
    // workspace_revision() is always Some — init_workspace_revision() is called
    // at AnalyzerDb::new() so this query always reads the revision and salsa can
    // properly invalidate it when files are added or removed.
    let rev = db
        .workspace_revision()
        .expect("WorkspaceRevision not initialized");
    let _ = rev.revision(db);
    db.note_workspace_index_walk();

    let files = db.all_source_files();
    let mut class_like: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut functions: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut constants: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
    let mut class_like_by_short_name: FxHashMap<Name, Vec<Name>> = FxHashMap::default();

    // Same precedence as `tier_insert` (native stub < user file < user
    // stub; native ties keep the first, non-native ties the last),
    // expressed as three ordered passes so a host that never populates the
    // singleton (this walk re-runs per workspace-revision bump) pays plain
    // map inserts — no per-symbol declarer counts, no per-collision tier
    // lookups. Equivalence with the imperative `tier_insert` builder is
    // pinned by `tracked_walk_matches_imperative_rebuild` below, not kept
    // by hand.
    let user_stub_set: std::collections::HashSet<_> =
        db.user_stub_source_files().into_iter().collect();
    let (native_stubs, user_files): (Vec<SourceFile>, Vec<SourceFile>) = files
        .into_iter()
        .partition(|f| f.path(db).starts_with("stubs/"));

    // Pass 1: native stubs with or_insert (first-write-wins among stubs).
    // collect_file_declarations has a name-only PartialEq so body-only edits
    // don't propagate to this index.
    for file in &native_stubs {
        let decls = collect_file_declarations(db, *file);
        for decl in &decls.class_like {
            class_like.entry(decl.key).or_insert(decl.loc);
        }
        for decl in &decls.functions {
            functions.entry(decl.key).or_insert(decl.loc);
        }
        for decl in &decls.constants {
            constants.entry(decl.key).or_insert(decl.loc);
        }
    }

    // Pass 2: user-analyzed files overwrite native stubs.
    for file in &user_files {
        if user_stub_set.contains(file) {
            continue; // handled in pass 3
        }
        let decls = collect_file_declarations(db, *file);
        for decl in &decls.class_like {
            class_like.insert(decl.key, decl.loc);
        }
        for decl in &decls.functions {
            functions.insert(decl.key, decl.loc);
        }
        for decl in &decls.constants {
            constants.insert(decl.key, decl.loc);
        }
    }

    // Pass 3: user stubs overwrite everything.
    for file in &user_stub_set {
        let decls = collect_file_declarations(db, *file);
        for decl in &decls.class_like {
            class_like.insert(decl.key, decl.loc);
        }
        for decl in &decls.functions {
            functions.insert(decl.key, decl.loc);
        }
        for decl in &decls.constants {
            constants.insert(decl.key, decl.loc);
        }
    }

    for &key in class_like.keys() {
        class_like_by_short_name
            .entry(short_name_key(key))
            .or_default()
            .push(key);
    }

    build_workspace_symbol_index(class_like, functions, constants, class_like_by_short_name)
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
pub struct GlobalVarMap(pub Arc<FxHashMap<Arc<str>, mir_types::Type>>);

impl PartialEq for GlobalVarMap {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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
    let mut out: FxHashMap<Arc<str>, mir_types::Type> = FxHashMap::default();
    for file in files.iter() {
        let defs = collect_file_definitions(db, *file);
        for (name, ty) in &defs.slice.global_vars {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.entry(gname).or_insert_with(|| ty.clone());
        }
    }
    GlobalVarMap(Arc::new(out))
}

#[cfg(test)]
mod builder_equivalence_tests {
    use super::*;
    use std::sync::Arc;

    /// The tracked fallback and the imperative `tier_insert` rebuild encode
    /// the same precedence (native stub < user file < user stub) in two
    /// shapes — ordered passes vs per-insert tier checks. Pin equality of
    /// all four maps on a fixture that collides every tier pair, so the two
    /// encodings can't drift apart.
    #[test]
    fn tracked_walk_matches_imperative_rebuild() {
        let mut db = crate::db::MirDbStorage::default();
        let native =
            "<?php\nclass Dup {}\nclass StubOnly {}\nfunction dup_fn() {}\nconst DUP = 1;\n";
        let native2 = "<?php\nclass Dup {}\n"; // native/native tie: first wins
        let user1 =
            "<?php\nclass Dup {}\nclass UserOnly {}\nfunction dup_fn() {}\nconst DUP = 2;\n";
        let user2 = "<?php\nnamespace App;\nclass Dup {}\n"; // same short name, other namespace
        let ustub = "<?php\nclass Dup {}\nfunction dup_fn() {}\n";
        db.upsert_source_file_with_durability(
            Arc::from("stubs/std.php"),
            Arc::from(native),
            salsa::Durability::HIGH,
        );
        db.upsert_source_file_with_durability(
            Arc::from("stubs/extra.php"),
            Arc::from(native2),
            salsa::Durability::HIGH,
        );
        db.upsert_source_file_with_durability(
            Arc::from("user1.php"),
            Arc::from(user1),
            salsa::Durability::LOW,
        );
        db.upsert_source_file_with_durability(
            Arc::from("app.php"),
            Arc::from(user2),
            salsa::Durability::LOW,
        );
        db.upsert_source_file_with_durability(
            Arc::from("mystub.php"),
            Arc::from(ustub),
            salsa::Durability::LOW,
        );
        db.register_user_stub_path(Arc::from("mystub.php"));

        // Tracked walk first (no singleton yet), then the imperative rebuild.
        let tracked = workspace_symbol_index(&db).clone();
        db.rebuild_workspace_symbol_index();
        let rebuilt = workspace_index(&db).clone();

        fn maps_equal(a: &FxHashMap<Name, SymbolLoc>, b: &FxHashMap<Name, SymbolLoc>) -> bool {
            a.len() == b.len() && a.iter().all(|(k, v)| b.get(k) == Some(v))
        }
        assert!(
            maps_equal(&tracked.class_like, &rebuilt.class_like),
            "class_like maps differ"
        );
        assert!(
            maps_equal(&tracked.functions, &rebuilt.functions),
            "functions maps differ"
        );
        assert!(
            maps_equal(&tracked.constants, &rebuilt.constants),
            "constants maps differ"
        );
        // Bucket order is build-path-specific; compare as sets per key.
        assert_eq!(
            tracked.class_like_by_short_name.len(),
            rebuilt.class_like_by_short_name.len()
        );
        for (short, fqcns) in tracked.class_like_by_short_name.iter() {
            let a: std::collections::HashSet<_> = fqcns.iter().collect();
            let b: std::collections::HashSet<_> = rebuilt
                .class_like_by_short_name
                .get(short)
                .expect("bucket missing from imperative build")
                .iter()
                .collect();
            assert_eq!(a, b, "short-name bucket {short:?} differs");
        }

        // Both must have applied the tier rule, not just agreed with each
        // other: the user stub wins `Dup`/`dup_fn`, the user file wins `DUP`.
        let dup = tracked.class_like.get(&Name::new("dup")).copied().unwrap();
        assert_eq!(dup.file().path(&db).as_ref(), "mystub.php");
        let dup_fn = tracked
            .functions
            .get(&Name::new("dup_fn"))
            .copied()
            .unwrap();
        assert_eq!(dup_fn.file().path(&db).as_ref(), "mystub.php");
        let dup_const = tracked.constants.get(&Name::new("DUP")).copied().unwrap();
        assert_eq!(dup_const.file().path(&db).as_ref(), "user1.php");
    }
}

#[cfg(test)]
mod decl_projection_tests {
    use super::*;
    use std::sync::Arc;

    /// The warm-start seed projects `FileDeclarations` from raw disk slices;
    /// any drift from the tracked query's projection would poison
    /// `file_decl_snapshots` and the singleton. Pin full equality — names AND
    /// `SymbolLoc` variants/indices (the name-only `PartialEq` is too weak).
    #[test]
    fn decls_from_slice_matches_tracked_projection() {
        let mut db = crate::db::MirDbStorage::default();
        let text = "<?php\nnamespace App;\n\
            const TOP = 1;\n\
            interface I {}\n\
            trait T {}\n\
            enum E { case A; }\n\
            abstract class C implements I { use T; }\n\
            class D extends C {}\n\
            function f(): void {}\n\
            function g(int $x): int { return $x; }\n";
        let sf = db.upsert_source_file_with_durability(
            Arc::from("proj.php"),
            Arc::from(text),
            salsa::Durability::LOW,
        );

        let tracked = collect_file_declarations(&db, sf);
        let slice = collect_file_definitions(&db, sf).slice.clone();
        let projected = decls_from_slice(&slice, sf);

        let pairs = [
            (&tracked.class_like, &projected.class_like),
            (&tracked.functions, &projected.functions),
            (&tracked.constants, &projected.constants),
        ];
        for (t, p) in pairs {
            assert_eq!(t.len(), p.len());
            for (td, pd) in t.iter().zip(p.iter()) {
                assert_eq!(td.key, pd.key);
                assert_eq!(td.symbol, pd.symbol);
                assert!(td.loc == pd.loc, "SymbolLoc drift for {:?}", td.key);
            }
        }
        assert_eq!(tracked.class_like.len(), 5, "I, T, E, C, D expected");
    }

    #[test]
    fn file_decl_snapshot_keeps_only_keys() {
        let mut db = crate::db::MirDbStorage::default();
        let file_a = db.upsert_source_file_with_durability(
            Arc::from("a.php"),
            Arc::from("<?php\n"),
            salsa::Durability::LOW,
        );
        let file_b = db.upsert_source_file_with_durability(
            Arc::from("b.php"),
            Arc::from("<?php\n"),
            salsa::Durability::LOW,
        );
        let decls_a = FileDeclarations {
            class_like: vec![DeclaredSymbol {
                key: Name::new("app\\thing"),
                symbol: Arc::from("App\\Thing"),
                loc: SymbolLoc::Class {
                    file: file_a,
                    idx: 0,
                },
            }],
            functions: vec![DeclaredSymbol {
                key: Name::new("app\\run"),
                symbol: Arc::from("App\\run"),
                loc: SymbolLoc::Function {
                    file: file_a,
                    idx: 0,
                },
            }],
            constants: vec![DeclaredSymbol {
                key: Name::new("THING"),
                symbol: Arc::from("THING"),
                loc: SymbolLoc::Constant {
                    file: file_a,
                    idx: 0,
                },
            }],
        };
        let decls_b = FileDeclarations {
            class_like: vec![DeclaredSymbol {
                key: Name::new("app\\thing"),
                symbol: Arc::from("DisplayOnly"),
                loc: SymbolLoc::Trait {
                    file: file_b,
                    idx: 9,
                },
            }],
            functions: vec![DeclaredSymbol {
                key: Name::new("app\\run"),
                symbol: Arc::from("DisplayOnlyFn"),
                loc: SymbolLoc::Function {
                    file: file_b,
                    idx: 4,
                },
            }],
            constants: vec![DeclaredSymbol {
                key: Name::new("THING"),
                symbol: Arc::from("DISPLAY_ONLY_CONST"),
                loc: SymbolLoc::Constant {
                    file: file_b,
                    idx: 7,
                },
            }],
        };

        let snap_a = FileDeclSnapshot::from_decls(&decls_a);
        let snap_b = FileDeclSnapshot::from_decls(&decls_b);

        assert_eq!(snap_a, snap_b);
        assert!(snap_a.matches_decls(&decls_a));
        assert!(snap_a.matches_decls(&decls_b));
    }
}
