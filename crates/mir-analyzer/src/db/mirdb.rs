use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_codebase::StubSlice;
use mir_types::{Name, Type};

use super::*;

// MirDbStorage concrete database

/// Concrete in-process Salsa database.
///
/// `Clone` is required for parallel batch analysis: salsa's supported
/// pattern for sharing a db across threads is to give each worker its
/// own clone (each clone gets a fresh `ZalsaLocal`, sharing the
/// underlying memoization storage).  Sharing `&MirDbStorage` across threads is
/// **not** supported because `salsa::Database: Send` (not `Sync`).
/// Per-clone staging buffer for reference locations recorded during a parallel
/// body analysis worker.  `record_reference_location` pushes here instead of directly
/// into the shared reference index, eliminating cross-thread contention.
/// After the parallel phase the owner calls `take_pending_ref_locs` and commits
/// the batch serially.
///
/// Internally a *stack of frames* (the base frame is always present): pure
/// per-scope analysis entry points push a fresh frame on entry and pop it on
/// exit, so refs recorded by a nested tracked query running on the same db
/// handle land in the nested frame instead of leaking into the caller's
/// staged refs. The flat-buffer behavior (record → base frame, take → drain
/// base frame) is the degenerate one-frame case.
///
/// The custom `Clone` impl returns a *new empty buffer* so that each `MirDbStorage`
/// worker clone starts fresh — we do NOT propagate one clone's pending entries
/// to another worker.
struct PendingRefLocs(Mutex<Vec<Vec<super::reference_locations::RefLoc>>>);

impl Default for PendingRefLocs {
    fn default() -> Self {
        Self(Mutex::new(vec![Vec::new()]))
    }
}

impl Clone for PendingRefLocs {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Classify a file's precedence tier for the workspace symbol index.
/// User stubs win over everything; native stub files (`stubs/…`) lose to
/// analyzed user/vendor files.
fn symbol_tier(
    file: SourceFile,
    db: &dyn MirDatabase,
    user_stubs: &rustc_hash::FxHashSet<Arc<str>>,
) -> crate::db::SymbolTier {
    use crate::db::SymbolTier;
    let path = file.path(db);
    if user_stubs.contains(path.as_ref()) {
        SymbolTier::UserStub
    } else if path.starts_with("stubs/") {
        SymbolTier::NativeStub
    } else {
        SymbolTier::UserFile
    }
}

/// Tier-aware insert into one index map + declarer-count bump. Overwrites the
/// existing entry only if the new tier outranks it (or ties at a non-native
/// tier, where last-write-wins matches the full-rebuild `insert` semantics;
/// native-stub ties keep the first, matching `or_insert`).
fn tier_insert(
    map: &mut FxHashMap<Name, crate::db::SymbolLoc>,
    counts: &mut FxHashMap<Name, u32>,
    key: Name,
    loc: crate::db::SymbolLoc,
    new_tier: crate::db::SymbolTier,
    db: &dyn MirDatabase,
    user_stubs: &rustc_hash::FxHashSet<Arc<str>>,
) {
    use crate::db::SymbolTier;
    *counts.entry(key).or_insert(0) += 1;
    match map.get(&key) {
        None => {
            map.insert(key, loc);
        }
        Some(existing) => {
            let existing_tier = symbol_tier(existing.file(), db, user_stubs);
            let overwrite = new_tier > existing_tier
                || (new_tier == existing_tier && new_tier != SymbolTier::NativeStub);
            if overwrite {
                map.insert(key, loc);
            }
        }
    }
}

/// Subtract one file's declarations from an index map + counts. Assumes the
/// caller has already ruled out the ambiguous case (count > 1 while this file
/// owns the winning entry). Removes the map entry only when no other file
/// declares the name (count → 0) and the entry currently points at `file`.
fn subtract_decls(
    map: &mut FxHashMap<Name, crate::db::SymbolLoc>,
    counts: &mut FxHashMap<Name, u32>,
    entries: &[(Name, crate::db::SymbolLoc)],
    file: SourceFile,
) {
    for (key, _) in entries {
        let remaining = match counts.get_mut(key) {
            Some(c) => {
                *c = c.saturating_sub(1);
                *c
            }
            None => 0,
        };
        if remaining == 0 {
            counts.remove(key);
            if map.get(key).map(|l| l.file()) == Some(file) {
                map.remove(key);
            }
        }
    }
}

#[salsa::db]
#[derive(Clone)]
pub struct MirDbStorage {
    storage: salsa::Storage<Self>,
    /// Unified reference index: symbol→locations, file→symbols and
    /// symbol→files views behind one lock with a single writer path, so the
    /// views cannot drift apart. See [`crate::db::ref_index::RefIndex`].
    ref_index: Arc<Mutex<crate::db::ref_index::RefIndex>>,
    /// Per-clone staging area for reference locations.  Workers push here
    /// during parallel analysis; the orchestrator drains and commits serially.
    pending_ref_locs: PendingRefLocs,
    /// File path → Salsa SourceFile input handle.
    source_files: Arc<FxHashMap<Arc<str>, SourceFile>>,
    /// Paths removed via `remove_source_file`. The `SourceFile` handle remains
    /// in `source_files` (salsa inputs are immortal in 0.27); this set makes
    /// the deleted state explicit and auditable, and provides the foundation
    /// for the Phase M2 tracked-struct migration.
    deleted_files: Arc<HashSet<Arc<str>>>,
    /// Side-channel resolver state. The `ResolverConfig` salsa input is
    /// lazily created on first `set_resolver` call; its `revision` is
    /// bumped on every subsequent change so dependent tracked queries
    /// (e.g. `resolve_fqcn_to_path`) are invalidated. The `Arc<dyn
    /// ClassResolver>` lives off-salsa because trait objects don't
    /// participate in `salsa::Update`.
    resolver_state: Arc<parking_lot::RwLock<ResolverState>>,
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
    /// Lazily-created [`crate::db::AnalyzeFileInput`] singleton input (see
    /// [`MirDatabase::analyze_config`]). Holds the PHP version as a tracked
    /// field so `analyze_file` / `infer_function` memos invalidate on
    /// version change while keeping a stable memo key.
    analyze_config_input: Arc<parking_lot::RwLock<Option<crate::db::AnalyzeFileInput>>>,
    /// Optional disk-backed definition cache. Shared with `AnalyzerDb::stub_cache`
    /// so `collect_file_definitions` can consult it directly without going
    /// through `collect_and_ingest_file`.
    stub_cache: Arc<parking_lot::RwLock<Option<Arc<crate::stub_cache::StubSliceCache>>>>,
    /// In-process parse-result cache: content-hash → StubSlice. Populated by
    /// `collect_and_ingest_file` so that `collect_file_definitions_uncached`
    /// can skip re-parsing files that were already parsed in the same session.
    /// Keyed by blake3 hash of the source text so stale entries from prior
    /// file versions are naturally evicted (different hash → different key).
    /// DashMap (64 shards) replaces a single RwLock so parallel workers contend
    /// on independent shards instead of serialising at a single write lock.
    parse_cache: Arc<crate::parse_cache::ParseCache>,
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
    /// Per-symbol declarer counts kept in lockstep with the workspace symbol
    /// index singleton. Lets `update_workspace_index_for_file` decide whether
    /// removing a file's declaration of a name is safe (count → 0) or ambiguous
    /// (another file still declares it → fall back to full rebuild). Rebuilt
    /// wholesale by `rebuild_workspace_symbol_index`; maintained incrementally
    /// by `merge_precomputed_into_workspace_index` / `update_workspace_index_for_file`.
    index_decl_counts: Arc<parking_lot::RwLock<crate::db::IndexDeclCounts>>,
    /// Frozen, borrow-only snapshot of the workspace symbol index for a
    /// single read-only analysis pass. Set ONLY on ephemeral per-pass db
    /// clones (the batch body/class passes) via [`freeze_workspace_index`];
    /// the canonical `self.db` clone always leaves this `None`.
    ///
    /// When `Some`, `find_class_like` / `find_function` / `find_global_constant`
    /// **borrow** `&WorkspaceSymbolIndex` through it (`frozen_workspace_index`)
    /// instead of calling `workspace_index(db)`, which clones the singleton's
    /// three `Arc<FxHashMap>`s on every call — cross-core atomic refcount
    /// traffic that makes the parallel body pass scale *negatively*. The
    /// borrow moves the Arc refcount only once per worker (at `map_with`
    /// clone), so the hot path is atomic-free.
    ///
    /// Correct by construction: the index is immutable for the duration of a
    /// frozen pass (all lazy-loading completes before the freeze), so a frozen
    /// read is byte-identical to the live `workspace_index(db)` it replaces.
    ///
    /// Holds the singleton **handle** alongside the snapshot so the borrow path
    /// can register a salsa dependency by reading `handle.revision(db)` (a
    /// `Copy` field, no map clone) — see [`MirDatabase::frozen_workspace_index`].
    /// The handle is cached here (not re-read from `workspace_symbol_index_input`
    /// per call) so the hot path takes no `parking_lot` read-lock either.
    frozen_index: Option<(
        crate::db::WorkspaceSymbolIndexSingleton,
        Arc<crate::db::WorkspaceSymbolIndex>,
    )>,
    /// Pass-scoped subtype-check cache, set alongside `frozen_index` on the
    /// ephemeral body-pass clone and shared across rayon workers. `None` on the
    /// canonical / open-file db. See [`crate::db::SubtypeCache`] and
    /// [`MirDatabase::subtype_cache`].
    subtype_cache: Option<Arc<crate::db::SubtypeCache>>,
}

/// Resolver-related state held outside salsa storage. Wrapped in a
/// `parking_lot::RwLock` so `MirDbStorage::clone()` (cheap for parallel readers)
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

impl Default for MirDbStorage {
    fn default() -> Self {
        let mut db = Self {
            storage: salsa::Storage::default(),
            ref_index: Arc::default(),
            pending_ref_locs: PendingRefLocs::default(),
            source_files: Arc::default(),
            deleted_files: Arc::default(),
            resolver_state: Arc::default(),
            workspace_revision_input: Arc::default(),
            user_stub_paths: Arc::default(),
            php_version: Arc::new(parking_lot::RwLock::new(Arc::from("8.2"))),
            analyze_config_input: Arc::default(),
            stub_cache: Arc::default(),
            parse_cache: Arc::default(),
            workspace_symbol_index_input: Arc::default(),
            file_decl_snapshots: Arc::default(),
            index_decl_counts: Arc::default(),
            frozen_index: None,
            subtype_cache: None,
        };
        db.init_workspace_revision();
        db
    }
}

#[salsa::db]
impl salsa::Database for MirDbStorage {}

#[salsa::db]
impl MirDatabase for MirDbStorage {
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

    fn file_imports(&self, file: &str) -> Arc<HashMap<Name, Name>> {
        let Some(sf) = self.source_files.get(file).copied() else {
            return Arc::new(HashMap::default());
        };
        // O(1) Arc refcount inc — `slice.imports` is itself `Arc<FxHashMap<...>>`.
        Arc::clone(&crate::db::collect_file_definitions(self, sf).slice.imports)
    }

    fn global_var_type(&self, name: &str) -> Option<Type> {
        crate::db::workspace_global_vars(self).0.get(name).cloned()
    }

    fn file_import_snapshots(&self) -> Vec<crate::db::FileImportSnapshot> {
        self.source_files
            .iter()
            .map(|(path, &sf)| {
                let imports =
                    Arc::clone(&crate::db::collect_file_definitions(self, sf).slice.imports);
                (path.clone(), imports)
            })
            .collect()
    }

    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>> {
        // Route through `workspace_index` so this reads the incrementally
        // maintained singleton (same source of truth as `find_class_like`)
        // rather than the O(N) tracked `workspace_symbol_index` query, which
        // re-runs on every revision bump during indexing.
        let idx = crate::db::workspace_index(self);
        let lower = Name::new(symbol).ascii_lowercase();
        let case_sensitive = Name::new(symbol);
        // Class-like and function keys are case-folded (PHP semantics).
        // Constants are case-sensitive, so tried last without lowercasing.
        // Global variables are not indexed here — they are not FQCNs and
        // are not looked up via this method in any current caller.
        let loc = idx
            .class_like
            .get(&lower)
            .or_else(|| idx.functions.get(&lower))
            .or_else(|| idx.constants.get(&case_sensitive));
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
        let mut out = HashSet::default();
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
        self.ref_index.lock().referencers_of(symbol_key)
    }

    fn record_reference_location(&self, loc: RefLoc) {
        let mut frames = self.pending_ref_locs.0.lock();
        frames
            .last_mut()
            .expect("PendingRefLocs base frame always present")
            .push(loc);
    }

    fn take_pending_ref_locs(&self) -> Vec<RefLoc> {
        let mut frames = self.pending_ref_locs.0.lock();
        std::mem::take(
            frames
                .last_mut()
                .expect("PendingRefLocs base frame always present"),
        )
    }

    fn push_ref_loc_frame(&self) {
        self.pending_ref_locs.0.lock().push(Vec::new());
    }

    fn pop_ref_loc_frame(&self) -> Vec<RefLoc> {
        let mut frames = self.pending_ref_locs.0.lock();
        if frames.len() > 1 {
            frames.pop().expect("len checked above")
        } else {
            // Unbalanced pop: drain the base frame instead of removing it,
            // preserving the invariant that the base frame always exists.
            std::mem::take(
                frames
                    .last_mut()
                    .expect("PendingRefLocs base frame always present"),
            )
        }
    }

    fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        self.ref_index.lock().file_locations(file)
    }

    fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        self.ref_index.lock().locations_of(symbol)
    }

    fn has_reference(&self, symbol: &str) -> bool {
        self.ref_index.lock().has_reference(symbol)
    }

    fn clear_file_references(&self, file: &str) {
        self.ref_index.lock().clear_file(file);
    }

    fn all_reference_location_pairs(&self) -> Vec<(Arc<str>, Arc<str>)> {
        self.ref_index.lock().all_pairs()
    }

    fn file_referenced_symbols(&self, file: &str) -> Vec<Arc<str>> {
        self.ref_index.lock().symbols_referenced_by(file)
    }

    fn lookup_source_file(&self, path: &str) -> Option<SourceFile> {
        self.source_files.get(path).copied()
    }

    fn analyze_config(&self) -> crate::db::AnalyzeFileInput {
        if let Some(cfg) = *self.analyze_config_input.read() {
            return cfg;
        }
        let mut slot = self.analyze_config_input.write();
        // Double-checked: another clone may have created it between locks.
        if let Some(cfg) = *slot {
            return cfg;
        }
        let cfg = crate::db::AnalyzeFileInput::new(self, self.php_version.read().clone());
        *slot = Some(cfg);
        cfg
    }

    fn resolver_config(&self) -> Option<ResolverConfig> {
        self.resolver_state.read().config
    }

    fn current_resolver(&self) -> Option<Arc<dyn crate::ClassResolver>> {
        self.resolver_state.read().resolver.clone()
    }

    fn workspace_revision(&self) -> Option<WorkspaceRevision> {
        *self.workspace_revision_input.read()
    }

    fn workspace_symbol_index_singleton(&self) -> Option<crate::db::WorkspaceSymbolIndexSingleton> {
        *self.workspace_symbol_index_input.read()
    }

    fn frozen_workspace_index(&self) -> Option<&crate::db::WorkspaceSymbolIndex> {
        let (handle, index) = self.frozen_index.as_ref()?;
        // Register a salsa dependency on the workspace index BEFORE the caller's
        // map lookup (and unconditionally, so even a miss — a negative result —
        // carries the dep). Reading the `Copy` `revision` field is a real input
        // read that joins the active tracked query's dep set, but clones no maps.
        // This is what keeps `class_ancestors_by_fqcn` & friends correct across
        // a mid-run index mutation while still avoiding the per-call Arc clones.
        let _ = handle.revision(self);
        Some(index.as_ref())
    }

    fn subtype_cache(&self) -> Option<&crate::db::SubtypeCache> {
        self.subtype_cache.as_deref()
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

    fn parse_cache(&self) -> Arc<crate::parse_cache::ParseCache> {
        self.parse_cache.clone()
    }
}

impl MirDbStorage {
    /// Wire a disk-backed stub cache into this db so `collect_file_definitions`
    /// can skip reparsing on cache hits. Called by `AnalyzerDb::with_cache_dir`.
    pub fn set_stub_cache(&self, cache: Arc<crate::stub_cache::StubSliceCache>) {
        *self.stub_cache.write() = Some(cache);
    }

    /// Store a pre-computed `StubSlice` for a given content hash so that
    /// `collect_file_definitions_uncached` can skip re-parsing files already
    /// processed by `collect_and_ingest_file` in the same session.
    pub fn prime_parse_cache(&self, hash: [u8; 32], slice: Arc<StubSlice>) {
        self.parse_cache.insert(hash, slice);
    }

    /// Snapshot the current workspace symbol index into this db clone's
    /// borrow-only `frozen_index`, so a subsequent read-only analysis pass can
    /// **borrow** the index per `find_class_like` call instead of cloning the
    /// singleton's three `Arc<FxHashMap>`s on every lookup.
    ///
    /// Call this ONLY on an ephemeral, per-pass `MirDbStorage` clone (e.g. the
    /// `db_main` used by the batch body pass) **after all index mutation for
    /// that pass has completed** (all lazy-loading done). The frozen view is
    /// then immutable for the pass, so each borrowed read is byte-identical to
    /// the `workspace_index(self)` clone it replaces — no staleness window.
    ///
    /// Never call this on the canonical `self.db` clone: leaving `frozen_index`
    /// `None` there is what keeps the open-file / LSP path (which mutates the
    /// index mid-analysis via lazy-load) reading the live singleton.
    pub fn freeze_workspace_index(&mut self) {
        // Only freeze the index when the singleton is populated: the borrow path
        // anchors its salsa dep on the singleton's `revision` field. With no
        // singleton (e.g. unit-test dbs that never rebuild), leave `frozen_index`
        // None so callers fall back to the live `workspace_index(db)`.
        if let Some(handle) = self.workspace_symbol_index_singleton() {
            let index = handle.index(self);
            self.frozen_index = Some((handle, Arc::new(index)));
        }
        // Begin the pass-scoped subtype cache. Sound regardless of the singleton:
        // its validity rests only on the class graph being immutable for the
        // pass (the caller freezes after all lazy-loading), the same invariant
        // as the frozen index. Dropped when this ephemeral db clone is dropped.
        self.subtype_cache = Some(Arc::new(crate::db::SubtypeCache::default()));
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
    /// input field).  Typical call sites: end of `collect_definitions`, end of
    /// `AnalysisSession::rebuild_workspace_symbol_index`, and after any
    /// `ingest_file` that detects a declaration change.
    pub fn rebuild_workspace_symbol_index(&mut self) {
        use crate::db::{
            collect_file_declarations, FileDeclarations, IndexDeclCounts, SymbolLoc,
            WorkspaceSymbolIndex,
        };

        let files = self.all_source_files();
        let user_stubs = self.user_stub_paths.read().clone();

        let mut class_like: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut functions: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut constants: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut counts = IndexDeclCounts::default();
        let mut new_snapshots: FxHashMap<SourceFile, FileDeclarations> = FxHashMap::default();

        // Single pass over all files. Tier-aware insertion (native stub <
        // user file < user stub) makes the result independent of iteration
        // order for distinct symbols and matches the 3-pass precedence of the
        // tracked `workspace_symbol_index` query — the incremental merge path
        // reuses the exact same `tier_insert` rule.
        {
            let db: &dyn MirDatabase = &*self;
            for &file in files.iter() {
                let tier = symbol_tier(file, db, &user_stubs);
                let decls = collect_file_declarations(db, file);
                for (key, loc) in &decls.class_like {
                    tier_insert(
                        &mut class_like,
                        &mut counts.class_like,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &decls.functions {
                    tier_insert(
                        &mut functions,
                        &mut counts.functions,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &decls.constants {
                    tier_insert(
                        &mut constants,
                        &mut counts.constants,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                new_snapshots.insert(file, decls);
            }
        }

        *self.file_decl_snapshots.write() = new_snapshots;
        *self.index_decl_counts.write() = counts;

        let new_index = WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };
        self.set_workspace_index(new_index);
    }

    /// Commit a freshly-built [`WorkspaceSymbolIndex`] onto the singleton
    /// input (creating it on first call), at `Durability::HIGH`. Skips the
    /// write when the new index is `Arc::ptr_eq`-equal to the existing one.
    fn set_workspace_index(&mut self, new_index: crate::db::WorkspaceSymbolIndex) {
        use crate::db::WorkspaceSymbolIndexSingleton;
        use salsa::Setter as _;
        let existing = *self.workspace_symbol_index_input.read();
        match existing {
            Some(s) => {
                let old = s.index(self);
                if old != new_index {
                    // Bump `revision` in lockstep with `index` so frozen-path
                    // readers (which anchor on `revision` to avoid cloning the
                    // maps) are invalidated exactly when the index changes.
                    let next = s.revision(self).wrapping_add(1);
                    s.set_index(self)
                        .with_durability(salsa::Durability::HIGH)
                        .to(new_index);
                    s.set_revision(self)
                        .with_durability(salsa::Durability::HIGH)
                        .to(next);
                }
            }
            None => {
                let s = WorkspaceSymbolIndexSingleton::builder(new_index, 0)
                    .durability(salsa::Durability::HIGH)
                    .new(self);
                *self.workspace_symbol_index_input.write() = Some(s);
            }
        }
    }

    /// Build the workspace symbol index from **precomputed** per-file
    /// declarations — no `collect_file_declarations` (no parse) runs here, so
    /// the write window is just map construction. The caller computes `decls`
    /// off-lock (on a snapshot). Used by [`crate::AnalysisSession::index_batch`]
    /// for the first chunk that seeds the singleton, keeping the write-lock hold
    /// short even on a large initial set.
    pub fn build_workspace_index_from_decls(
        &mut self,
        decls: Vec<(SourceFile, crate::db::FileDeclarations)>,
    ) {
        use crate::db::{IndexDeclCounts, SymbolLoc};
        let user_stubs = self.user_stub_paths.read().clone();
        let mut class_like: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut functions: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut constants: FxHashMap<Name, SymbolLoc> = FxHashMap::default();
        let mut counts = IndexDeclCounts::default();
        let mut snaps = FxHashMap::default();
        {
            let db: &dyn MirDatabase = &*self;
            for (file, d) in &decls {
                let tier = symbol_tier(*file, db, &user_stubs);
                for (key, loc) in &d.class_like {
                    tier_insert(
                        &mut class_like,
                        &mut counts.class_like,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &d.functions {
                    tier_insert(
                        &mut functions,
                        &mut counts.functions,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &d.constants {
                    tier_insert(
                        &mut constants,
                        &mut counts.constants,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
            }
        }
        for (file, d) in decls {
            snaps.insert(file, d);
        }
        *self.file_decl_snapshots.write() = snaps;
        *self.index_decl_counts.write() = counts;
        let new_index = crate::db::WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };
        self.set_workspace_index(new_index);
    }

    /// Incrementally merge **precomputed** per-file declarations into the
    /// existing singleton — no parse runs under the lock. Mirror of
    /// [`Self::merge_precomputed_into_workspace_index`] but with the (off-lock)
    /// declaration collection already done by the caller. Files already present
    /// in `file_decl_snapshots` are skipped (avoids double-counting). No-op if
    /// the singleton hasn't been created yet.
    pub fn merge_precomputed_into_workspace_index(
        &mut self,
        decls: &[(SourceFile, crate::db::FileDeclarations)],
    ) {
        let Some(singleton) = *self.workspace_symbol_index_input.read() else {
            let mut snaps = self.file_decl_snapshots.write();
            for (file, d) in decls {
                snaps.entry(*file).or_insert_with(|| d.clone());
            }
            return;
        };
        let user_stubs = self.user_stub_paths.read().clone();
        let cur = singleton.index(self);
        let mut class_like = (*cur.class_like).clone();
        let mut functions = (*cur.functions).clone();
        let mut constants = (*cur.constants).clone();
        let mut counts = self.index_decl_counts.write();
        let mut to_store: Vec<(SourceFile, crate::db::FileDeclarations)> = Vec::new();
        {
            let db: &dyn MirDatabase = &*self;
            let snaps = self.file_decl_snapshots.read();
            for (file, d) in decls {
                if snaps.contains_key(file) {
                    continue;
                }
                let tier = symbol_tier(*file, db, &user_stubs);
                for (key, loc) in &d.class_like {
                    tier_insert(
                        &mut class_like,
                        &mut counts.class_like,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &d.functions {
                    tier_insert(
                        &mut functions,
                        &mut counts.functions,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                for (key, loc) in &d.constants {
                    tier_insert(
                        &mut constants,
                        &mut counts.constants,
                        *key,
                        *loc,
                        tier,
                        db,
                        &user_stubs,
                    );
                }
                to_store.push((*file, d.clone()));
            }
        }
        drop(counts);
        {
            let mut snaps = self.file_decl_snapshots.write();
            for (file, d) in to_store {
                snaps.insert(file, d);
            }
        }
        let new_index = crate::db::WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };
        self.set_workspace_index(new_index);
    }

    /// Incrementally update the singleton for ONE edited file: subtract its old
    /// declarations (from `file_decl_snapshots`) and add its new ones.
    ///
    /// Returns `true` if the update was applied incrementally. Returns `false`
    /// — having made **no** mutation — when subtraction is ambiguous (a removed
    /// name is still declared by another file and this file was the winning
    /// entry, so the replacement loc is unknown); the caller must then call
    /// [`Self::rebuild_workspace_symbol_index`].
    ///
    /// Also returns `false` if no singleton exists yet (nothing to update).
    pub fn update_workspace_index_for_file(&mut self, file: SourceFile) -> bool {
        use crate::db::{collect_file_declarations, SymbolLoc};
        let Some(singleton) = *self.workspace_symbol_index_input.read() else {
            return false;
        };
        let user_stubs = self.user_stub_paths.read().clone();
        let old_decls = self.file_decl_snapshots.read().get(&file).cloned();

        // Compute the new declarations once. If the declared NAMES are
        // unchanged (body-only edit — `FileDeclarations` PartialEq is name-only)
        // do nothing: no singleton write, so the HIGH-durability dep does not
        // invalidate body-analysis memos. This is the warm-cache guarantee.
        let new_decls = {
            let db: &dyn MirDatabase = &*self;
            collect_file_declarations(db, file)
        };
        if old_decls.as_ref() == Some(&new_decls) {
            return true;
        }

        let tier = {
            let db: &dyn MirDatabase = &*self;
            symbol_tier(file, db, &user_stubs)
        };

        let cur = singleton.index(self);
        let mut class_like = (*cur.class_like).clone();
        let mut functions = (*cur.functions).clone();
        let mut constants = (*cur.constants).clone();
        let mut counts = self.index_decl_counts.write();

        // Dry-run ambiguity check on the OLD decls before mutating anything.
        if let Some(old) = &old_decls {
            let ambiguous = |entries: &[(Name, SymbolLoc)],
                             map: &FxHashMap<Name, SymbolLoc>,
                             cnt: &FxHashMap<Name, u32>|
             -> bool {
                entries.iter().any(|(key, _)| {
                    let c = cnt.get(key).copied().unwrap_or(0);
                    // count > 1 means another file also declares it; if this
                    // file currently owns the winning entry we can't cheaply
                    // recompute the replacement → ambiguous.
                    c > 1 && map.get(key).map(|l| l.file()) == Some(file)
                })
            };
            if ambiguous(&old.class_like, &class_like, &counts.class_like)
                || ambiguous(&old.functions, &functions, &counts.functions)
                || ambiguous(&old.constants, &constants, &counts.constants)
            {
                return false;
            }
        }

        // Subtract old decls.
        if let Some(old) = &old_decls {
            subtract_decls(
                &mut class_like,
                &mut counts.class_like,
                &old.class_like,
                file,
            );
            subtract_decls(&mut functions, &mut counts.functions, &old.functions, file);
            subtract_decls(&mut constants, &mut counts.constants, &old.constants, file);
        }

        // Add new decls.
        {
            let db: &dyn MirDatabase = &*self;
            for (key, loc) in &new_decls.class_like {
                tier_insert(
                    &mut class_like,
                    &mut counts.class_like,
                    *key,
                    *loc,
                    tier,
                    db,
                    &user_stubs,
                );
            }
            for (key, loc) in &new_decls.functions {
                tier_insert(
                    &mut functions,
                    &mut counts.functions,
                    *key,
                    *loc,
                    tier,
                    db,
                    &user_stubs,
                );
            }
            for (key, loc) in &new_decls.constants {
                tier_insert(
                    &mut constants,
                    &mut counts.constants,
                    *key,
                    *loc,
                    tier,
                    db,
                    &user_stubs,
                );
            }
        }
        drop(counts);
        self.file_decl_snapshots.write().insert(file, new_decls);

        let new_index = crate::db::WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };
        self.set_workspace_index(new_index);
        true
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

    /// Commit a batch of reference locations into the shared index in one
    /// lock acquisition.  Must be called serially after all parallel workers
    /// have dropped their db clones and returned their pending buffers.
    pub fn commit_reference_locations_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        self.ref_index.lock().append_batch(locs);
    }

    /// Replace `file`'s reference locations wholesale (clear + append) in
    /// one lock acquisition. Use when `locs` is known to be the file's
    /// *complete* reference set — fresh `analyze_file` output or a
    /// disk-cache replay. Entries in `locs` belonging to other files (e.g.
    /// recorded by nested on-demand inference) are appended without
    /// clearing those files.
    pub fn set_file_reference_locations(&self, file: &str, locs: Vec<RefLoc>) {
        self.ref_index.lock().set_file_refs(file, locs);
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
        *self.php_version.write() = version.clone();
        // Write through to the AnalyzeFileInput singleton (if it has been
        // created) so tracked queries reading `cfg.php_version(db)` are
        // invalidated. Never hold the lock across the salsa setter.
        let existing = *self.analyze_config_input.read();
        if let Some(cfg) = existing {
            use salsa::Setter as _;
            cfg.set_php_version(self).to(version);
        }
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
            Arc::make_mut(&mut self.deleted_files).remove(path.as_ref());
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

    /// Remove the Salsa SourceFile handle for `path` from the registry and
    /// drop its contribution to the workspace symbol index singleton.
    ///
    /// The index must be updated here: with `bump_workspace_revision` no longer
    /// nulling the singleton, a stale entry could otherwise keep resolving a
    /// removed file's symbols (the salsa input itself is never deleted). If the
    /// incremental subtract is ambiguous, the singleton is nulled so the next
    /// lookup falls back to the tracked query / a finalize rebuilds it.
    pub fn remove_source_file(&mut self, path: &str) {
        let sf = self.source_files.get(path).copied();
        if Arc::make_mut(&mut self.source_files).remove(path).is_some() {
            Arc::make_mut(&mut self.deleted_files).insert(Arc::from(path));
            if let Some(sf) = sf {
                if self.workspace_symbol_index_input.read().is_some()
                    && !self.remove_file_from_workspace_index(sf)
                {
                    *self.workspace_symbol_index_input.write() = None;
                }
                self.file_decl_snapshots.write().remove(&sf);
                // Free the file text. The salsa input slot is immortal in 0.27
                // (no delete API), but the Arc<str> content — potentially hundreds
                // of KB per file — can be dropped now. This also invalidates any
                // still-cached memo for this file, accelerating LRU eviction.
                {
                    use salsa::Setter as _;
                    sf.set_text(self)
                        .with_durability(salsa::Durability::LOW)
                        .to(Arc::from(""));
                }
            }
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
    ///
    /// **Does NOT null the workspace symbol index singleton.** In the
    /// eager-static-input model the singleton is maintained incrementally
    /// (`merge_precomputed_into_workspace_index` on add, `update_/remove_..._index`
    /// on edit/remove). Nulling here was the source of the warm-cache churn:
    /// every lazily-added file destroyed the singleton and forced an O(N)
    /// fallback rebuild that cascade-invalidated body-analysis memos. The
    /// invariant is now: **whoever mutates the input set is responsible for
    /// refreshing the singleton** (bulk-register paths call merge/finalize;
    /// edits call update; removes call remove). Steady-state body-only edits
    /// add no files, never bump, and never touch the singleton.
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

    /// Subtract one file's declarations from the singleton without re-adding
    /// (used on file removal). Returns `false` (no mutation) if subtraction is
    /// ambiguous — the caller then nulls the singleton to force a fallback
    /// rebuild. Returns `true` if applied or if there's nothing to do.
    fn remove_file_from_workspace_index(&mut self, file: SourceFile) -> bool {
        let Some(singleton) = *self.workspace_symbol_index_input.read() else {
            return true;
        };
        let old_decls = self.file_decl_snapshots.read().get(&file).cloned();
        let Some(old) = old_decls else {
            return true; // never indexed → nothing to subtract
        };
        let cur = singleton.index(self);
        let mut class_like = (*cur.class_like).clone();
        let mut functions = (*cur.functions).clone();
        let mut constants = (*cur.constants).clone();
        let mut counts = self.index_decl_counts.write();

        let ambiguous = |entries: &[(Name, crate::db::SymbolLoc)],
                         map: &FxHashMap<Name, crate::db::SymbolLoc>,
                         cnt: &FxHashMap<Name, u32>|
         -> bool {
            entries.iter().any(|(key, _)| {
                cnt.get(key).copied().unwrap_or(0) > 1
                    && map.get(key).map(|l| l.file()) == Some(file)
            })
        };
        if ambiguous(&old.class_like, &class_like, &counts.class_like)
            || ambiguous(&old.functions, &functions, &counts.functions)
            || ambiguous(&old.constants, &constants, &counts.constants)
        {
            return false;
        }
        subtract_decls(
            &mut class_like,
            &mut counts.class_like,
            &old.class_like,
            file,
        );
        subtract_decls(&mut functions, &mut counts.functions, &old.functions, file);
        subtract_decls(&mut constants, &mut counts.constants, &old.constants, file);
        drop(counts);

        let new_index = crate::db::WorkspaceSymbolIndex {
            class_like: Arc::new(class_like),
            functions: Arc::new(functions),
            constants: Arc::new(constants),
        };
        self.set_workspace_index(new_index);
        true
    }

    /// Current workspace generation counter. Bumped on every file add/remove.
    /// Exposed as the "are we up to date" epoch for background indexing
    /// ([`crate::AnalysisSession::index_generation`]).
    pub fn workspace_revision_value(&self) -> u64 {
        self.workspace_revision_input
            .read()
            .map(|r| r.revision(self))
            .unwrap_or(0)
    }

    /// Number of source files currently registered.
    pub fn source_file_count(&self) -> usize {
        self.source_files.len()
    }

    /// All registered source file paths.
    pub fn source_file_paths(&self) -> Vec<Arc<str>> {
        self.source_files.keys().cloned().collect()
    }

    /// Reset `file`'s reference-location index before re-analysis. Name
    /// data itself is derived lazily from `collect_file_definitions` and
    /// doesn't need explicit clearing.
    pub fn remove_file_definitions(&mut self, file: &str) {
        self.clear_file_references(file);
    }
}
