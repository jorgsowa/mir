//! Session-based analysis API for incremental, per-file analysis.
//!
//! [`AnalysisSession`] owns the salsa database and per-session caches for a
//! long-running analysis context shared across many per-file analyses. Reads
//! clone the database under a brief lock, then run lock-free; writes hold the
//! lock briefly to mutate canonical state. `MirDb::clone()` is cheap
//! (Arc-wrapped registries), so this pattern gives parallel readers without
//! blocking on concurrent writes for longer than the clone itself.
//!
//! See [`crate::file_analyzer::FileAnalyzer`] for the per-file analysis
//! entry point that operates against a session.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use mir_codebase::{FileId, FileIdMap};

use crate::analyzer_db::AnalyzerDb;
use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{MirDatabase, MirDb, RefLoc};
use crate::php_version::PhpVersion;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
///
/// Cheap to clone the inner db for parallel reads; writes funnel through
/// [`Self::ingest_file`], [`Self::invalidate_file`], and the crate-internal
/// [`Self::with_db_mut`].
pub struct AnalysisSession {
    /// Shared database management (salsa, file registry, stub tracking).
    pub(crate) db: Arc<AnalyzerDb>,
    pub(crate) cache: Option<Arc<AnalysisCache>>,
    /// PSR-4 / Composer autoload map. Retained alongside `resolver` so the
    /// `psr4()` accessor can still return a typed `Psr4Map` for callers that
    /// need Composer-specific data (project_files / vendor_files / etc.).
    pub(crate) psr4: Option<Arc<Psr4Map>>,
    /// Generic class resolver used for on-demand lazy loading. When `psr4`
    /// is set via [`Self::with_psr4`], this is populated with the same map
    /// re-typed as `dyn ClassResolver`. Consumers can also supply their own
    /// resolver via [`Self::with_class_resolver`] without going through
    /// Composer.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
    pub(crate) php_version: PhpVersion,
    pub(crate) user_stub_files: Vec<PathBuf>,
    pub(crate) user_stub_dirs: Vec<PathBuf>,
    /// Path ↔ FileId mapping shared with `reverse_dep_map`.
    file_id_map: Arc<RwLock<FileIdMap>>,
    /// In-memory reverse dependency map: target_file → set of files that
    /// depend on it. Always maintained (not gated on disk cache presence),
    /// enabling `reanalyze_dependents` and `dependency_graph()` without a
    /// disk cache. Updated in `ingest_file` and `invalidate_file`.
    reverse_dep_map: Arc<RwLock<HashMap<FileId, HashSet<FileId>>>>,
    /// Tracks symbols that were previously defined in a file but have since
    /// been removed (deleted or renamed). When `ingest_file` detects that
    /// a symbol disappears, it records it here so `dependency_graph()` can
    /// still produce edges to files that reference the now-gone symbol.
    ///
    /// Keyed by the file that used to define the symbols. Symbols are removed
    /// from the set when re-added to the same file on a subsequent ingest.
    /// The set may contain symbols with no current referencers; those are
    /// harmless — the `symbol_referencers_of` lookup returns empty.
    stale_defined_symbols: Arc<RwLock<HashMap<String, HashSet<Arc<str>>>>>,
    /// Negative cache: FQCNs that `load_class` already failed on.
    /// The value is the resolver-mapped path (when known) so eviction on
    /// `set_file_text` / `ingest_file` is a path equality check rather than
    /// re-running the resolver per entry. `None` means the resolver itself
    /// couldn't map the FQCN; those entries survive file edits (no source
    /// change makes a never-resolvable name resolvable).
    /// Bounded to `UNRESOLVABLE_CACHE_CAP`; clears on overflow.
    unresolvable_fqcns: UnresolvableCache,
    /// Pluggable source-text provider for lazy-load. Defaults to filesystem
    /// reads ([`crate::FsSourceProvider`]); LSPs swap in a VFS-backed
    /// implementation so unsaved buffers override on-disk content.
    source_provider: Arc<dyn crate::SourceProvider>,
}

/// FQCN → optional resolver-mapped path. See the field doc on
/// `AnalysisSession::unresolvable_fqcns`.
type UnresolvableCache = Arc<RwLock<HashMap<Arc<str>, Option<Arc<str>>>>>;

/// Cap on the negative-resolution cache. Sized to accommodate a large
/// workspace's worth of genuinely-missing references without unbounded
/// growth. On overflow the cache is cleared; the cost is a few extra
/// resolver calls until it re-fills.
const UNRESOLVABLE_CACHE_CAP: usize = 10_000;

impl AnalysisSession {
    /// Create a session targeting the given PHP language version.
    pub fn new(php_version: PhpVersion) -> Self {
        Self {
            db: Arc::new(AnalyzerDb::new()),
            cache: None,
            psr4: None,
            resolver: None,
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
            file_id_map: Arc::new(RwLock::new(FileIdMap::new())),
            reverse_dep_map: Arc::new(RwLock::new(HashMap::default())),
            stale_defined_symbols: Arc::new(RwLock::new(HashMap::default())),
            unresolvable_fqcns: Arc::new(RwLock::new(HashMap::default())),
            source_provider: Arc::new(crate::FsSourceProvider),
        }
    }

    /// Swap in a custom [`crate::SourceProvider`]. LSPs install a VFS-backed
    /// provider here so the analyzer reads from unsaved editor buffers
    /// instead of disk.
    pub fn with_source_provider(mut self, provider: Arc<dyn crate::SourceProvider>) -> Self {
        self.source_provider = provider;
        self
    }

    /// Attach a pre-built [`AnalysisCache`] (the body-analysis issue cache) and
    /// open a sibling definition [`StubSlice`] cache under the same root, so
    /// callers using this builder get the same speedup as `with_cache_dir`.
    ///
    /// Rebuilds the shared database to attach the definition cache — call
    /// **before** any file is ingested. A debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::storage::StubSlice
    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache must be called before any file is ingested"
        );
        let dir = cache.cache_dir().to_path_buf();
        self.db = Arc::new(AnalyzerDb::new().with_cache_dir(&dir));
        self.cache = Some(cache);
        self
    }

    /// Convenience: open a disk-backed cache at `cache_dir` and attach it.
    ///
    /// Attaches both the body-analysis issue cache ([`AnalysisCache`]) and the
    /// definition [`StubSlice`] cache to the shared database. Builds a fresh
    /// [`AnalyzerDb`] internally — call **before** any file is ingested. A
    /// debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::storage::StubSlice
    pub fn with_cache_dir(mut self, cache_dir: &std::path::Path) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache_dir must be called before any file is ingested"
        );
        self.db = Arc::new(AnalyzerDb::new().with_cache_dir(cache_dir));
        self.cache = Some(Arc::new(AnalysisCache::open(cache_dir)));
        self
    }

    /// Attach a Composer autoload map (PSR-4, PSR-0, classmap, files).
    /// Sets the same map as the active [`crate::ClassResolver`] so
    /// [`Self::load_class`] works out of the box.
    pub fn with_psr4(mut self, map: Arc<Psr4Map>) -> Self {
        let user_resolver: Arc<dyn crate::ClassResolver> = map.clone();
        // Wrap with stub awareness so `find_class_like` / `resolve_fqcn_to_path`
        // can map built-in PHP class FQCNs (`ArrayObject`, `Exception`, …)
        // to their stub virtual paths.
        let resolver: Arc<dyn crate::ClassResolver> = Arc::new(crate::ChainedClassResolver::new(
            user_resolver,
            Arc::new(crate::StubClassResolver),
        ));
        self.psr4 = Some(map);
        self.resolver = Some(resolver.clone());
        // Mirror into MirDb so salsa-tracked resolver queries
        // (`db::resolve_fqcn_to_path`, Phase 2) see the same resolver and
        // are invalidated on swap.
        self.db.salsa.write().set_resolver(Some(resolver));
        self
    }

    /// Attach a generic class resolver for projects that don't use Composer
    /// (WordPress, Drupal, custom autoloaders, workspace-walk indexes).
    /// Replaces any previously-set Composer-backed resolver. Automatically
    /// wrapped with stub awareness so PHP built-ins remain resolvable.
    pub fn with_class_resolver(mut self, resolver: Arc<dyn crate::ClassResolver>) -> Self {
        let wrapped: Arc<dyn crate::ClassResolver> = Arc::new(crate::ChainedClassResolver::new(
            resolver,
            Arc::new(crate::StubClassResolver),
        ));
        self.db.salsa.write().set_resolver(Some(wrapped.clone()));
        self.resolver = Some(wrapped);
        self
    }

    pub fn with_user_stubs(mut self, files: Vec<PathBuf>, dirs: Vec<PathBuf>) -> Self {
        self.user_stub_files = files;
        self.user_stub_dirs = dirs;
        self
    }

    pub fn php_version(&self) -> PhpVersion {
        self.php_version
    }

    pub fn cache(&self) -> Option<&AnalysisCache> {
        self.cache.as_deref()
    }

    pub fn psr4(&self) -> Option<&Psr4Map> {
        self.psr4.as_deref()
    }

    /// Load only the curated set of essential stubs (Core, standard, SPL,
    /// date) plus any configured user stubs. About 25 of 120 stub files;
    /// covers types and functions used by virtually all PHP code.
    ///
    /// Other extension stubs (Reflection, gd, openssl, …) can be brought in
    /// on demand via [`Self::ensure_stubs_for_ast`] when user code references
    /// them. Idempotent — already-loaded stubs are skipped.
    pub fn ensure_essential_stubs(&self) {
        self.db
            .ingest_stub_paths(crate::stubs::ESSENTIAL_STUB_PATHS, self.php_version);
        self.ensure_user_stubs_loaded();
    }

    /// Load every embedded PHP stub plus any configured user stubs.
    /// Use for batch tools (CLI, full project analysis) where comprehensive
    /// symbol coverage matters more than cold-start latency.
    pub fn ensure_all_stubs(&self) {
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.db.ingest_stub_paths(&paths, self.php_version);
        self.ensure_user_stubs_loaded();
    }

    /// Ensure the embedded stub that defines `name` (a function) is ingested.
    /// Returns `true` when a matching stub exists (whether or not it was
    /// already loaded), `false` when `name` isn't a known PHP built-in.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead —
    /// it auto-discovers needed stubs from a parsed file.
    #[doc(hidden)]
    pub fn ensure_stub_for_function(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_function(name) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `fqcn` (a class / interface /
    /// trait / enum) is ingested. Case-insensitive lookup with optional
    /// leading backslash.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead.
    #[doc(hidden)]
    pub fn ensure_stub_for_class(&self, fqcn: &str) -> bool {
        match crate::stubs::stub_path_for_class(fqcn) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `name` (a constant) is ingested.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead.
    #[doc(hidden)]
    pub fn ensure_stub_for_constant(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_constant(name) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Number of distinct embedded stubs currently ingested into the session.
    /// Useful for diagnostics and bench reporting.
    pub fn loaded_stub_count(&self) -> usize {
        self.db.loaded_stubs.lock().len()
    }

    /// Auto-discover and ingest the embedded stubs needed to cover every
    /// built-in PHP function / class / constant referenced by `source`.
    ///
    /// Used by [`crate::FileAnalyzer::analyze`] to keep essentials-only mode
    /// correct without forcing callers to enumerate which stubs they need.
    /// Idempotent — already-loaded stubs are skipped via [`Self::loaded_stubs`].
    ///
    /// The discovery scan is a coarse identifier sweep (see
    /// [`crate::stubs::collect_referenced_builtin_paths`]) — it may pull in
    /// a slightly larger set than the file strictly needs, but never misses
    /// a referenced built-in. Cost is sub-millisecond per file.
    ///
    /// Fast path: if every embedded stub is already loaded (e.g. after a
    /// batch tool called [`Self::ensure_all_stubs`]), the source scan
    /// is skipped entirely.
    pub fn ensure_stubs_for_source(&self, source: &str) {
        // Cheap check first: skip the scan entirely when we already know we
        // have everything. Avoids a ~50-500µs source walk on every analyze
        // call in batch / warm-session scenarios.
        {
            let loaded = self.db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths(source);
        if paths.is_empty() {
            return;
        }
        self.db.ingest_stub_paths(&paths, self.php_version);
    }

    /// Discover and ingest stubs by walking the parsed AST of a PHP file.
    ///
    /// Similar to [`Self::ensure_stubs_for_source`], but takes an already-parsed
    /// AST instead of raw source text. Produces zero false positives since it
    /// only extracts identifiers from actual AST nodes (not from strings or
    /// comments). Preferred over `ensure_stubs_for_source` when the AST is
    /// already available (e.g., in [`crate::FileAnalyzer`]).
    ///
    /// Idempotent and skips the scan if all stubs are already loaded.
    pub fn ensure_stubs_for_ast(&self, program: &php_ast::owned::Program) {
        {
            let loaded = self.db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths_from_ast(program);
        if paths.is_empty() {
            return;
        }
        self.db.ingest_stub_paths(&paths, self.php_version);
    }

    /// Scan a parsed AST for class references and lazy-load any that are
    /// PSR-4-resolvable but not yet registered as `SourceFile` inputs. After
    /// this call, `find_class_like(fqcn)` can pull-resolve the referenced
    /// classes without needing a retry loop.
    ///
    /// The current implementation reuses [`crate::diagnostics::collect_referenced_class_fqcns`]
    /// already used by the diagnostics pass. Missing classes are passed
    /// through [`Self::load_class_transitive`] so their inheritance
    /// chain is also primed (body analysis reads parents/interfaces while
    /// resolving members).
    /// Returns true if this session has a configured class resolver
    /// (typically a PSR-4 / classmap autoloader chained with the stub
    /// resolver). Used by `FileAnalyzer` to skip the AST-scan preload
    /// when no resolver is wired up.
    pub fn has_resolver(&self) -> bool {
        self.resolver.is_some()
    }

    /// Run both pre-passes (builtin-stub loading and PSR-4 class preloading)
    /// in one call.  Replaces the two separate `ensure_stubs_for_ast` /
    /// `preload_psr4_classes_for_ast` calls at every `FileAnalyzer::analyze`
    /// site.
    pub fn prepare_ast_for_analysis(&self, program: &php_ast::owned::Program, file: &str) {
        self.ensure_stubs_for_ast(program);
        self.preload_psr4_classes_for_ast(program, file);
    }

    pub fn preload_psr4_classes_for_ast(&self, program: &php_ast::owned::Program, file: &str) {
        if self.resolver.is_none() {
            return;
        }
        let refs = collect_class_refs_from_ast(program);
        if refs.is_empty() {
            return;
        }
        // Resolve names against the file's namespace/imports up front, then
        // drop the snapshot before lazy-loading (which mutates inputs).
        let resolved: Vec<String> = {
            let db = self.snapshot_db();
            refs.into_iter()
                .map(|raw| crate::db::resolve_name_via_db(&db, file, &raw))
                .collect()
        };
        for fqcn in resolved {
            if self.contains_class(&fqcn) {
                continue;
            }
            let _ = self.load_class(&fqcn);
        }
    }

    fn ensure_user_stubs_loaded(&self) {
        self.db
            .ingest_user_stubs(&self.user_stub_files, &self.user_stub_dirs);
    }

    /// Cheap clone of the salsa db for a read-only query. The lock is held
    /// only for the duration of the clone, so concurrent readers never
    /// serialize on each other or on writes for longer than the clone itself.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice. Public consumers should use the typed query methods
    /// ([`Self::definition_of`], [`Self::hover`], etc.) instead.
    #[doc(hidden)]
    pub fn snapshot_db(&self) -> MirDb {
        self.db.snapshot_db()
    }

    /// Commit a batch of reference locations from a db snapshot into the
    /// session's shared maps.  Called by [`crate::FileAnalyzer`] and
    /// [`crate::BatchFileAnalyzer`] after parallel body analysis to flush the pending
    /// buffers that accumulate in worker db clones.
    pub(crate) fn commit_ref_locs_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        let guard = self.db.salsa.read();
        guard.commit_reference_locations_batch(locs);
    }

    /// Run a closure with read access to a database snapshot.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice.
    #[doc(hidden)]
    pub fn read<R>(&self, f: impl FnOnce(&dyn MirDatabase) -> R) -> R {
        let db = self.snapshot_db();
        f(&db)
    }

    /// definition-collection ingestion. Updates the file's source text in the salsa db,
    /// runs definition collection, and ingests the resulting stub slice.
    /// Triggers stub loading on first call. Also updates the cache's reverse-
    /// dependency graph for `file` so cross-file invalidation stays correct
    /// across incremental edits — without rebuilding the graph from scratch.
    ///
    /// If `file` was previously ingested, its old definitions and reference
    /// locations are removed first so renames / deletions don't leave stale
    /// state in the codebase. (Without this, long-running sessions would
    /// accumulate dead reference-location entries indefinitely.)
    pub fn ingest_file(&self, file: Arc<str>, source: Arc<str>) {
        self.ensure_all_stubs();

        // Snapshot symbols defined before clearing — O(symbols_in_file) with forward index.
        let old_symbols: HashSet<Arc<str>> = {
            let guard = self.db.salsa.read();
            guard.file_defined_symbols(file.as_ref())
        };

        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file.as_ref());
        }
        let _file_defs =
            self.db
                .collect_and_ingest_file(file.clone(), source.as_ref(), self.php_version);

        // Snapshot symbols after ingesting — O(symbols_in_file).
        let new_symbols: HashSet<Arc<str>> = {
            let guard = self.db.salsa.read();
            guard.file_defined_symbols(file.as_ref())
        };

        // Symbols removed from this file must be tracked so dependency_graph()
        // can still produce edges to files referencing the now-gone symbols.
        let deleted: Vec<Arc<str>> = old_symbols.difference(&new_symbols).cloned().collect();
        let re_added: Vec<Arc<str>> = new_symbols.difference(&old_symbols).cloned().collect();
        if !deleted.is_empty() || !re_added.is_empty() {
            let mut stale = self.stale_defined_symbols.write();
            let entry = stale.entry(file.as_ref().to_string()).or_default();
            for sym in deleted {
                entry.insert(sym);
            }
            for sym in &re_added {
                entry.remove(sym);
            }
            if entry.is_empty() {
                stale.remove(file.as_ref());
            }
        }

        self.update_reverse_deps_for(&file);
        // Only evict cache entries whose resolver-mapped path equals this
        // file. FQCNs the resolver can't map (psr4 miss) stay cached — no
        // ingest could change their fate. Avoids the per-keystroke storm
        // where wholesale clearing forces every unresolved FQCN to re-hit
        // the resolver on the next FileAnalyzer iteration.
        self.evict_unresolvable_for_file(&file);

        // If the workspace symbol index singleton has already been built,
        // check whether this edit changed any declared names. If so, rebuild
        // the singleton so subsequent `find_class_like` / `find_function`
        // calls see the new names. Body-only edits skip this (name-only
        // PartialEq on FileDeclarations returns equal → no rebuild → the
        // HIGH-durability singleton dep short-circuits in O(1)).
        {
            let mut guard = self.db.salsa.write();
            if guard.workspace_symbol_index_singleton().is_some() {
                if let Some(sf) = guard.lookup_source_file(file.as_ref()) {
                    if guard.file_declarations_changed(sf) {
                        guard.rebuild_workspace_symbol_index();
                    }
                }
            }
        }
    }

    /// Register `source` as the text of `file` in the salsa input layer **without**
    /// parsing or running definition collection.
    ///
    /// This is the LSP-friendly bulk-population entry point: after a workspace
    /// scan, callers can feed every discovered file's text to the session
    /// cheaply (an Arc clone plus a HashMap insert per file). Symbol resolution
    /// then happens on demand via [`Self::load_class`], which reads
    /// the file from disk through the configured [`crate::ClassResolver`] and
    /// runs definition collection lazily when a class FQCN actually needs to resolve.
    ///
    /// Contrast with [`Self::ingest_file`], which eagerly parses, runs definition collection,
    /// and populates the symbol index. Use `ingest_file` for files the user is
    /// actively editing (where in-memory text diverges from disk); use
    /// `set_file_text` for files known only through the workspace scan.
    ///
    /// Clears the negative cache: a previously-unresolvable FQCN may now
    /// resolve if its defining file is among the newly-registered set.
    pub fn set_file_text(&self, file: Arc<str>, source: Arc<str>) {
        {
            let mut guard = self.db.salsa.write();
            guard.upsert_source_file(file.clone(), source);
        }
        self.evict_unresolvable_for_file(&file);
    }

    /// Bulk-register stable vendor / library files with HIGH salsa durability.
    ///
    /// HIGH-durability files are not expected to change during the session.
    /// When a LOW-durability project file is edited, salsa can skip O(N)
    /// dependency verification for every HIGH-durability file, reducing
    /// `workspace_symbol_index` re-verification cost to O(project files only).
    ///
    /// Definition collection runs lazily on first symbol access; no parsing at call time.
    pub fn set_stable_workspace_files<I>(&self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        let mut guard = self.db.salsa.write();
        for (file, source) in files {
            guard.upsert_source_file_with_durability(file, source, salsa::Durability::HIGH);
        }
    }

    /// Build or refresh the `WorkspaceSymbolIndexSingleton` from all currently
    /// registered files.
    ///
    /// After this call, `find_class_like`, `find_function`, and
    /// `find_global_constant` read `singleton.index(db)` — a single
    /// `Durability::HIGH` tracked dep — instead of recomputing the full
    /// O(N_files) dep list via `workspace_symbol_index`. On subsequent
    /// LOW-durability (project-file) body edits the dep short-circuits in O(1).
    ///
    /// Call this once after all vendor + stub + project files have been
    /// ingested (end of workspace warm-up). Also called automatically by
    /// [`Self::ingest_file`] when a file's declared names change.
    pub fn rebuild_workspace_symbol_index(&self) {
        self.db.salsa.write().rebuild_workspace_symbol_index();
    }

    /// Bulk variant of [`Self::set_file_text`]. Acquires the salsa write lock
    /// once for the entire batch instead of once per file.
    ///
    /// The intended LSP scan loop is:
    /// ```text
    /// let files: Vec<_> = walk_workspace()
    ///     .map(|path| (path, fs::read(&path).unwrap()))
    ///     .collect();
    /// session.set_workspace_files(files);
    /// ```
    /// After this call, every file's source text is known to salsa. No
    /// parsing has happened yet — Definition collection runs per file on the first
    /// `load_class` that needs to consult it.
    pub fn set_workspace_files<I>(&self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        let registered_paths: Vec<Arc<str>> = {
            let mut guard = self.db.salsa.write();
            files
                .into_iter()
                .map(|(file, source)| {
                    guard.upsert_source_file(file.clone(), source);
                    file
                })
                .collect()
        };
        if !registered_paths.is_empty() && self.resolver.is_some() {
            self.evict_unresolvable_for_files(&registered_paths);
        }
    }

    /// Drop a file's contribution to the session: codebase definitions,
    /// reference locations, salsa input handle, cache entry, and outgoing
    /// reverse-dependency edges. Cache entries of *dependent* files are
    /// also evicted (cross-file invalidation).
    ///
    /// Use this when a file is closed by the consumer, or before a re-ingest
    /// of substantially changed content. (Plain re-ingest via
    /// [`Self::ingest_file`] also drops old definitions, but does not
    /// remove the salsa input handle — call this for full cleanup.)
    pub fn invalidate_file(&self, file: &str) {
        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file);
            guard.remove_source_file(file);
        }
        // Remove this file's outgoing deps from the in-memory reverse dep map.
        self.update_in_memory_reverse_deps(file, &HashSet::default());
        // Clear stale symbol tracking for this file — it's fully gone.
        self.stale_defined_symbols.write().remove(file);
        if let Some(cache) = &self.cache {
            cache.update_reverse_deps_for_file(file, &HashSet::default());
            cache.evict_with_dependents(&[file.to_string()]);
        }
        // The file is gone; cache entries that previously mapped to it stay
        // unresolvable until the file (or another with matching symbols) is
        // ingested again. Selective evict mirrors the ingest path.
        self.evict_unresolvable_for_file(file);
    }

    /// Number of files currently tracked in this session's salsa input set.
    /// Stable across reads; useful for diagnostics and memory bounds checks.
    pub fn tracked_file_count(&self) -> usize {
        let guard = self.db.salsa.read();
        guard.source_file_count()
    }

    // -----------------------------------------------------------------------
    // Read-only codebase queries
    //
    // All take a brief lock to clone the db, then run the lookup against the
    // owned snapshot — concurrent edits proceed without blocking.
    // -----------------------------------------------------------------------

    /// Resolve a top-level symbol (class or function) to its declaration
    /// location. Powers go-to-definition.
    ///
    /// **Side effects:** if the symbol isn't yet known, this may invoke the
    /// configured [`crate::SourceProvider`] to fault in additional files and
    /// mutate the salsa input set. Use [`Self::definition_of_cached`] for a
    /// pure variant that only consults already-loaded state.
    ///
    /// Returns:
    /// - `Ok(Location)` — symbol found with a source location
    /// - `Err(NotFound)` — no such symbol in the codebase
    /// - `Err(NoSourceLocation)` — symbol exists but has no recorded span
    ///   (e.g. some stub-only declarations)
    pub fn definition_of(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<mir_codebase::storage::Location, crate::SymbolLookupError> {
        // Trigger any necessary lazy-load mutations before snapshotting.
        match symbol {
            crate::Symbol::Class(fqcn) => {
                let _ = self.load_class(fqcn.as_ref());
            }
            crate::Symbol::Function(fqn) => {
                let _ = self.load_class(fqn.as_ref());
            }
            crate::Symbol::Method { class, .. }
            | crate::Symbol::Property { class, .. }
            | crate::Symbol::ClassConstant { class, .. } => {
                let _ = self.load_class(class.as_ref());
            }
            _ => {}
        }
        self.definition_of_cached(symbol)
    }

    /// Pure variant of [`Self::definition_of`]. Never invokes the
    /// [`crate::SourceProvider`] and never mutates salsa inputs; resolves
    /// only against state already loaded by `set_file_text` / `ingest_file`.
    /// Returns `Err(NotFound)` when the symbol isn't in the loaded set, even
    /// if a resolver could in principle map it.
    pub fn definition_of_cached(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<mir_codebase::storage::Location, crate::SymbolLookupError> {
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                class
                    .location()
                    .cloned()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                f.location
                    .clone()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Method { class, name }
            | crate::Symbol::Property { class, name }
            | crate::Symbol::ClassConstant { class, name } => {
                crate::db::member_location_via_db(&db, class, name)
                    .ok_or(crate::SymbolLookupError::NotFound)
            }
            crate::Symbol::GlobalConstant(_) => Err(crate::SymbolLookupError::NoSourceLocation),
        }
    }

    /// Hover information for a symbol: type, docstring, and definition location.
    ///
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor
    /// position, then build a [`crate::Symbol`] from its `kind`. This method
    /// assembles the displayable hover data.
    ///
    /// **Side effects:** when `symbol`'s owning class isn't yet loaded, this
    /// may invoke the configured [`crate::SourceProvider`] to fault in
    /// dependencies. Use [`Self::hover_cached`] for a pure variant.
    ///
    /// Returns `Err(NotFound)` if the symbol doesn't exist. May still return
    /// `Ok` with `docstring: None` or `definition: None` if those specific
    /// pieces aren't available.
    pub fn hover(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        // Trigger lazy loading for class-rooted symbols before snapshotting.
        // No-op when the class is already known; ensures inherited member
        // lookups have the chain present.
        match symbol {
            crate::Symbol::Class(fqcn) => {
                self.load_class(fqcn.as_ref());
            }
            crate::Symbol::Method { class, .. }
            | crate::Symbol::Property { class, .. }
            | crate::Symbol::ClassConstant { class, .. } => {
                // 10 mirrors the default depth used by reanalyze_dependents.
                self.load_class_transitive(class.as_ref(), 10);
            }
            _ => {}
        }
        self.hover_cached(symbol)
    }

    /// Pure variant of [`Self::hover`]. Never invokes the
    /// [`crate::SourceProvider`]; consults only the already-loaded db.
    pub fn hover_cached(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Union};
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = f
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Union::mixed);
                let docstring = f.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: f.location.clone(),
                })
            }
            crate::Symbol::Method { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, m) = crate::db::find_method_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = m
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Union::mixed);
                let docstring = m.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: m.location.clone(),
                })
            }
            crate::Symbol::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = Union::single(Atomic::TNamedObject {
                    fqcn: mir_types::Symbol::from(fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                });
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: class.location().cloned(),
                })
            }
            crate::Symbol::Property { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, p) = crate::db::find_property_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = p.ty.clone().unwrap_or_else(Union::mixed);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: p.location.clone(),
                })
            }
            crate::Symbol::ClassConstant { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, c) = crate::db::find_class_constant_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: c.ty.clone(),
                    docstring: None,
                    definition: c.location.clone(),
                })
            }
            crate::Symbol::GlobalConstant(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let ty = crate::db::find_global_constant(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: (*ty).clone(),
                    docstring: None,
                    definition: None,
                })
            }
        }
    }

    /// Raw reference locations indexed by string symbol key, kept for tests
    /// that use the legacy stringly-typed API. Prefer [`Self::references_to`]
    /// with a typed [`crate::Symbol`].
    #[doc(hidden)]
    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        use crate::db::MirDatabase;
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// Every recorded reference to `symbol` with its source location as a Range.
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor,
    /// build a [`crate::Symbol`] from it, and pass it here.
    pub fn references_to(&self, symbol: &crate::Symbol) -> Vec<(Arc<str>, crate::Range)> {
        let db = self.snapshot_db();
        let key = symbol.codebase_key();
        db.reference_locations(&key)
            .into_iter()
            .map(|(file, line, col_start, col_end)| {
                let range = crate::Range {
                    start: crate::Position {
                        line,
                        column: col_start as u32,
                    },
                    end: crate::Position {
                        line,
                        column: col_end as u32,
                    },
                };
                (file, range)
            })
            .collect()
    }

    /// Class-level issues (inheritance violations, abstract-method gaps, override
    /// incompatibilities) for the given set of files.
    ///
    /// These checks are cross-file by nature and are not emitted by
    /// [`crate::FileAnalyzer::analyze`]. Call this after ingesting or
    /// re-analyzing a file and its dependents to get the full diagnostic picture.
    ///
    /// Circular-inheritance checks always run against the full workspace graph
    /// regardless of the `files` filter — a cycle is a workspace-wide problem.
    pub fn class_issues(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        let db = self.snapshot_db();
        let file_set: HashSet<Arc<str>> = files.iter().cloned().collect();
        let file_data: Vec<(Arc<str>, Arc<str>)> = files
            .iter()
            .filter_map(|f| Some((f.clone(), self.source_of(f)?)))
            .collect();
        crate::class::ClassAnalyzer::with_files(&db, file_set, &file_data).analyze_all()
    }

    /// All declarations defined in `file` as a **hierarchical tree**.
    ///
    /// Classes/interfaces/traits/enums are returned with their methods,
    /// properties, and constants nested in `children`. Top-level functions
    /// and constants are returned with empty `children`.
    pub fn document_symbols(&self, file: &str) -> Vec<crate::symbol::DocumentSymbol> {
        use crate::symbol::{DocumentSymbol, DocumentSymbolKind};

        let db = self.snapshot_db();
        let Some(sf) = db.lookup_source_file(file) else {
            return Vec::new();
        };
        let defs = crate::db::collect_file_definitions(&db, sf);
        let mut out: Vec<DocumentSymbol> = Vec::new();

        let class_children =
            |methods: &indexmap::IndexMap<Arc<str>, Arc<mir_codebase::storage::MethodDef>>,
             props: Option<&indexmap::IndexMap<Arc<str>, mir_codebase::storage::PropertyDef>>,
             consts: &indexmap::IndexMap<Arc<str>, mir_codebase::storage::ConstantDef>,
             is_enum: bool|
             -> Vec<DocumentSymbol> {
                let mut out: Vec<DocumentSymbol> = Vec::new();
                for (_, m) in methods.iter() {
                    out.push(DocumentSymbol {
                        name: m.name.clone(),
                        kind: DocumentSymbolKind::Method,
                        location: m.location.clone(),
                        children: Vec::new(),
                    });
                }
                if let Some(props) = props {
                    for (_, p) in props.iter() {
                        out.push(DocumentSymbol {
                            name: p.name.clone(),
                            kind: DocumentSymbolKind::Property,
                            location: p.location.clone(),
                            children: Vec::new(),
                        });
                    }
                }
                let const_kind = if is_enum {
                    DocumentSymbolKind::EnumCase
                } else {
                    DocumentSymbolKind::Constant
                };
                for (_, c) in consts.iter() {
                    out.push(DocumentSymbol {
                        name: c.name.clone(),
                        kind: const_kind,
                        location: c.location.clone(),
                        children: Vec::new(),
                    });
                }
                out
            };

        for c in defs.slice.classes.iter() {
            out.push(DocumentSymbol {
                name: c.fqcn.clone(),
                kind: DocumentSymbolKind::Class,
                location: c.location.clone(),
                children: class_children(
                    &c.own_methods,
                    Some(&c.own_properties),
                    &c.own_constants,
                    false,
                ),
            });
        }
        for i in defs.slice.interfaces.iter() {
            out.push(DocumentSymbol {
                name: i.fqcn.clone(),
                kind: DocumentSymbolKind::Interface,
                location: i.location.clone(),
                children: class_children(&i.own_methods, None, &i.own_constants, false),
            });
        }
        for t in defs.slice.traits.iter() {
            out.push(DocumentSymbol {
                name: t.fqcn.clone(),
                kind: DocumentSymbolKind::Trait,
                location: t.location.clone(),
                children: class_children(
                    &t.own_methods,
                    Some(&t.own_properties),
                    &t.own_constants,
                    false,
                ),
            });
        }
        for e in defs.slice.enums.iter() {
            let mut children = class_children(&e.own_methods, None, &e.own_constants, true);
            for (_, case) in e.cases.iter() {
                children.push(DocumentSymbol {
                    name: case.name.clone(),
                    kind: DocumentSymbolKind::EnumCase,
                    location: case.location.clone(),
                    children: Vec::new(),
                });
            }
            out.push(DocumentSymbol {
                name: e.fqcn.clone(),
                kind: DocumentSymbolKind::Enum,
                location: e.location.clone(),
                children,
            });
        }
        for f in defs.slice.functions.iter() {
            out.push(DocumentSymbol {
                name: f.fqn.clone(),
                kind: DocumentSymbolKind::Function,
                location: f.location.clone(),
                children: Vec::new(),
            });
        }
        for (name, _) in defs.slice.constants.iter() {
            out.push(DocumentSymbol {
                name: name.clone(),
                kind: DocumentSymbolKind::Constant,
                location: None,
                children: Vec::new(),
            });
        }
        out
    }

    /// Returns `true` if a function with `fqn` is registered and active in
    /// the codebase. Case-insensitive lookup with optional leading backslash.
    pub fn contains_function(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::function_exists_via_db(&db, fqn)
    }

    /// Returns `true` if a class / interface / trait / enum with `fqcn` is
    /// registered and active in the codebase.
    pub fn contains_class(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::type_exists_via_db(&db, fqcn)
    }

    /// Returns `true` if `class` has a method named `name` registered. Method
    /// names are matched case-insensitively (PHP method dispatch semantics).
    pub fn contains_method(&self, class: &str, name: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::has_method_in_chain(&db, class, name)
    }

    /// Resolve `fqcn` via the configured [`crate::ClassResolver`] and ingest
    /// the mapped file. The session keeps a negative cache so repeated calls
    /// for an unresolvable name don't re-hit the resolver; the cache is
    /// invalidated on any [`Self::ingest_file`] / [`Self::invalidate_file`].
    ///
    /// This is the LSP-friendly entry point: the analyzer never touches
    /// `vendor/` on its own, but consumers can ask it to resolve individual
    /// symbols on demand. Designed to be called when a diagnostic would
    /// otherwise report `UndefinedClass`.
    ///
    /// Returns a [`crate::LoadOutcome`] distinguishing
    /// already-loaded / freshly-loaded / not-resolvable. Use
    /// [`crate::LoadOutcome::is_loaded`] when only success matters.
    pub fn load_class(&self, fqcn: &str) -> crate::LoadOutcome {
        if self.contains_class(fqcn) {
            return crate::LoadOutcome::AlreadyLoaded;
        }
        if self.unresolvable_fqcns.read().contains_key(fqcn) {
            return crate::LoadOutcome::NotResolvable;
        }
        if self.try_resolve_and_ingest(fqcn) {
            crate::LoadOutcome::Loaded
        } else {
            // Cache the failure with the resolver-mapped path (if any) so
            // future file edits can selectively evict.
            let resolved_path: Option<Arc<str>> = self
                .resolver
                .as_ref()
                .and_then(|r| r.resolve(fqcn))
                .map(|p| Arc::from(p.to_string_lossy().as_ref()));
            let key: Arc<str> = Arc::from(fqcn);
            let mut cache = self.unresolvable_fqcns.write();
            if cache.len() >= UNRESOLVABLE_CACHE_CAP {
                cache.clear();
            }
            cache.insert(key, resolved_path);
            crate::LoadOutcome::NotResolvable
        }
    }

    /// Inner load path: resolver lookup + ingest, no caching. Returns `true`
    /// iff `fqcn` ends up registered. Failure buckets are recorded for
    /// telemetry.
    fn try_resolve_and_ingest(&self, fqcn: &str) -> bool {
        use crate::metrics::{record_lazy_load_failure, LazyLoadFailure};
        let Some(resolver) = &self.resolver else {
            record_lazy_load_failure(LazyLoadFailure::NoResolver, fqcn);
            return false;
        };
        let Some(path) = resolver.resolve(fqcn) else {
            record_lazy_load_failure(LazyLoadFailure::ResolverNone, fqcn);
            return false;
        };
        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        // Prefer in-memory text from a prior `set_file_text` /
        // `set_workspace_files` call; fall back to disk. This makes the LSP's
        // unsaved-edit buffer authoritative over the on-disk content for the
        // same path.
        let src: Arc<str> = match self.source_of(&file) {
            Some(text) => text,
            None => match self.source_provider.read(&path.to_string_lossy()) {
                Some(text) => text,
                None => {
                    record_lazy_load_failure(LazyLoadFailure::SourceUnreadable, fqcn);
                    return false;
                }
            },
        };
        self.ingest_file(file, src);
        if self.contains_class(fqcn) {
            true
        } else {
            record_lazy_load_failure(LazyLoadFailure::IngestThenMissing, fqcn);
            false
        }
    }

    /// Lazy-load every class transitively reachable from `fqcn` via parent /
    /// interface / trait edges. Useful when the consumer needs not just the
    /// requested class but enough of its inheritance chain to type-check
    /// member access.
    ///
    /// Walks at most `max_depth` levels (default in batch analysis is 10).
    /// Returns the number of classes successfully loaded (not counting
    /// `fqcn` itself if it was already present).
    pub fn load_class_transitive(&self, fqcn: &str, max_depth: usize) -> usize {
        if self.resolver.is_none() {
            return 0;
        }
        let mut loaded = 0;
        let mut frontier: Vec<String> = vec![fqcn.to_string()];
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::default();

        for _ in 0..max_depth {
            if frontier.is_empty() {
                break;
            }
            let mut next: Vec<String> = Vec::new();
            for name in frontier.drain(..) {
                if !visited.insert(name.clone()) {
                    continue;
                }
                let was_present = self.contains_class(&name);
                let resolved = self.load_class(&name).is_loaded();
                if resolved && !was_present {
                    loaded += 1;
                    // Walk the new class's parent / interfaces / traits via pull.
                    let db = self.snapshot_db();
                    let here = crate::db::Fqcn::from_str(&db, name.as_str());
                    if let Some(class) = crate::db::find_class_like(&db, here) {
                        if let Some(parent) = class.parent() {
                            next.push(parent.to_string());
                        }
                        for iface in class.interfaces().iter() {
                            next.push(iface.to_string());
                        }
                        for tr in class.class_traits().iter() {
                            next.push(tr.to_string());
                        }
                        for ext in class.extends().iter() {
                            next.push(ext.to_string());
                        }
                    }
                }
            }
            frontier = next;
        }
        loaded
    }

    /// Evict every negative-cache entry whose stored resolver-mapped path
    /// equals `file`. FQCNs cached as never-resolvable (path `None`) are left
    /// alone — no source-text change can make them resolvable.
    fn evict_unresolvable_for_file(&self, file: &str) {
        let mut cache = self.unresolvable_fqcns.write();
        if cache.is_empty() {
            return;
        }
        cache.retain(|_fqcn, path| path.as_deref() != Some(file));
    }

    /// Bulk variant of [`Self::evict_unresolvable_for_file`]. One `HashSet`
    /// build + one pass over the cache; no resolver calls.
    fn evict_unresolvable_for_files(&self, files: &[Arc<str>]) {
        let mut cache = self.unresolvable_fqcns.write();
        if cache.is_empty() {
            return;
        }
        let registered: HashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        cache.retain(|_fqcn, path| match path {
            Some(p) => !registered.contains(p.as_ref()),
            None => true,
        });
    }

    /// Retrieve the source text the session has registered for `file`, if
    /// any. Returns `None` when the file has never been ingested. Used by
    /// the parallel re-analysis path to re-feed dependents to body analysis without
    /// the caller having to track sources independently.
    pub fn source_of(&self, file: &str) -> Option<Arc<str>> {
        let db = self.snapshot_db();
        let sf = db.lookup_source_file(file)?;
        Some(sf.text(&db))
    }

    /// Re-analyze every transitive dependent of `file` in parallel.
    ///
    /// When the user saves a file that other files depend on (e.g. editing
    /// a base class, an interface, or a trait), those dependents may have
    /// new diagnostics. This method computes them in parallel using rayon
    /// and returns the per-file analysis results so the LSP server can
    /// publish updated diagnostics in one batch.
    ///
    /// Source text for dependents is retrieved from the session's salsa
    /// inputs (set by previous `ingest_file` calls) — the caller doesn't
    /// need to track or re-read files. Files for which the session has no
    /// source are silently skipped (returns the analyzable subset).
    ///
    /// Cross-file inferred return types are resolved on demand via salsa.
    pub fn reanalyze_dependents(&self, file: &str) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        use rayon::prelude::*;

        // Phase 1: compute dependents + gather their sources outside the
        // analysis loop so each worker has everything it needs.
        let dependents = self.dependency_graph().transitive_dependents(file);
        if dependents.is_empty() {
            return Vec::new();
        }
        let with_source: Vec<(Arc<str>, Arc<str>)> = dependents
            .into_iter()
            .filter_map(|path| {
                let arc_path: Arc<str> = Arc::from(path.as_str());
                let src = self.source_of(&path)?;
                Some((arc_path, src))
            })
            .collect();
        if with_source.is_empty() {
            return Vec::new();
        }

        // Phase 2: parallel parse + analyze. Each rayon worker gets its own
        // database snapshot via FileAnalyzer; writes are isolated to the
        // session's canonical db (none happen here since we only run body analysis).
        with_source
            .into_par_iter()
            .map(|(file, source)| {
                let parsed = php_rs_parser::parse(source.as_ref());
                let analyzer = crate::FileAnalyzer::new(self);
                let analysis = analyzer.analyze(
                    file.clone(),
                    source.as_ref(),
                    &parsed.program,
                    &parsed.source_map,
                );
                (file, analysis)
            })
            .collect()
    }

    /// FQCNs that `file` imports via `use` statements but that aren't yet
    /// loaded in the session.
    ///
    /// Designed as the input to background prefetching: after the LSP server
    /// ingests an open buffer, it can call this and lazy-load the returned
    /// FQCNs on a worker thread so the user's first Cmd+Click into vendor
    /// code doesn't pay the file-read+parse cost.
    ///
    /// Returns an empty Vec if the file hasn't been ingested or has no
    /// unresolved imports.
    pub fn pending_lazy_loads(&self, file: &str) -> Vec<Arc<str>> {
        let db = self.snapshot_db();
        let imports = db.file_imports(file);
        if imports.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for fqcn in imports.values() {
            let here = crate::db::Fqcn::new(&db, *fqcn);
            if crate::db::find_class_like(&db, here).is_some() {
                continue;
            }
            if let Some(resolver) = &self.resolver {
                if resolver.resolve(fqcn.as_str()).is_some() {
                    out.push(Arc::from(fqcn.as_str()));
                }
            }
        }
        out
    }

    /// Convenience: synchronously lazy-load every import of `file` that
    /// isn't already in the codebase. Returns the number successfully loaded.
    ///
    /// For non-blocking prefetch, call this from a worker thread:
    ///
    /// ```ignore
    /// let s = session.clone();  // AnalysisSession is wrapped in Arc by callers
    /// std::thread::spawn(move || {
    ///     s.prefetch_imports(&file_path);
    /// });
    /// ```
    ///
    /// Internally walks the inheritance chain of each loaded class to a
    /// shallow depth so member access on imported types type-checks without
    /// the user paying the cost on their first navigation.
    pub fn prefetch_imports(&self, file: &str) -> usize {
        let pending = self.pending_lazy_loads(file);
        let mut loaded = 0;
        for fqcn in pending {
            // Use the transitive walker with a small depth so we pick up
            // parent classes / interfaces needed for member resolution, but
            // don't recursively pull in the entire vendor tree.
            loaded += self.load_class_transitive(&fqcn, 2);
        }
        loaded
    }

    /// All class / interface / trait / enum FQCNs currently known to the
    /// session, each paired with the file that defines them when available.
    ///
    /// Use this to build workspace-wide views (outline, fuzzy search, etc.).
    /// Consumers implement their own search/match logic on top — the analyzer
    /// only exposes the iterator.
    pub fn all_classes(&self) -> Vec<(Arc<str>, Option<mir_codebase::storage::Location>)> {
        let db = self.snapshot_db();
        crate::db::workspace_classes(&db)
            .iter()
            .filter_map(|fqcn| {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                crate::db::find_class_like(&db, here)
                    .map(|class| (fqcn.clone(), class.location().cloned()))
            })
            .collect()
    }

    /// All global function FQNs currently known to the session, each paired
    /// with their declaration location when available.
    pub fn all_functions(&self) -> Vec<(Arc<str>, Option<mir_codebase::storage::Location>)> {
        let db = self.snapshot_db();
        crate::db::workspace_functions(&db)
            .iter()
            .filter_map(|fqn| {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                crate::db::find_function(&db, here).map(|f| (fqn.clone(), f.location.clone()))
            })
            .collect()
    }

    /// Compute `file`'s outgoing dependency edges and update both the in-memory
    /// reverse-dep map (always) and the disk cache's reverse-dep graph (if configured).
    fn update_reverse_deps_for(&self, file: &str) {
        let db = self.snapshot_db();
        let targets = file_outgoing_dependencies(&db, file);

        // Always update the in-memory map.
        self.update_in_memory_reverse_deps(file, &targets);

        // Also persist to disk cache if configured.
        if let Some(cache) = self.cache.as_deref() {
            cache.update_reverse_deps_for_file(file, &targets);
        }
    }

    /// Update the in-memory reverse dependency map for `file` with `new_targets`.
    /// Removes `file` from all existing entries, then adds it as a dependent of
    /// each target in `new_targets` (excluding self-edges).
    fn update_in_memory_reverse_deps(&self, file: &str, new_targets: &HashSet<String>) {
        let file_id = self.file_id_map.write().assign_or_get(file);
        let target_ids: Vec<FileId> = {
            let mut id_map = self.file_id_map.write();
            new_targets
                .iter()
                .map(|t| id_map.assign_or_get(t))
                .collect()
        };

        let mut map = self.reverse_dep_map.write();
        for dependents in map.values_mut() {
            dependents.remove(&file_id);
        }
        map.retain(|_, dependents| !dependents.is_empty());
        for target_id in target_ids {
            if target_id != file_id {
                map.entry(target_id).or_default().insert(file_id);
            }
        }
    }

    /// BFS transitive dependents of `file` using the in-memory reverse dep map.
    ///
    /// O(D) where D is the number of transitive dependents — faster than
    /// [`Self::dependency_graph().transitive_dependents()`] which rebuilds the
    /// full graph on every call. Only covers structural dependencies from definition collection
    /// (imports, class hierarchy, type hints); does not include bare FQN body
    /// references recorded during body analysis. For full fidelity, use
    /// `dependency_graph().transitive_dependents()` after body analysis is complete.
    pub fn structural_dependents(&self, file: &str) -> Vec<String> {
        let Some(start_id) = self.file_id_map.read().get(file) else {
            return Vec::new();
        };
        let map = self.reverse_dep_map.read();
        let mut visited: HashSet<FileId> = HashSet::default();
        let mut queue = vec![start_id];
        let mut result_ids = Vec::new();
        while let Some(current_id) = queue.pop() {
            if !visited.insert(current_id) {
                continue;
            }
            if let Some(deps) = map.get(&current_id) {
                for &dep_id in deps {
                    if !visited.contains(&dep_id) {
                        queue.push(dep_id);
                        result_ids.push(dep_id);
                    }
                }
            }
        }
        drop(map);
        let id_map = self.file_id_map.read();
        result_ids
            .iter()
            .filter_map(|&id| id_map.path(id))
            .map(|s| s.to_string())
            .collect()
    }

    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// O(edges) — iterates the `file_references` forward index (file → symbol
    /// keys it references) which is always current, then resolves each symbol
    /// to its defining file via O(1) lookup.  Total cost is O(E) where E is the
    /// number of (file, symbol) reference edges, vs. the old O(F × S × R) scan.
    pub fn dependency_graph(&self) -> crate::DependencyGraph {
        let db = self.snapshot_db();

        let all_files: Vec<String> = db
            .source_file_paths()
            .iter()
            .map(|f| f.as_ref().to_string())
            .collect();

        let mut dependencies: HashMap<String, Vec<String>> = HashMap::default();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::default();

        for file in &all_files {
            // O(degree(file)) — forward index lookup, no full-table scan.
            let symbol_keys = db.file_referenced_symbols(file);
            let mut file_deps: HashSet<String> = HashSet::default();
            for symbol_key in &symbol_keys {
                let lookup: &str = match symbol_key.split_once("::") {
                    Some((class, _)) => class,
                    None => symbol_key.as_ref(),
                };
                if let Some(def_file) = db.symbol_defining_file(lookup) {
                    let def = def_file.as_ref().to_string();
                    if &def != file {
                        file_deps.insert(def);
                    }
                }
            }
            for dep in &file_deps {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(file.clone());
                dependencies
                    .entry(file.clone())
                    .or_default()
                    .push(dep.clone());
            }
        }

        // Merge structural deps from definition collection from the incremental reverse_dep_map.
        // dependency_graph() above only captures bare-FQN references recorded during body analysis;
        // the reverse_dep_map covers imports, class hierarchy (extends/implements/use),
        // and type-hint-only references that never appear in file_referenced_symbols.
        // Together they give a complete picture without requiring body analysis on every file.
        {
            let id_map = self.file_id_map.read();
            let rev = self.reverse_dep_map.read();
            for (&target_id, dep_set) in rev.iter() {
                let Some(target) = id_map.path(target_id) else {
                    continue;
                };
                let target = target.to_string();
                for &dep_id in dep_set {
                    let Some(dep) = id_map.path(dep_id) else {
                        continue;
                    };
                    let dep = dep.to_string();
                    if dep != target {
                        dependents
                            .entry(target.clone())
                            .or_default()
                            .push(dep.clone());
                        dependencies
                            .entry(dep.clone())
                            .or_default()
                            .push(target.clone());
                    }
                }
            }
        }

        for deps in dependents.values_mut() {
            deps.sort();
            deps.dedup();
        }
        for deps in dependencies.values_mut() {
            deps.sort();
            deps.dedup();
        }

        // Augment with stale dependents: files referencing symbols that were
        // deleted from their defining file. These edges disappear from the
        // symbol_defining_file lookup but the referencing file still needs
        // re-analysis to surface the now-broken reference.
        {
            let stale = self.stale_defined_symbols.read();
            if !stale.is_empty() {
                for (file, deleted_syms) in stale.iter() {
                    for sym in deleted_syms {
                        let lookup: &str = match sym.split_once("::") {
                            Some((class, _)) => class,
                            None => sym.as_ref(),
                        };
                        for referencing_file in db.symbol_referencers_of(lookup) {
                            let ref_file = referencing_file.as_ref().to_string();
                            if &ref_file != file {
                                dependents
                                    .entry(file.clone())
                                    .or_default()
                                    .push(ref_file.clone());
                                dependencies.entry(ref_file).or_default().push(file.clone());
                            }
                        }
                    }
                }
                // Re-sort and dedup since we may have added entries.
                for deps in dependents.values_mut() {
                    deps.sort();
                    deps.dedup();
                }
                for deps in dependencies.values_mut() {
                    deps.sort();
                    deps.dedup();
                }
            }
        }

        crate::DependencyGraph {
            dependencies,
            dependents,
        }
    }
}

/// Compute the set of files `file` depends on: defining files of its imports,
/// plus parent / interfaces / traits' defining files for any classes declared
/// in `file`. Self-edges are excluded.
fn file_outgoing_dependencies(db: &dyn MirDatabase, file: &str) -> HashSet<String> {
    let mut targets: HashSet<String> = HashSet::default();

    let mut add_target = |symbol: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            let def = defining_file.as_ref().to_string();
            if def != file {
                targets.insert(def);
            }
        }
    };

    let extract_named_objects = |union: &mir_types::Union| {
        union
            .types
            .iter()
            .filter_map(|atomic| match atomic {
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(*fqcn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let imports = db.file_imports(file);
    for fqcn in imports.values() {
        add_target(fqcn.as_str());
    }

    // Walk every class/interface/trait/enum/function defined in this file
    // via the pull-path slice. Push-path lookup_*_node have been retired.
    if let Some(sf) = db.lookup_source_file(file) {
        let defs = crate::db::collect_file_definitions(db, sf);
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
    }

    // Also track bare-FQN references recorded during body analysis (new \Foo(), \Foo::method(),
    // \foo()) that do not appear in use-import statements.
    for symbol_key in db.file_referenced_symbols(file) {
        let lookup: &str = match symbol_key.split_once("::") {
            Some((class, _)) => class,
            None => &symbol_key,
        };
        add_target(lookup);
    }

    targets
}

/// AST visitor that collects class FQCN references for PSR-4 preloading.
/// Captures identifiers from `new X`, static calls / property / constant
/// access, type hints, and `instanceof`. Does *not* normalize via PSR-4 /
/// imports — callers run the raw string through `resolve_name_via_db`.
fn collect_class_refs_from_ast(program: &php_ast::owned::Program) -> Vec<String> {
    use php_ast::ast::BinaryOp;
    use php_ast::owned::visitor::{
        walk_owned_catch_clause, walk_owned_expr, walk_owned_program, walk_owned_type_hint,
        OwnedVisitor,
    };
    use php_ast::owned::{ExprKind, TypeHintKind};
    use std::ops::ControlFlow;

    fn owned_name_str(name: &php_ast::owned::Name) -> String {
        let joined: String = name
            .parts
            .iter()
            .map(|p| p.as_ref())
            .collect::<Vec<&str>>()
            .join("\\");
        if name.kind == php_ast::ast::NameKind::FullyQualified {
            format!("\\{joined}")
        } else {
            joined
        }
    }

    struct V {
        names: std::collections::HashSet<String>,
    }
    impl OwnedVisitor for V {
        fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> ControlFlow<()> {
            match &expr.kind {
                ExprKind::New(n) => {
                    if let ExprKind::Identifier(name) = &n.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::StaticMethodCall(c) => {
                    if let ExprKind::Identifier(name) = &c.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::StaticPropertyAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::ClassConstAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
                    if let ExprKind::Identifier(name) = &b.right.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                _ => {}
            }
            walk_owned_expr(self, expr)
        }

        fn visit_type_hint(&mut self, hint: &php_ast::owned::TypeHint) -> ControlFlow<()> {
            if let TypeHintKind::Named(name) = &hint.kind {
                let s = owned_name_str(name);
                if !s.is_empty() {
                    self.names.insert(s);
                }
            }
            walk_owned_type_hint(self, hint)
        }

        fn visit_catch_clause(&mut self, catch: &php_ast::owned::CatchClause) -> ControlFlow<()> {
            for ty in catch.types.iter() {
                self.names.insert(owned_name_str(ty));
            }
            walk_owned_catch_clause(self, catch)
        }
    }
    let mut v = V {
        names: std::collections::HashSet::default(),
    };
    let _ = walk_owned_program(&mut v, program);
    v.names.into_iter().collect()
}
