use std::collections::{HashMap, HashSet};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_codebase::StubSlice;
use mir_types::Union;

use super::*;

// MirDb concrete database

/// Concrete in-process Salsa database.
///
/// `Clone` is required for parallel batch analysis: salsa's supported
/// pattern for sharing a db across threads is to give each worker its
/// own clone (each clone gets a fresh `ZalsaLocal`, sharing the
/// underlying memoization storage).  Sharing `&MirDb` across threads is
/// **not** supported because `salsa::Database: Send` (not `Sync`).
type ReferenceLocations = Arc<Mutex<FxHashMap<Arc<str>, Vec<(Arc<str>, u32, u16, u16)>>>>;
/// Forward index: file path → set of symbol keys that file references.
/// Kept in sync with `reference_locations` for O(degree) lookups.
type FileReferences = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;
/// Reverse reference index: symbol key → set of files that reference it.
/// Transpose of `FileReferences`; maintained in lockstep so deletions can
/// find referencing files in O(1).
type SymbolReferencers = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;

/// Per-clone staging buffer for reference locations recorded during a parallel
/// Pass 2 worker.  `record_reference_location` pushes here instead of directly
/// into the shared `Arc<Mutex<...>>` maps, eliminating cross-thread contention.
/// After the parallel phase the owner calls `take_pending_ref_locs` and commits
/// the batch serially via `commit_reference_locations_batch`.
///
/// The custom `Clone` impl returns a *new empty buffer* so that each `MirDb`
/// worker clone starts fresh — we do NOT propagate one clone's pending entries
/// to another worker.
#[derive(Default)]
struct PendingRefLocs(Mutex<Vec<super::reference_locations::RefLoc>>);

impl Clone for PendingRefLocs {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[salsa::db]
#[derive(Clone)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
    /// Public symbol key → reference locations.
    reference_locations: ReferenceLocations,
    /// Forward index: file → set of symbol keys it references. Kept in sync
    /// with `reference_locations` to provide O(degree) dep-graph lookups.
    file_references: FileReferences,
    /// Reverse reference index: symbol key → set of files that reference it.
    /// Maintained in lockstep with `file_references`.
    symbol_referencers: SymbolReferencers,
    /// Per-clone staging area for reference locations.  Workers push here
    /// during parallel analysis; the orchestrator drains and commits serially.
    pending_ref_locs: PendingRefLocs,
    /// File path → Salsa SourceFile input handle.
    source_files: Arc<FxHashMap<Arc<str>, SourceFile>>,
    /// Side-channel resolver state. The `ResolverConfig` salsa input is
    /// lazily created on first `set_resolver` call; its `revision` is
    /// bumped on every subsequent change so dependent tracked queries
    /// (e.g. `resolve_fqcn_to_path`) are invalidated. The `Arc<dyn
    /// ClassResolver>` lives off-salsa because trait objects don't
    /// participate in `salsa::Update`.
    resolver_state: Arc<parking_lot::RwLock<ResolverState>>,
    /// Lazily-created singleton [`InferredReturnTypes`] input. Stored
    /// alongside `resolver_state` (rather than as a bare salsa input
    /// handle on `Self`) because the handle is `Copy` and we need
    /// `MirDb: Clone`-friendly storage.
    inferred_return_types_input: Arc<parking_lot::RwLock<Option<InferredReturnTypes>>>,
    /// Lazily-created [`WorkspaceRevision`] singleton input; bumped on
    /// file add/remove so workspace-enumeration tracked queries
    /// (`workspace_classes`, `workspace_functions`) invalidate.
    workspace_revision_input: Arc<parking_lot::RwLock<Option<WorkspaceRevision>>>,
    /// Paths of user-provided stub files (registered via `ingest_user_stubs`).
    /// Used by `workspace_symbol_index` to give user stubs priority over
    /// native stubs when two files define the same symbol.
    user_stub_paths: Arc<parking_lot::RwLock<rustc_hash::FxHashSet<Arc<str>>>>,
    /// Target PHP version for this analysis run. Defaults to "8.2".
    /// Set once before any analysis begins; read by `collect_file_definitions`
    /// to filter `@since`/`@removed` stub symbols.
    php_version: Arc<parking_lot::RwLock<Arc<str>>>,
    /// Optional disk-backed Pass-1 cache. Shared with `SharedDb::stub_cache`
    /// so `collect_file_definitions` can consult it without going through the
    /// push-based `collect_and_ingest_file` path.
    stub_cache: Arc<parking_lot::RwLock<Option<Arc<crate::stub_cache::StubSliceCache>>>>,
    /// In-process parse-result cache: content-hash → StubSlice. Populated by
    /// `collect_and_ingest_file` so that `collect_file_definitions_uncached`
    /// can skip re-parsing files that were already parsed in the same session.
    /// Keyed by blake3 hash of the source text so stale entries from prior
    /// file versions are naturally evicted (different hash → different key).
    /// DashMap (64 shards) replaces a single RwLock so parallel workers contend
    /// on independent shards instead of serialising at a single write lock.
    parse_cache: Arc<dashmap::DashMap<[u8; 32], Arc<StubSlice>>>,
    /// Pre-built FQCN symbol index singleton. Written imperatively by
    /// `rebuild_workspace_symbol_index` and read by `find_class_like` /
    /// `find_function` / `find_global_constant` via `singleton.index(db)`
    /// (one HIGH-durability tracked dep) instead of the O(N_files) tracked
    /// dep list that `workspace_symbol_index` accumulates.
    workspace_symbol_index_input:
        Arc<parking_lot::RwLock<Option<crate::db::WorkspaceSymbolIndexSingleton>>>,
    /// Per-file declaration snapshots used to detect name changes during
    /// incremental edits. When `ingest_file` is called with a body-only
    /// edit the snapshot comparison returns `false` and the rebuild is
    /// skipped, keeping the singleton's revision unchanged.
    file_decl_snapshots:
        Arc<parking_lot::RwLock<FxHashMap<SourceFile, crate::db::FileDeclarations>>>,
}

/// Resolver-related state held outside salsa storage. Wrapped in a
/// `parking_lot::RwLock` so `MirDb::clone()` (cheap for parallel readers)
/// shares one slot rather than copying.
#[derive(Default)]
struct ResolverState {
    /// Lazily created on first `set_resolver`. Once created, the handle
    /// is stable across clones and subsequent resolver swaps.
    config: Option<ResolverConfig>,
    /// Currently active resolver. `None` for sessions configured without
    /// PSR-4 / classmap support.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
}

impl Default for MirDb {
    fn default() -> Self {
        let mut db = Self {
            storage: salsa::Storage::default(),
            reference_locations: ReferenceLocations::default(),
            file_references: FileReferences::default(),
            symbol_referencers: SymbolReferencers::default(),
            pending_ref_locs: PendingRefLocs::default(),
            source_files: Arc::default(),
            resolver_state: Arc::default(),
            inferred_return_types_input: Arc::default(),
            workspace_revision_input: Arc::default(),
            user_stub_paths: Arc::default(),
            php_version: Arc::new(parking_lot::RwLock::new(Arc::from("8.2"))),
            stub_cache: Arc::default(),
            parse_cache: Arc::default(),
            workspace_symbol_index_input: Arc::default(),
            file_decl_snapshots: Arc::default(),
        };
        db.init_workspace_revision();
        db
    }
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        self.php_version.read().clone()
    }

    fn file_namespace(&self, file: &str) -> Option<Arc<str>> {
        let sf = self.source_files.get(file).copied()?;
        crate::db::collect_file_definitions(self, sf)
            .slice
            .namespace
            .clone()
    }

    fn file_imports(&self, file: &str) -> HashMap<String, String> {
        let Some(sf) = self.source_files.get(file).copied() else {
            return HashMap::default();
        };
        crate::db::collect_file_definitions(self, sf)
            .slice
            .imports
            .clone()
    }

    fn global_var_type(&self, name: &str) -> Option<Union> {
        crate::db::workspace_global_vars(self).0.get(name).cloned()
    }

    fn file_import_snapshots(&self) -> Vec<(Arc<str>, HashMap<String, String>)> {
        self.source_files
            .iter()
            .map(|(path, &sf)| {
                let imports = crate::db::collect_file_definitions(self, sf)
                    .slice
                    .imports
                    .clone();
                (path.clone(), imports)
            })
            .collect()
    }

    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>> {
        let idx = crate::db::workspace_symbol_index(self);
        let lower = symbol.to_ascii_lowercase();
        // Class-like and function keys are case-folded (PHP semantics).
        // Constants are case-sensitive, so tried last without lowercasing.
        // Global variables are not indexed here — they are not FQCNs and
        // are not looked up via this method in any current caller.
        let loc = idx
            .class_like
            .get(&lower)
            .or_else(|| idx.functions.get(&lower))
            .or_else(|| idx.constants.get(symbol));
        loc.map(|l| {
            let sf = match l {
                SymbolLoc::Class { file, .. }
                | SymbolLoc::Interface { file, .. }
                | SymbolLoc::Trait { file, .. }
                | SymbolLoc::Enum { file, .. }
                | SymbolLoc::Function { file, .. }
                | SymbolLoc::Constant { file, .. } => *file,
            };
            sf.path(self)
        })
    }

    fn symbols_defined_in_file(&self, file: &str) -> Vec<Arc<str>> {
        self.file_defined_symbols(file).into_iter().collect()
    }

    fn file_defined_symbols(&self, file: &str) -> HashSet<Arc<str>> {
        let Some(sf) = self.source_files.get(file).copied() else {
            return HashSet::default();
        };
        let defs = crate::db::collect_file_definitions(self, sf);
        let mut out = HashSet::new();
        for c in defs.slice.classes.iter() {
            out.insert(c.fqcn.clone());
        }
        for i in defs.slice.interfaces.iter() {
            out.insert(i.fqcn.clone());
        }
        for t in defs.slice.traits.iter() {
            out.insert(t.fqcn.clone());
        }
        for e in defs.slice.enums.iter() {
            out.insert(e.fqcn.clone());
        }
        for f in defs.slice.functions.iter() {
            out.insert(f.fqn.clone());
        }
        for (name, _) in defs.slice.constants.iter() {
            out.insert(name.clone());
        }
        for (name, _) in defs.slice.global_vars.iter() {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.insert(gname);
        }
        out
    }

    fn symbol_referencers_of(&self, symbol_key: &str) -> Vec<Arc<str>> {
        self.symbol_referencers
            .lock()
            .get(symbol_key)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn record_reference_location(&self, loc: RefLoc) {
        self.pending_ref_locs.0.lock().push(loc);
    }

    fn take_pending_ref_locs(&self) -> Vec<RefLoc> {
        std::mem::take(&mut *self.pending_ref_locs.0.lock())
    }

    fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u16, u16)]) {
        for (symbol, line, col_start, col_end) in locs {
            self.record_reference_location(RefLoc {
                symbol_key: Arc::from(symbol.as_str()),
                file: file.clone(),
                line: *line,
                col_start: *col_start,
                col_end: *col_end,
            });
        }
    }

    fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self.reference_locations.lock();
        let mut out = Vec::new();
        for (symbol, locs) in refs.iter() {
            for (loc_file, line, col_start, col_end) in locs {
                if loc_file.as_ref() == file {
                    out.push((symbol.clone(), *line, *col_start, *col_end));
                }
            }
        }
        out
    }

    fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self.reference_locations.lock();
        refs.get(symbol).cloned().unwrap_or_default()
    }

    fn has_reference(&self, symbol: &str) -> bool {
        let refs = self.reference_locations.lock();
        refs.get(symbol).is_some_and(|locs| !locs.is_empty())
    }

    fn clear_file_references(&self, file: &str) {
        // Drain the forward index first to learn which symbols this file referenced,
        // then remove only those entries — O(degree) instead of O(S×R).
        let symbol_keys = {
            let mut file_refs = self.file_references.lock();
            file_refs.remove(file).unwrap_or_default()
        };
        let mut refs = self.reference_locations.lock();
        let mut sym_refs = self.symbol_referencers.lock();
        for key in &symbol_keys {
            if let Some(locs) = refs.get_mut(key) {
                locs.retain(|(loc_file, _, _, _)| loc_file.as_ref() != file);
            }
            let empty = if let Some(referencers) = sym_refs.get_mut(key) {
                referencers.remove(file);
                referencers.is_empty()
            } else {
                false
            };
            if empty {
                sym_refs.remove(key);
            }
        }
    }

    fn all_reference_location_pairs(&self) -> Vec<(Arc<str>, Arc<str>)> {
        let refs = self.reference_locations.lock();
        let mut pairs = Vec::new();
        for (symbol, locs) in refs.iter() {
            for (file, _, _, _) in locs {
                pairs.push((file.clone(), symbol.clone()));
            }
        }
        pairs
    }

    fn file_referenced_symbols(&self, file: &str) -> Vec<Arc<str>> {
        let file_refs = self.file_references.lock();
        file_refs
            .get(file)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn lookup_source_file(&self, path: &str) -> Option<SourceFile> {
        self.source_files.get(path).copied()
    }

    fn resolver_config(&self) -> Option<ResolverConfig> {
        self.resolver_state.read().config
    }

    fn current_resolver(&self) -> Option<Arc<dyn crate::ClassResolver>> {
        self.resolver_state.read().resolver.clone()
    }

    fn inferred_return_types(&self) -> Option<InferredReturnTypes> {
        *self.inferred_return_types_input.read()
    }

    fn workspace_revision(&self) -> Option<WorkspaceRevision> {
        *self.workspace_revision_input.read()
    }

    fn workspace_symbol_index_singleton(&self) -> Option<crate::db::WorkspaceSymbolIndexSingleton> {
        *self.workspace_symbol_index_input.read()
    }

    fn all_source_files(&self) -> Vec<SourceFile> {
        self.source_files.values().copied().collect()
    }

    fn user_stub_source_files(&self) -> Vec<SourceFile> {
        let user_paths = self.user_stub_paths.read();
        self.source_files
            .iter()
            .filter(|(path, _)| user_paths.contains(path.as_ref()))
            .map(|(_, &sf)| sf)
            .collect()
    }

    fn stub_cache(&self) -> Option<Arc<crate::stub_cache::StubSliceCache>> {
        self.stub_cache.read().clone()
    }

    fn parse_cache(&self) -> Arc<dashmap::DashMap<[u8; 32], Arc<StubSlice>>> {
        self.parse_cache.clone()
    }
}

impl MirDb {
    /// Wire a disk-backed stub cache into this db so `collect_file_definitions`
    /// can skip reparsing on cache hits. Called by `SharedDb::with_cache_dir`.
    pub fn set_stub_cache(&self, cache: Arc<crate::stub_cache::StubSliceCache>) {
        *self.stub_cache.write() = Some(cache);
    }

    /// Store a pre-computed `StubSlice` for a given content hash so that
    /// `collect_file_definitions_uncached` can skip re-parsing files already
    /// processed by `collect_and_ingest_file` in the same session.
    pub fn prime_parse_cache(&self, hash: [u8; 32], slice: Arc<StubSlice>) {
        self.parse_cache.insert(hash, slice);
    }

    /// Rebuild the `WorkspaceSymbolIndexSingleton` salsa input from scratch.
    ///
    /// Iterates every registered `SourceFile`, calls `collect_file_declarations`
    /// on each (salsa-memoized — cheap after Parse-1 primes the caches), builds
    /// a fresh `WorkspaceSymbolIndex`, and sets it on the singleton input with
    /// `Durability::HIGH`.  Tracked queries that read `singleton.index(db)` get
    /// a single HIGH-durability dep; on LOW-durability project-file body edits
    /// salsa short-circuits the dep in O(1) instead of walking O(N_files).
    ///
    /// Also updates `file_decl_snapshots` so `file_declarations_changed` can
    /// quickly detect whether a subsequent edit changed any declared names.
    ///
    /// **Must be called outside any tracked-query context** (it sets a salsa
    /// input field).  Typical call sites: end of `collect_types_only`, end of
    /// `AnalysisSession::rebuild_workspace_symbol_index`, and after any
    /// `ingest_file` that detects a declaration change.
    pub fn rebuild_workspace_symbol_index(&mut self) {
        use crate::db::{
            collect_file_declarations, FileDeclarations, SymbolLoc, WorkspaceSymbolIndex,
            WorkspaceSymbolIndexSingleton,
        };
        use salsa::Setter as _;

        let files = self.all_source_files();
        let user_stub_files = self.user_stub_source_files();

        let mut class_like: FxHashMap<String, SymbolLoc> = FxHashMap::default();
        let mut functions: FxHashMap<String, SymbolLoc> = FxHashMap::default();
        let mut constants: FxHashMap<String, SymbolLoc> = FxHashMap::default();
        let mut new_snapshots: FxHashMap<SourceFile, FileDeclarations> = FxHashMap::default();

        // Immutable borrow scope: collect declarations from all files.
        {
            let db: &dyn MirDatabase = &*self;
            for &file in files.iter() {
                let decls = collect_file_declarations(db, file);
                for (key, loc) in &decls.class_like {
                    class_like.entry(key.clone()).or_insert(*loc);
                }
                for (key, loc) in &decls.functions {
                    functions.entry(key.clone()).or_insert(*loc);
                }
                for (key, loc) in &decls.constants {
                    constants.entry(key.clone()).or_insert(*loc);
                }
                new_snapshots.insert(file, decls);
            }
            // User stubs override native stubs for the same symbol.
            for &file in user_stub_files.iter() {
                let decls = collect_file_declarations(db, file);
                for (key, loc) in &decls.class_like {
                    class_like.insert(key.clone(), *loc);
                }
                for (key, loc) in &decls.functions {
                    functions.insert(key.clone(), *loc);
                }
                for (key, loc) in &decls.constants {
                    constants.insert(key.clone(), *loc);
                }
            }
        }

        *self.file_decl_snapshots.write() = new_snapshots;

        let new_index = WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };

        let existing = *self.workspace_symbol_index_input.read();
        match existing {
            Some(s) => {
                let old = s.index(self);
                if old != new_index {
                    s.set_index(self)
                        .with_durability(salsa::Durability::HIGH)
                        .to(new_index);
                }
            }
            None => {
                let s = WorkspaceSymbolIndexSingleton::builder(new_index)
                    .durability(salsa::Durability::HIGH)
                    .new(self);
                *self.workspace_symbol_index_input.write() = Some(s);
            }
        }
    }

    /// Check whether the declared names in `file` differ from the last
    /// snapshot captured by `rebuild_workspace_symbol_index`.
    ///
    /// Returns `true` if the declarations changed (or the file is new) so
    /// the caller should call `rebuild_workspace_symbol_index`. Returns
    /// `false` if the names are identical (body-only edit) so the rebuild
    /// — and its HIGH-durability singleton set — can be skipped.
    ///
    /// Updates the snapshot for `file` when a change is detected.
    pub fn file_declarations_changed(&mut self, file: SourceFile) -> bool {
        let new_decls = {
            let db: &dyn MirDatabase = &*self;
            crate::db::collect_file_declarations(db, file)
        };
        let mut snapshots = self.file_decl_snapshots.write();
        match snapshots.get(&file) {
            Some(old) if *old == new_decls => false,
            _ => {
                snapshots.insert(file, new_decls);
                true
            }
        }
    }

    /// Commit a batch of reference locations into the shared maps in one lock
    /// acquisition per map.  Must be called serially after all parallel workers
    /// have dropped their db clones and returned their pending buffers.
    pub fn commit_reference_locations_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        let mut refs = self.reference_locations.lock();
        let mut file_refs = self.file_references.lock();
        let mut sym_refs = self.symbol_referencers.lock();
        for loc in locs {
            file_refs
                .entry(loc.file.clone())
                .or_default()
                .insert(loc.symbol_key.clone());
            sym_refs
                .entry(loc.symbol_key.clone())
                .or_default()
                .insert(loc.file.clone());
            let entry = refs.entry(loc.symbol_key).or_default();
            let tuple = (loc.file, loc.line, loc.col_start, loc.col_end);
            if !entry.iter().any(|e| e == &tuple) {
                entry.push(tuple);
            }
        }
    }

    /// Drain this db's pending buffer and commit it directly to the shared maps.
    ///
    /// Use on serial paths (e.g. `re_analyze_file`) where the db is not a
    /// worker clone: pending locations accumulate in the shared db itself and
    /// must be flushed before callers read the reference maps.
    pub fn commit_pending_to_maps(&self) {
        let locs = std::mem::take(&mut *self.pending_ref_locs.0.lock());
        self.commit_reference_locations_batch(locs);
    }

    /// Install or replace the active class resolver.
    ///
    /// First call lazily creates the singleton [`ResolverConfig`] salsa
    /// input (revision = 0); subsequent calls bump the revision so
    /// downstream tracked queries (notably
    /// [`crate::db::resolve_fqcn_to_path`]) are invalidated.
    ///
    /// Mark a file path as a user-provided stub so `workspace_symbol_index`
    /// gives it priority over native stubs for the same symbol.
    pub fn register_user_stub_path(&self, path: Arc<str>) {
        self.user_stub_paths.write().insert(path);
    }

    /// Returns `true` if `file` was registered as a user stub.
    pub fn is_user_stub_file(&self, file: SourceFile, db: &dyn crate::db::MirDatabase) -> bool {
        self.user_stub_paths.read().contains(file.path(db).as_ref())
    }

    /// Update the target PHP version for `@since`/`@removed` filtering in
    /// `collect_file_definitions`. Must be called before any stub files are
    /// registered so the salsa cache sees consistent results.
    pub fn set_php_version(&mut self, version: Arc<str>) {
        *self.php_version.write() = version;
    }

    /// `None` clears the resolver. The `ResolverConfig` input is *not*
    /// removed — it remains as a versioned anchor, with revision bumped to
    /// signal the change.
    pub fn set_resolver(&mut self, resolver: Option<Arc<dyn crate::ClassResolver>>) {
        use salsa::Setter as _;
        // The lock and salsa storage are independent; we briefly read /
        // briefly write the lock, but never hold it across a salsa setter
        // call (which needs `&mut self`).
        let existing = self.resolver_state.read().config;
        match existing {
            Some(c) => {
                let current = c.revision(self);
                c.set_revision(self).to(current.wrapping_add(1));
            }
            None => {
                let c = ResolverConfig::new(self, 0);
                self.resolver_state.write().config = Some(c);
            }
        }
        self.resolver_state.write().resolver = resolver;
    }

    /// Create a new or update an existing Salsa SourceFile input for `path`.
    /// Returns the stable handle that callers should retain for tracked queries.
    pub fn upsert_source_file(&mut self, path: Arc<str>, text: Arc<str>) -> SourceFile {
        self.upsert_source_file_with_durability(path, text, salsa::Durability::LOW)
    }

    /// Like [`upsert_source_file`] but lets callers set the salsa durability.
    ///
    /// Use `Durability::HIGH` for files that will not change within the session
    /// (built-in PHP stubs, vendor packages). This lets salsa skip O(N)
    /// dependency verification for `workspace_symbol_index` when only a
    /// `Durability::LOW` project file changes.
    pub fn upsert_source_file_with_durability(
        &mut self,
        path: Arc<str>,
        text: Arc<str>,
        durability: salsa::Durability,
    ) -> SourceFile {
        use salsa::Setter as _;
        if let Some(&sf) = self.source_files.get(&path) {
            if sf.text(self) != text {
                sf.set_text(self).with_durability(durability).to(text);
            }
            return sf;
        }
        let sf = SourceFile::builder(path.clone(), text)
            .durability(durability)
            .new(self);
        Arc::make_mut(&mut self.source_files).insert(path, sf);
        self.bump_workspace_revision();
        sf
    }

    /// Remove the Salsa SourceFile handle for `path` from the registry.
    pub fn remove_source_file(&mut self, path: &str) {
        if Arc::make_mut(&mut self.source_files).remove(path).is_some() {
            self.bump_workspace_revision();
        }
    }

    /// Create the WorkspaceRevision salsa input at revision 0 if it doesn't
    /// exist yet. Called once at database construction so workspace_symbol_index
    /// always reads the revision and salsa can invalidate it on first file add.
    pub fn init_workspace_revision(&mut self) {
        if self.workspace_revision_input.read().is_none() {
            let rev = crate::db::WorkspaceRevision::new(self, 0);
            *self.workspace_revision_input.write() = Some(rev);
        }
    }

    /// Bump the workspace revision so tracked `workspace_*` queries
    /// reading it invalidate. Lazily creates the singleton input on
    /// first call.
    fn bump_workspace_revision(&mut self) {
        use salsa::Setter as _;
        let existing = *self.workspace_revision_input.read();
        match existing {
            Some(rev) => {
                let cur = rev.revision(self);
                rev.set_revision(self).to(cur.wrapping_add(1));
            }
            None => {
                let rev = crate::db::WorkspaceRevision::new(self, 0);
                *self.workspace_revision_input.write() = Some(rev);
            }
        }
        // A new file was added to the workspace. The pre-built symbol index
        // singleton no longer covers all registered files; clear it so
        // `find_class_like` / `find_function` fall back to the tracked
        // `workspace_symbol_index` query until an explicit rebuild is done.
        *self.workspace_symbol_index_input.write() = None;
    }

    /// Number of source files currently registered.
    pub fn source_file_count(&self) -> usize {
        self.source_files.len()
    }

    /// All registered source file paths.
    pub fn source_file_paths(&self) -> Vec<Arc<str>> {
        self.source_files.keys().cloned().collect()
    }

    /// Clear push-based state for `file` and reset its reference-location
    /// index.  After the salsa migration all symbol lookups are derived from
    /// `collect_file_definitions`; only the reference-location side-index
    /// still needs explicit clearing before re-analysis.
    pub fn remove_file_definitions(&mut self, file: &str) {
        self.clear_file_references(file);
    }

    /// No-op — retained only so call sites that were written against the old
    /// push-based ingest path continue to compile without changes.  All symbol
    /// data is now derived lazily from `collect_file_definitions` tracked
    /// queries; calling this function has no effect.
    #[inline]
    pub fn ingest_stub_slice(&mut self, _slice: &StubSlice) {}

    /// No-op — retained for the same reason as `ingest_stub_slice`.
    #[inline]
    pub fn ingest_stub_slices<'a, I>(&mut self, _slices: I)
    where
        I: IntoIterator<Item = &'a StubSlice>,
    {
    }

    /// Commit a parallel-sweep-collected [`InferredReturnTypes`] buffer
    /// into the Salsa db.  **Must be called serially**, after all rayon
    /// workers from the priming sweep have dropped their db clones, so
    /// that `Storage::cancel_others` sees strong-count==1 inside the
    /// setter.  Calling this from inside a `for_each_with` / `map_with`
    /// closure will deadlock.
    ///
    /// Skips writes whose value already matches the current Salsa-tracked
    /// value (preserves PR21's fast-skip semantics).  Skips inactive
    /// nodes — there's no point committing an inferred return for a node
    /// that has been deactivated by a re-analyze.
    /// Commit inferred return types collected during the priming sweep.
    /// Takes ownership of the function and method inferred type vectors.
    pub fn commit_inferred_return_types(
        &mut self,
        functions: Vec<(Arc<str>, mir_types::Union)>,
        methods: Vec<(Arc<str>, Arc<str>, mir_types::Union)>,
    ) {
        let merged_functions = self.inferred_function_map_clone();
        let merged_methods = self.inferred_method_map_clone();
        let mut new_functions = (*merged_functions).clone();
        let mut new_methods = (*merged_methods).clone();

        for (fqn, inferred) in functions {
            let arc_inferred = Arc::new(inferred);
            new_functions.insert(fqn, arc_inferred);
        }
        for (fqcn, name, inferred) in methods {
            let name_lower: Arc<str> = if name.chars().all(|c| !c.is_uppercase()) {
                name.clone()
            } else {
                Arc::from(name.to_lowercase().as_str())
            };
            let arc_inferred = Arc::new(inferred);
            new_methods.insert((fqcn, name_lower), arc_inferred);
        }

        self.set_inferred_return_types_input(Arc::new(new_functions), Arc::new(new_methods));
    }

    /// Snapshot the function inferred map from the singleton input, or an
    /// empty map if the input hasn't been created yet.
    fn inferred_function_map_clone(&self) -> Arc<crate::db::FunctionInferredMap> {
        match *self.inferred_return_types_input.read() {
            Some(input) => input.functions(self),
            None => Arc::new(rustc_hash::FxHashMap::default()),
        }
    }

    /// Snapshot the method inferred map from the singleton input, or an
    /// empty map if the input hasn't been created yet.
    fn inferred_method_map_clone(&self) -> Arc<crate::db::MethodInferredMap> {
        match *self.inferred_return_types_input.read() {
            Some(input) => input.methods(self),
            None => Arc::new(rustc_hash::FxHashMap::default()),
        }
    }

    /// Set / create the singleton [`crate::db::InferredReturnTypes`] input.
    /// Lazily creates the input on first call (handle stored on
    /// `inferred_return_types_input`); subsequent calls update the
    /// existing handle via setters.
    fn set_inferred_return_types_input(
        &mut self,
        functions: Arc<crate::db::FunctionInferredMap>,
        methods: Arc<crate::db::MethodInferredMap>,
    ) {
        use salsa::Setter as _;
        let existing = *self.inferred_return_types_input.read();
        match existing {
            Some(handle) => {
                handle.set_functions(self).to(functions);
                handle.set_methods(self).to(methods);
            }
            None => {
                let handle = crate::db::InferredReturnTypes::new(self, functions, methods);
                *self.inferred_return_types_input.write() = Some(handle);
            }
        }
    }
}
