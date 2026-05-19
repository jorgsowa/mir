//! Session-based analysis API for incremental, per-file analysis.
//!
//! [`AnalysisSession`] owns the salsa database and per-session caches for a
//! long-running analysis context shared across many per-file analyses. Reads
//! clone the database under a brief lock, then run lock-free; writes hold the
//! lock briefly to mutate canonical state. `MirDb::clone()` is cheap
//! (Arc-wrapped registries), so this pattern gives parallel readers without
//! blocking on concurrent writes for longer than the clone itself.
//!
//! See [`crate::file_analyzer::FileAnalyzer`] for the per-file Pass 2 entry
//! point that operates against a session.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{MirDatabase, MirDb, RefLoc};
use crate::php_version::PhpVersion;
use crate::shared_db::SharedDb;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
///
/// Cheap to clone the inner db for parallel reads; writes funnel through
/// [`Self::ingest_file`], [`Self::invalidate_file`], and the crate-internal
/// [`Self::with_db_mut`].
pub struct AnalysisSession {
    /// Shared database management (salsa, file registry, stub tracking).
    /// Extracted to allow code sharing with ProjectAnalyzer.
    shared_db: Arc<SharedDb>,
    cache: Option<Arc<AnalysisCache>>,
    /// PSR-4 / Composer autoload map. Retained alongside `resolver` so the
    /// `psr4()` accessor can still return a typed `Psr4Map` for callers that
    /// need Composer-specific data (project_files / vendor_files / etc.).
    psr4: Option<Arc<Psr4Map>>,
    /// Generic class resolver used for on-demand lazy loading. When `psr4`
    /// is set via [`Self::with_psr4`], this is populated with the same map
    /// re-typed as `dyn ClassResolver`. Consumers can also supply their own
    /// resolver via [`Self::with_class_resolver`] without going through
    /// Composer.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
    php_version: PhpVersion,
    user_stub_files: Vec<PathBuf>,
    user_stub_dirs: Vec<PathBuf>,
    /// In-memory reverse dependency map: target_file → set of files that
    /// depend on it. Always maintained (not gated on disk cache presence),
    /// enabling `analyze_dependents_of` and `dependency_graph()` without a
    /// disk cache. Updated in `ingest_file` and `invalidate_file`.
    reverse_dep_map: Arc<RwLock<HashMap<String, HashSet<String>>>>,
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
    /// Negative cache: FQCNs that `lookup_class_or_load` already failed on.
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
            shared_db: Arc::new(SharedDb::new()),
            cache: None,
            psr4: None,
            resolver: None,
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
            reverse_dep_map: Arc::new(RwLock::new(HashMap::new())),
            stale_defined_symbols: Arc::new(RwLock::new(HashMap::new())),
            unresolvable_fqcns: Arc::new(RwLock::new(HashMap::new())),
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

    /// Attach a pre-built [`AnalysisCache`] (the Pass-2 issue cache) and
    /// open a sibling Pass-1 [`StubSlice`] cache under the same root, so
    /// callers using this builder get the same speedup as `with_cache_dir`.
    ///
    /// Rebuilds the shared database to attach the Pass-1 cache — call
    /// **before** any file is ingested. A debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::storage::StubSlice
    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        debug_assert_eq!(
            self.shared_db.source_file_count(),
            0,
            "AnalysisSession::with_cache must be called before any file is ingested"
        );
        let dir = cache.cache_dir().to_path_buf();
        self.shared_db = Arc::new(SharedDb::new().with_cache_dir(&dir));
        self.cache = Some(cache);
        self
    }

    /// Convenience: open a disk-backed cache at `cache_dir` and attach it.
    ///
    /// Attaches both the Pass-2 issue cache ([`AnalysisCache`]) and the
    /// Pass-1 [`StubSlice`] cache to the shared database. Builds a fresh
    /// [`SharedDb`] internally — call **before** any file is ingested. A
    /// debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::storage::StubSlice
    pub fn with_cache_dir(mut self, cache_dir: &std::path::Path) -> Self {
        debug_assert_eq!(
            self.shared_db.source_file_count(),
            0,
            "AnalysisSession::with_cache_dir must be called before any file is ingested"
        );
        self.shared_db = Arc::new(SharedDb::new().with_cache_dir(cache_dir));
        self.cache = Some(Arc::new(AnalysisCache::open(cache_dir)));
        self
    }

    /// Attach a Composer autoload map (PSR-4, PSR-0, classmap, files).
    /// Sets the same map as the active [`crate::ClassResolver`] so
    /// [`Self::lazy_load_class`] works out of the box.
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
        self.shared_db.salsa.write().set_resolver(Some(resolver));
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
        self.shared_db
            .salsa
            .write()
            .set_resolver(Some(wrapped.clone()));
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

    /// Load every PHP built-in stub plus any configured user stubs.
    ///
    /// **Deprecated**: prefer [`Self::ensure_all_stubs_loaded`] (explicit
    /// "comprehensive") or [`Self::ensure_essential_stubs_loaded`] (fast
    /// cold-start with auto-discovery on demand).
    #[doc(hidden)]
    pub fn ensure_stubs_loaded(&self) {
        self.ensure_all_stubs_loaded();
    }

    /// Load only the curated set of essential stubs (Core, standard, SPL,
    /// date) plus any configured user stubs. About 25 of 120 stub files;
    /// covers types and functions used by virtually all PHP code.
    ///
    /// Other extension stubs (Reflection, gd, openssl, …) can be brought in
    /// on demand via [`Self::ensure_stubs_for_symbol`] when user code
    /// references them. Idempotent — already-loaded stubs are skipped.
    pub fn ensure_essential_stubs_loaded(&self) {
        self.shared_db
            .ingest_stub_paths(crate::stubs::ESSENTIAL_STUB_PATHS, self.php_version);
        self.ensure_user_stubs_loaded();
    }

    /// Load every embedded PHP stub plus any configured user stubs.
    /// Use for batch tools (CLI, full project analysis) where comprehensive
    /// symbol coverage matters more than cold-start latency.
    pub fn ensure_all_stubs_loaded(&self) {
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Number of distinct embedded stubs currently ingested into the session.
    /// Useful for diagnostics and bench reporting.
    pub fn loaded_stub_count(&self) -> usize {
        self.shared_db.loaded_stubs.lock().len()
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
    /// batch tool called [`Self::ensure_all_stubs_loaded`]), the source scan
    /// is skipped entirely.
    pub fn ensure_stubs_for_source(&self, source: &str) {
        // Cheap check first: skip the scan entirely when we already know we
        // have everything. Avoids a ~50-500µs source walk on every analyze
        // call in batch / warm-session scenarios.
        {
            let loaded = self.shared_db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths(source);
        if paths.is_empty() {
            return;
        }
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
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
    pub fn ensure_stubs_for_ast(&self, program: &php_ast::ast::Program<'_, '_>) {
        {
            let loaded = self.shared_db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths_from_ast(program);
        if paths.is_empty() {
            return;
        }
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
    }

    /// Scan a parsed AST for class references and lazy-load any that are
    /// PSR-4-resolvable but not yet registered as `SourceFile` inputs. After
    /// this call, `find_class_like(fqcn)` can pull-resolve the referenced
    /// classes without needing a retry loop.
    ///
    /// The current implementation reuses [`crate::diagnostics::collect_referenced_class_fqcns`]
    /// already used by the diagnostics pass. Missing classes are passed
    /// through [`Self::lazy_load_class_transitive`] so their inheritance
    /// chain is also primed (Pass-2 reads parents/interfaces while
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
    pub fn prepare_ast_for_analysis(&self, program: &php_ast::ast::Program<'_, '_>, file: &str) {
        self.ensure_stubs_for_ast(program);
        self.preload_psr4_classes_for_ast(program, file);
    }

    pub fn preload_psr4_classes_for_ast(
        &self,
        program: &php_ast::ast::Program<'_, '_>,
        file: &str,
    ) {
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
            let _ = self.lazy_load_class(&fqcn);
        }
    }

    fn ensure_user_stubs_loaded(&self) {
        self.shared_db
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
        self.shared_db.snapshot_db()
    }

    /// Commit a batch of reference locations from a db snapshot into the
    /// session's shared maps.  Called by [`crate::FileAnalyzer`] and
    /// [`crate::BatchFileAnalyzer`] after parallel Pass 2 to flush the pending
    /// buffers that accumulate in worker db clones.
    pub(crate) fn commit_ref_locs_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        let guard = self.shared_db.salsa.read();
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

    /// Pass 1 ingestion. Updates the file's source text in the salsa db,
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
        self.ensure_stubs_loaded();

        // Snapshot symbols defined before clearing — O(symbols_in_file) with forward index.
        let old_symbols: HashSet<Arc<str>> = {
            let guard = self.shared_db.salsa.read();
            guard.file_defined_symbols(file.as_ref())
        };

        {
            let mut guard = self.shared_db.salsa.write();
            guard.remove_file_definitions(file.as_ref());
        }
        let _file_defs =
            self.shared_db
                .collect_and_ingest_file(file.clone(), source.as_ref(), self.php_version);

        // Snapshot symbols after ingesting — O(symbols_in_file).
        let new_symbols: HashSet<Arc<str>> = {
            let guard = self.shared_db.salsa.read();
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
    }

    /// Register `source` as the text of `file` in the salsa input layer **without**
    /// parsing or running Pass 1.
    ///
    /// This is the LSP-friendly bulk-population entry point: after a workspace
    /// scan, callers can feed every discovered file's text to the session
    /// cheaply (an Arc clone plus a HashMap insert per file). Symbol resolution
    /// then happens on demand via [`Self::lookup_class_or_load`], which reads
    /// the file from disk through the configured [`crate::ClassResolver`] and
    /// runs Pass 1 lazily when a class FQCN actually needs to resolve.
    ///
    /// Contrast with [`Self::ingest_file`], which eagerly parses, runs Pass 1,
    /// and populates the symbol index. Use `ingest_file` for files the user is
    /// actively editing (where in-memory text diverges from disk); use
    /// `set_file_text` for files known only through the workspace scan.
    ///
    /// Clears the negative cache: a previously-unresolvable FQCN may now
    /// resolve if its defining file is among the newly-registered set.
    pub fn set_file_text(&self, file: Arc<str>, source: Arc<str>) {
        {
            let mut guard = self.shared_db.salsa.write();
            guard.upsert_source_file(file.clone(), source);
        }
        self.evict_unresolvable_for_file(&file);
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
    /// parsing has happened yet — Pass 1 runs per file on the first
    /// `lookup_class_or_load` that needs to consult it.
    pub fn set_workspace_files<I>(&self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        let registered_paths: Vec<Arc<str>> = {
            let mut guard = self.shared_db.salsa.write();
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
            let mut guard = self.shared_db.salsa.write();
            guard.remove_file_definitions(file);
            guard.remove_source_file(file);
        }
        // Remove this file's outgoing deps from the in-memory reverse dep map.
        self.update_in_memory_reverse_deps(file, &HashSet::new());
        // Clear stale symbol tracking for this file — it's fully gone.
        self.stale_defined_symbols.write().remove(file);
        if let Some(cache) = &self.cache {
            cache.update_reverse_deps_for_file(file, &HashSet::new());
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
        let guard = self.shared_db.salsa.read();
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
    /// mutate the salsa input set. Use [`Self::definition_of_loaded`] for a
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
                let _ = self.lazy_load_class(fqcn.as_ref());
            }
            crate::Symbol::Function(fqn) => {
                let _ = self.lazy_load_class(fqn.as_ref());
            }
            crate::Symbol::Method { class, .. }
            | crate::Symbol::Property { class, .. }
            | crate::Symbol::ClassConstant { class, .. } => {
                let _ = self.lazy_load_class(class.as_ref());
            }
            _ => {}
        }
        self.definition_of_loaded(symbol)
    }

    /// Pure variant of [`Self::definition_of`]. Never invokes the
    /// [`crate::SourceProvider`] and never mutates salsa inputs; resolves
    /// only against state already loaded by `set_file_text` / `ingest_file`.
    /// Returns `Err(NotFound)` when the symbol isn't in the loaded set, even
    /// if a resolver could in principle map it.
    pub fn definition_of_loaded(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<mir_codebase::storage::Location, crate::SymbolLookupError> {
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Class(fqcn) => {
                let here = crate::db::Fqcn::new(&db, fqcn.clone());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                class
                    .location()
                    .cloned()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Function(fqn) => {
                let here = crate::db::Fqcn::new(&db, fqn.clone());
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
    /// dependencies. Use [`Self::hover_loaded`] for a pure variant.
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
                self.lookup_class_or_load(fqcn.as_ref());
            }
            crate::Symbol::Method { class, .. }
            | crate::Symbol::Property { class, .. }
            | crate::Symbol::ClassConstant { class, .. } => {
                self.lookup_class_or_load_transitive(class.as_ref());
            }
            _ => {}
        }
        self.hover_loaded(symbol)
    }

    /// Pure variant of [`Self::hover`]. Never invokes the
    /// [`crate::SourceProvider`]; consults only the already-loaded db.
    pub fn hover_loaded(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Union};
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Function(fqn) => {
                let here = crate::db::Fqcn::new(&db, fqn.clone());
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
                let here = crate::db::Fqcn::new(&db, class.clone());
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
                let here = crate::db::Fqcn::new(&db, fqcn.clone());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = Union::single(Atomic::TNamedObject {
                    fqcn: fqcn.clone(),
                    type_params: Vec::new(),
                });
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: class.location().cloned(),
                })
            }
            crate::Symbol::Property { class, name } => {
                let here = crate::db::Fqcn::new(&db, class.clone());
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
                let here = crate::db::Fqcn::new(&db, class.clone());
                let (_, c) = crate::db::find_class_constant_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: c.ty.clone(),
                    docstring: None,
                    definition: c.location.clone(),
                })
            }
            crate::Symbol::GlobalConstant(fqn) => {
                let here = crate::db::Fqcn::new(&db, fqn.clone());
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
    pub fn class_issues_for(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
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
            |methods: &indexmap::IndexMap<Arc<str>, Arc<mir_codebase::storage::MethodStorage>>,
             props: Option<
                &indexmap::IndexMap<Arc<str>, mir_codebase::storage::PropertyStorage>,
            >,
             consts: &indexmap::IndexMap<Arc<str>, mir_codebase::storage::ConstantStorage>,
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

    /// Try to resolve `fqcn` via PSR-4 and ingest the mapped file, returning
    /// a detailed outcome distinguishing "already there" from "freshly loaded".
    pub fn lazy_load_class_with_outcome(&self, fqcn: &str) -> crate::LazyLoadOutcome {
        if self.contains_class(fqcn) {
            return crate::LazyLoadOutcome::AlreadyLoaded;
        }
        if self.lazy_load_class(fqcn) {
            crate::LazyLoadOutcome::Loaded
        } else {
            crate::LazyLoadOutcome::NotResolvable
        }
    }

    /// Try to resolve `fqcn` via the configured [`crate::ClassResolver`] and
    /// ingest the mapped file.
    ///
    /// This is the LSP-friendly lazy-load entry point: the analyzer never
    /// touches `vendor/` on its own, but consumers can ask it to resolve
    /// individual symbols on demand. Designed to be called when a diagnostic
    /// would otherwise report `UndefinedClass`.
    ///
    /// Returns `true` if either the class is already known or a matching
    /// file was found and successfully ingested. Returns `false` if:
    /// - No resolver is configured (neither `with_psr4` nor `with_class_resolver` called),
    /// - The resolver can't map `fqcn` to a file,
    /// - The file can't be read, or
    /// - The file parsed but did not define `fqcn`.
    pub fn lazy_load_class(&self, fqcn: &str) -> bool {
        use crate::metrics::{record_lazy_load_failure, LazyLoadFailure};
        if self.contains_class(fqcn) {
            return true;
        }
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
    pub fn lazy_load_class_transitive(&self, fqcn: &str, max_depth: usize) -> usize {
        if self.resolver.is_none() {
            return 0;
        }
        let mut loaded = 0;
        let mut frontier: Vec<String> = vec![fqcn.to_string()];
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

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
                let resolved = self.lazy_load_class(&name);
                if resolved && !was_present {
                    loaded += 1;
                    // Walk the new class's parent / interfaces / traits via pull.
                    let db = self.snapshot_db();
                    let here = crate::db::Fqcn::new(&db, Arc::<str>::from(name.as_str()));
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

    /// Resolve `fqcn` to a source file, lazy-loading via the configured
    /// [`crate::ClassResolver`] if it isn't already registered.
    ///
    /// This is the recommended entry point for callers (LSP, Pass 2 diagnostic
    /// emission) that want "does this class exist anywhere in the workspace?"
    /// semantics without enumerating dependencies upfront. The fast path is a
    /// single DashMap lookup; the slow path runs only on miss and is itself
    /// negative-cached so repeated lookups for genuinely-missing names don't
    /// re-hit the resolver. The negative cache is invalidated on any
    /// [`Self::ingest_file`] / [`Self::invalidate_file`] call.
    ///
    /// Returns `None` if the class is not registered AND the resolver can't
    /// map `fqcn` to a readable file that defines it.
    /// Returns `true` if `fqcn` is resolvable (already loaded or lazily
    /// loaded on this call). Returns `false` if resolution fails.
    pub fn lookup_class_or_load(&self, fqcn: &str) -> bool {
        if self.contains_class(fqcn) {
            return true;
        }
        if self.unresolvable_fqcns.read().contains_key(fqcn) {
            return false;
        }
        if !self.lazy_load_class(fqcn) {
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
            return false;
        }
        true
    }

    /// Like [`Self::lookup_class_or_load`] but additionally walks the
    /// inheritance chain (parent + interfaces + traits) so subsequent
    /// member-lookup queries on the returned node have the full chain loaded.
    pub fn lookup_class_or_load_transitive(&self, fqcn: &str) -> bool {
        if !self.lookup_class_or_load(fqcn) {
            return false;
        }
        // 10 mirrors the default depth used by analyze_dependents_of.
        self.lazy_load_class_transitive(fqcn, 10);
        true
    }

    /// Returns `true` if `fqn` is a known global function. No resolver-driven
    /// slow path: functions are not name-mapped to files by PSR-4.
    pub fn lookup_function_or_load(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::new(&db, std::sync::Arc::<str>::from(fqn));
        crate::db::find_function(&db, here).is_some()
    }

    /// Retrieve the source text the session has registered for `file`, if
    /// any. Returns `None` when the file has never been ingested. Used by
    /// the parallel re-analysis path to re-feed dependents to Pass 2 without
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
    /// Does not run inference sweeps. For full-fidelity cross-file inferred
    /// return types, follow up with [`Self::run_inference_sweep`] over the
    /// affected file set.
    pub fn analyze_dependents_of(&self, file: &str) -> Vec<(Arc<str>, crate::FileAnalysis)> {
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
        // session's canonical db (none happen here since we only run Pass 2).
        with_source
            .into_par_iter()
            .map(|(file, source)| {
                let arena = crate::arena::create_parse_arena(source.len());
                let parsed = php_rs_parser::parse(&arena, source.as_ref());
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
            let here = crate::db::Fqcn::new(&db, Arc::<str>::from(fqcn.as_str()));
            if crate::db::find_class_like(&db, here).is_some() {
                continue;
            }
            if let Some(resolver) = &self.resolver {
                if resolver.resolve(fqcn).is_some() {
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
            loaded += self.lazy_load_class_transitive(&fqcn, 2);
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
                let here = crate::db::Fqcn::new(&db, fqcn.clone());
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
                let here = crate::db::Fqcn::new(&db, fqn.clone());
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
        let mut map = self.reverse_dep_map.write();
        for dependents in map.values_mut() {
            dependents.remove(file);
        }
        map.retain(|_, dependents| !dependents.is_empty());
        for target in new_targets {
            if target != file {
                map.entry(target.clone())
                    .or_default()
                    .insert(file.to_string());
            }
        }
    }

    /// BFS transitive dependents of `file` using the in-memory reverse dep map.
    ///
    /// O(D) where D is the number of transitive dependents — faster than
    /// [`Self::dependency_graph().transitive_dependents()`] which rebuilds the
    /// full graph on every call. Only covers Pass 1 structural dependencies
    /// (imports, class hierarchy, type hints); does not include bare FQN body
    /// references recorded during Pass 2. For full fidelity, use
    /// `dependency_graph().transitive_dependents()` after Pass 2 is complete.
    pub fn structural_dependents_of(&self, file: &str) -> Vec<String> {
        let map = self.reverse_dep_map.read();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue = vec![file.to_string()];
        let mut result = Vec::new();
        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(deps) = map.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push(dep.clone());
                        result.push(dep.clone());
                    }
                }
            }
        }
        result
    }

    /// Cross-file inference sweep. For each `(file, source)` pair, calls the
    /// Salsa-tracked `infer_file_return_types` query in parallel, then commits
    /// the collected inferred return types to INPUT fields.
    ///
    /// Files must already be ingested via [`Self::ingest_file`] before calling
    /// this method. Subsequent [`FileAnalyzer::analyze`] calls read the committed
    /// INPUT fields via O(1) lookups with no lock contention.
    pub fn run_inference_sweep(&self, files: &[(Arc<str>, Arc<str>)]) {
        use rayon::prelude::*;
        let db_priming = self.snapshot_db();
        let inferred_results: Vec<crate::db::InferredFileTypes> = files
            .par_iter()
            .map_with(db_priming, |db, (path, _src)| {
                if let Some(sf) = db.lookup_source_file(path) {
                    crate::db::infer_file_return_types(db, sf)
                } else {
                    crate::db::InferredFileTypes::empty()
                }
            })
            .collect();
        let mut functions = Vec::new();
        let mut methods = Vec::new();
        for result in inferred_results {
            for (fqn, ty) in result.functions.iter() {
                functions.push((fqn.clone(), (**ty).clone()));
            }
            for ((fqcn, name), ty) in result.methods.iter() {
                methods.push((fqcn.clone(), name.clone(), (**ty).clone()));
            }
        }
        let mut guard = self.shared_db.salsa.write();
        guard.commit_inferred_return_types(functions, methods);
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

        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for file in &all_files {
            // O(degree(file)) — forward index lookup, no full-table scan.
            let symbol_keys = db.file_referenced_symbols(file);
            let mut file_deps: HashSet<String> = HashSet::new();
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

        // Merge Pass 1 structural deps from the incremental reverse_dep_map.
        // dependency_graph() above only captures Pass 2 bare-FQN references;
        // the reverse_dep_map covers imports, class hierarchy (extends/implements/use),
        // and type-hint-only references that never appear in file_referenced_symbols.
        // Together they give a complete picture without requiring Pass 2 on every file.
        {
            let rev = self.reverse_dep_map.read();
            for (target, dep_set) in rev.iter() {
                for dep in dep_set {
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
    let mut targets: HashSet<String> = HashSet::new();

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
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(fqcn.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let imports = db.file_imports(file);
    for fqcn in imports.values() {
        add_target(fqcn);
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

    // Also track bare-FQN references recorded during Pass 2 (new \Foo(), \Foo::method(),
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
fn collect_class_refs_from_ast(program: &php_ast::ast::Program<'_, '_>) -> Vec<String> {
    use php_ast::ast::{BinaryOp, ExprKind, TypeHintKind};
    use php_ast::visitor::{walk_catch_clause, walk_expr, walk_program, walk_type_hint, Visitor};
    use std::ops::ControlFlow;

    struct V {
        names: std::collections::HashSet<String>,
    }
    impl<'arena, 'src> Visitor<'arena, 'src> for V {
        fn visit_expr(&mut self, expr: &php_ast::ast::Expr<'arena, 'src>) -> ControlFlow<()> {
            match &expr.kind {
                ExprKind::New(n) => {
                    if let ExprKind::Identifier(name) = &n.class.kind {
                        self.names.insert(name.to_string());
                    }
                }
                ExprKind::StaticMethodCall(c) => {
                    if let ExprKind::Identifier(name) = &c.class.kind {
                        self.names.insert(name.to_string());
                    }
                }
                ExprKind::StaticPropertyAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.to_string());
                    }
                }
                ExprKind::ClassConstAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.to_string());
                    }
                }
                ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
                    if let ExprKind::Identifier(name) = &b.right.kind {
                        self.names.insert(name.to_string());
                    }
                }
                _ => {}
            }
            walk_expr(self, expr)
        }

        fn visit_type_hint(
            &mut self,
            hint: &php_ast::ast::TypeHint<'arena, 'src>,
        ) -> ControlFlow<()> {
            if let TypeHintKind::Named(name) = &hint.kind {
                let s = name.to_string_repr().into_owned();
                if !s.is_empty() {
                    self.names.insert(s);
                }
            }
            walk_type_hint(self, hint)
        }

        fn visit_catch_clause(
            &mut self,
            catch: &php_ast::ast::CatchClause<'arena, 'src>,
        ) -> ControlFlow<()> {
            for ty in catch.types.iter() {
                self.names.insert(ty.to_string_repr().into_owned());
            }
            walk_catch_clause(self, catch)
        }
    }
    let mut v = V {
        names: std::collections::HashSet::new(),
    };
    let _ = walk_program(&mut v, program);
    v.names.into_iter().collect()
}
