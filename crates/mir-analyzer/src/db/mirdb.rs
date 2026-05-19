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
/// find referencing files in O(1) even after the symbol's defining file entry
/// has been removed from `symbol_to_file`.
type SymbolReferencers = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;
/// Forward index: file path → set of symbol FQNs it defines.
/// Maintained in lockstep with `symbol_to_file` so `remove_file_definitions`
/// can find a file's symbols in O(symbols_in_file) instead of O(total_symbols).
type FileDefinedSymbols = Arc<Mutex<FxHashMap<Arc<str>, HashSet<Arc<str>>>>>;

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
    // Keep registries behind `Arc`s so `MirDb::clone()` stays cheap for
    // parallel analysis workers. The salsa storage is already shared by clone;
    // these maps only hold stable input handles, so copy-on-write insertion is
    // enough for the canonical mutable db paths.
    /// File path → first declared namespace.
    file_namespaces: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// File path → use-alias imports.
    file_imports: Arc<FxHashMap<Arc<str>, HashMap<String, String>>>,
    /// Global variable name (without `$`) → collected type.
    global_vars: Arc<FxHashMap<Arc<str>, Union>>,
    /// Symbol FQN → defining file.
    symbol_to_file: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// Forward index: file → set of symbol FQNs it defines.
    /// Maintained in lockstep with `symbol_to_file` so `remove_file_definitions`
    /// can find a file's symbols in O(symbols_in_file) instead of O(total_symbols).
    file_to_defined_symbols: FileDefinedSymbols,
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
    parse_cache: Arc<parking_lot::RwLock<FxHashMap<[u8; 32], Arc<StubSlice>>>>,
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
            file_namespaces: Arc::default(),
            file_imports: Arc::default(),
            global_vars: Arc::default(),
            symbol_to_file: Arc::default(),
            file_to_defined_symbols: FileDefinedSymbols::default(),
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
        self.file_namespaces.get(file).cloned()
    }

    fn file_imports(&self, file: &str) -> HashMap<String, String> {
        self.file_imports.get(file).cloned().unwrap_or_default()
    }

    fn global_var_type(&self, name: &str) -> Option<Union> {
        self.global_vars.get(name).cloned()
    }

    fn file_import_snapshots(&self) -> Vec<(Arc<str>, HashMap<String, String>)> {
        self.file_imports
            .iter()
            .map(|(file, imports)| (file.clone(), imports.clone()))
            .collect()
    }

    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>> {
        self.symbol_to_file.get(symbol).cloned()
    }

    fn symbols_defined_in_file(&self, file: &str) -> Vec<Arc<str>> {
        self.file_to_defined_symbols
            .lock()
            .get(file)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn file_defined_symbols(&self, file: &str) -> HashSet<Arc<str>> {
        self.file_to_defined_symbols
            .lock()
            .get(file)
            .cloned()
            .unwrap_or_default()
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

    fn parse_cache(&self) -> Arc<parking_lot::RwLock<FxHashMap<[u8; 32], Arc<StubSlice>>>> {
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
        self.parse_cache.write().insert(hash, slice);
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
    }

    /// Number of source files currently registered.
    pub fn source_file_count(&self) -> usize {
        self.source_files.len()
    }

    /// All registered source file paths.
    pub fn source_file_paths(&self) -> Vec<Arc<str>> {
        self.source_files.keys().cloned().collect()
    }

    /// Insert `symbol` into both `symbol_to_file` and the `file_to_defined_symbols`
    /// forward index. All definition-registration sites must use this helper.
    fn register_symbol(&mut self, symbol: Arc<str>, file: Arc<str>) {
        Arc::make_mut(&mut self.symbol_to_file).insert(symbol.clone(), file.clone());
        self.file_to_defined_symbols
            .lock()
            .entry(file)
            .or_default()
            .insert(symbol);
    }

    pub fn remove_file_definitions(&mut self, file: &str) {
        // O(1) forward-index lookup instead of O(total_symbols) scan.
        let symbol_set: HashSet<Arc<str>> = self
            .file_to_defined_symbols
            .lock()
            .remove(file)
            .unwrap_or_default();
        {
            let s2f = Arc::make_mut(&mut self.symbol_to_file);
            for sym in &symbol_set {
                s2f.remove(sym.as_ref());
            }
        }
        Arc::make_mut(&mut self.file_namespaces).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.file_imports).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.global_vars).retain(|name, _| !symbol_set.contains(name));
        self.clear_file_references(file);
    }

    /// Walk one collected [`StubSlice`] and upsert the corresponding db nodes.
    ///
    /// This is the canonical post-Pass-1 ingestion path: each file's slice is
    /// fed in directly, so batch analysis does not need any intermediate
    /// mutable codebase store between Pass 1 and Pass 2.
    pub fn ingest_stub_slice(&mut self, slice: &StubSlice) {
        if let Some(file) = &slice.file {
            if let Some(namespace) = &slice.namespace {
                Arc::make_mut(&mut self.file_namespaces).insert(file.clone(), namespace.clone());
            }
            if !slice.imports.is_empty() {
                Arc::make_mut(&mut self.file_imports).insert(file.clone(), slice.imports.clone());
            }
            for (name, _) in &slice.global_vars {
                let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
                self.register_symbol(Arc::from(global_name), file.clone());
            }
            for cls in &slice.classes {
                self.register_symbol(cls.fqcn.clone(), file.clone());
            }
            for iface in &slice.interfaces {
                self.register_symbol(iface.fqcn.clone(), file.clone());
            }
            for tr in &slice.traits {
                self.register_symbol(tr.fqcn.clone(), file.clone());
            }
            for en in &slice.enums {
                self.register_symbol(en.fqcn.clone(), file.clone());
            }
            for func in &slice.functions {
                self.register_symbol(func.fqn.clone(), file.clone());
            }
        }
        for (name, ty) in &slice.global_vars {
            let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
            Arc::make_mut(&mut self.global_vars).insert(Arc::from(global_name), ty.clone());
        }
    }

    /// Bulk-ingest many stub slices in one call.
    ///
    /// Why this exists: when an external `Arc<MirDb>` snapshot is alive (e.g.
    /// an LSP server holds one for query serving), each `Arc::make_mut` inside
    /// [`Self::ingest_stub_slice`] forces a copy-on-write clone of the
    /// underlying `HashMap`. Calling `ingest_stub_slice` N times in sequence
    /// with the snapshot alive between calls pays one clone *per call* —
    /// asymptotically O(N × map_size), which becomes pathological at vendor
    /// scale (~2k+ slices).
    ///
    /// Inside this bulk path the snapshot doesn't get refreshed between
    /// slices, so the first slice's clone establishes a fresh inner `Arc` with
    /// `strong_count == 1` and every subsequent insert in the batch is O(1).
    /// Net cost: O(N + map_size) instead of O(N × map_size).
    ///
    /// Use this whenever you're about to ingest more than one slice in a row,
    /// such as in:
    /// - LSP warm-up over a `composer.lock` worth of vendor files
    /// - Project-wide reindex
    /// - Cache hydration on session restart
    pub fn ingest_stub_slices<'a, I>(&mut self, slices: I)
    where
        I: IntoIterator<Item = &'a StubSlice>,
    {
        for slice in slices {
            self.ingest_stub_slice(slice);
        }
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
