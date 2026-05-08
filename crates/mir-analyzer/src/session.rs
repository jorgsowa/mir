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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

use rayon::prelude::*;
use salsa::Setter as _;

use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{collect_file_definitions, FileDefinitions, MirDatabase, MirDb, SourceFile};
use crate::php_version::PhpVersion;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
///
/// Cheap to clone the inner db for parallel reads; writes funnel through
/// [`Self::ingest_file`], [`Self::invalidate_file`], and the crate-internal
/// [`Self::with_db_mut`].
pub struct AnalysisSession {
    salsa: Mutex<(MirDb, HashMap<Arc<str>, SourceFile>)>,
    cache: Option<Arc<AnalysisCache>>,
    psr4: Option<Arc<Psr4Map>>,
    /// Set of stub virtual paths that have already been ingested. Replaces an
    /// older `AtomicBool stubs_loaded` flag — tracking individual paths lets
    /// us lazy-load extension stubs on demand without re-ingesting essentials.
    loaded_stubs: Mutex<HashSet<&'static str>>,
    /// True once user stubs (configured via [`Self::with_user_stubs`]) have
    /// been ingested. They are loaded together with the essential set on the
    /// first call to a stubs-loading method.
    user_stubs_loaded: AtomicBool,
    php_version: PhpVersion,
    user_stub_files: Vec<PathBuf>,
    user_stub_dirs: Vec<PathBuf>,
}

impl AnalysisSession {
    /// Create a session targeting the given PHP language version.
    pub fn new(php_version: PhpVersion) -> Self {
        Self {
            salsa: Mutex::new((MirDb::default(), HashMap::new())),
            cache: None,
            psr4: None,
            loaded_stubs: Mutex::new(HashSet::new()),
            user_stubs_loaded: AtomicBool::new(false),
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
        }
    }

    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_psr4(mut self, map: Arc<Psr4Map>) -> Self {
        self.psr4 = Some(map);
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
    /// Idempotent. Equivalent to the legacy "load everything" behavior; use
    /// [`Self::ensure_essential_stubs_loaded`] in incremental scenarios where
    /// cold-start latency matters more than comprehensive stub coverage.
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
        self.ingest_stub_paths(crate::stubs::ESSENTIAL_STUB_PATHS);
        self.ensure_user_stubs_loaded();
    }

    /// Load every embedded PHP stub plus any configured user stubs.
    /// Use for batch tools (CLI, full project analysis) where comprehensive
    /// symbol coverage matters more than cold-start latency.
    pub fn ensure_all_stubs_loaded(&self) {
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.ingest_stub_paths(&paths);
        self.ensure_user_stubs_loaded();
    }

    /// Ensure the embedded stub that defines `name` (a function) is ingested.
    /// Returns `true` when a matching stub exists (whether or not it was
    /// already loaded), `false` when `name` isn't a known PHP built-in.
    pub fn ensure_stub_for_function(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_function(name) {
            Some(path) => {
                self.ingest_stub_paths(&[path]);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `fqcn` (a class / interface /
    /// trait / enum) is ingested. Case-insensitive lookup with optional
    /// leading backslash.
    pub fn ensure_stub_for_class(&self, fqcn: &str) -> bool {
        match crate::stubs::stub_path_for_class(fqcn) {
            Some(path) => {
                self.ingest_stub_paths(&[path]);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `name` (a constant) is ingested.
    pub fn ensure_stub_for_constant(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_constant(name) {
            Some(path) => {
                self.ingest_stub_paths(&[path]);
                true
            }
            None => false,
        }
    }

    /// Number of distinct embedded stubs currently ingested into the session.
    /// Useful for diagnostics and bench reporting.
    pub fn loaded_stub_count(&self) -> usize {
        self.loaded_stubs.lock().len()
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
            let loaded = self.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths(source);
        if paths.is_empty() {
            return;
        }
        self.ingest_stub_paths(&paths);
    }

    /// Internal: parse + ingest each path in `paths` that hasn't already been
    /// ingested. Holds the salsa write lock per file (brief), and the
    /// `loaded_stubs` set lock briefly to record paths.
    fn ingest_stub_paths(&self, paths: &[&'static str]) {
        // Pick out the not-yet-loaded paths first to avoid redundant parsing.
        let needed: Vec<&'static str> = {
            let loaded = self.loaded_stubs.lock();
            paths
                .iter()
                .copied()
                .filter(|p| !loaded.contains(p))
                .collect()
        };
        if needed.is_empty() {
            return;
        }

        let php_version = self.php_version;
        // Parse in parallel; ingest serially under the salsa write lock.
        let slices: Vec<(&'static str, mir_codebase::storage::StubSlice)> = needed
            .par_iter()
            .filter_map(|&path| {
                crate::stubs::stub_content_for_path(path).map(|content| {
                    let slice =
                        crate::stubs::stub_slice_from_source(path, content, Some(php_version));
                    (path, slice)
                })
            })
            .collect();

        let mut guard = self.salsa.lock();
        let mut loaded = self.loaded_stubs.lock();
        for (path, slice) in slices {
            if loaded.insert(path) {
                guard.0.ingest_stub_slice(&slice);
            }
        }
    }

    fn ensure_user_stubs_loaded(&self) {
        if self.user_stub_files.is_empty() && self.user_stub_dirs.is_empty() {
            return;
        }
        let was_loaded = self.user_stubs_loaded.load(Ordering::Relaxed);
        if was_loaded {
            return;
        }
        let slices = crate::stubs::user_stub_slices(&self.user_stub_files, &self.user_stub_dirs);
        let mut salsa = self.salsa.lock();
        for slice in slices {
            salsa.0.ingest_stub_slice(&slice);
        }
        self.user_stubs_loaded.store(true, Ordering::Relaxed);
    }

    /// Cheap clone of the salsa db for a read-only query. The lock is held
    /// only for the duration of the clone, so concurrent readers never
    /// serialize on each other or on writes for longer than the clone itself.
    pub fn snapshot_db(&self) -> MirDb {
        let guard = self.salsa.lock();
        guard.0.clone()
    }

    /// Run a closure with read access to a database snapshot. The snapshot is
    /// taken under a brief lock, then the closure runs without holding it.
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
    pub fn ingest_file(&self, file: Arc<str>, source: Arc<str>) -> FileDefinitions {
        self.ensure_stubs_loaded();
        let file_defs = {
            let mut guard = self.salsa.lock();
            let (ref mut db, ref mut files) = *guard;
            let salsa_file = match files.get(&file) {
                Some(&sf) => {
                    // Re-ingestion: drop old definitions + reference locations
                    // before collecting fresh ones. Mirrors what
                    // ProjectAnalyzer::re_analyze_file does.
                    db.remove_file_definitions(file.as_ref());
                    if sf.text(db).as_ref() != source.as_ref() {
                        sf.set_text(db).to(source.clone());
                    }
                    sf
                }
                None => {
                    let sf = SourceFile::new(db, file.clone(), source.clone());
                    files.insert(file.clone(), sf);
                    sf
                }
            };
            collect_file_definitions(db, salsa_file)
        };
        {
            let mut guard = self.salsa.lock();
            guard.0.ingest_stub_slice(&file_defs.slice);
        }
        self.update_reverse_deps_for(&file);
        file_defs
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
            let mut guard = self.salsa.lock();
            let (ref mut db, ref mut files) = *guard;
            db.remove_file_definitions(file);
            files.remove(file);
        }
        if let Some(cache) = &self.cache {
            cache.update_reverse_deps_for_file(file, &HashSet::new());
            cache.evict_with_dependents(&[file.to_string()]);
        }
    }

    /// Number of files currently tracked in this session's salsa input set.
    /// Stable across reads; useful for diagnostics and memory bounds checks.
    pub fn tracked_file_count(&self) -> usize {
        let guard = self.salsa.lock();
        guard.1.len()
    }

    // -----------------------------------------------------------------------
    // Read-only codebase queries
    //
    // All take a brief lock to clone the db, then run the lookup against the
    // owned snapshot — concurrent edits proceed without blocking.
    // -----------------------------------------------------------------------

    /// Resolve `symbol` (a class FQCN or function FQN) to its declaration
    /// location. Powers go-to-definition for top-level symbols. Returns
    /// `None` if the symbol isn't known to the codebase or has no recorded
    /// source span (e.g. some stub-only declarations).
    pub fn definition_of(&self, symbol: &str) -> Option<mir_codebase::storage::Location> {
        let db = self.snapshot_db();
        db.lookup_class_node(symbol)
            .filter(|n| n.active(&db))
            .and_then(|n| n.location(&db))
            .or_else(|| {
                db.lookup_function_node(symbol)
                    .filter(|n| n.active(&db))
                    .and_then(|n| n.location(&db))
            })
    }

    /// Resolve a class member (method / property / class constant / enum case)
    /// to its declaration location, walking the inheritance chain.
    pub fn member_definition(
        &self,
        fqcn: &str,
        member_name: &str,
    ) -> Option<mir_codebase::storage::Location> {
        let db = self.snapshot_db();
        crate::db::member_location_via_db(&db, fqcn, member_name)
    }

    /// Every recorded reference to `symbol` (as `(file, line, col_start,
    /// col_end)`). Use [`crate::symbol::ResolvedSymbol::codebase_key`] to
    /// build the lookup key from a `ResolvedSymbol` returned by
    /// [`crate::FileAnalysis::symbol_at`].
    pub fn references_to(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// All declarations defined in `file` (classes, interfaces, traits, enums,
    /// functions, constants). Powers outline / document-symbols views and any
    /// other consumer that needs the file's top-level symbol set. Returns an
    /// empty Vec if `file` hasn't been ingested.
    pub fn document_symbols(&self, file: &str) -> Vec<crate::symbol::DocumentSymbol> {
        use crate::symbol::{DocumentSymbol, DocumentSymbolKind};

        let db = self.snapshot_db();
        let mut out = Vec::new();
        for symbol in db.symbols_defined_in_file(file) {
            // Try class side first — covers Class / Interface / Trait / Enum.
            if let Some(class_node) = db.lookup_class_node(symbol.as_ref()) {
                if !class_node.active(&db) {
                    continue;
                }
                let kind = crate::db::class_kind_via_db(&db, symbol.as_ref())
                    .map(|k| {
                        if k.is_interface {
                            DocumentSymbolKind::Interface
                        } else if k.is_trait {
                            DocumentSymbolKind::Trait
                        } else if k.is_enum {
                            DocumentSymbolKind::Enum
                        } else {
                            DocumentSymbolKind::Class
                        }
                    })
                    .unwrap_or(DocumentSymbolKind::Class);
                out.push(DocumentSymbol {
                    name: symbol.clone(),
                    kind,
                    location: class_node.location(&db),
                });
                continue;
            }
            if let Some(fn_node) = db.lookup_function_node(symbol.as_ref()) {
                if !fn_node.active(&db) {
                    continue;
                }
                out.push(DocumentSymbol {
                    name: symbol.clone(),
                    kind: DocumentSymbolKind::Function,
                    location: fn_node.location(&db),
                });
                continue;
            }
            // Constants and other top-level declarations: emit with no
            // location info; consumers can still surface them in an outline.
            out.push(DocumentSymbol {
                name: symbol,
                kind: DocumentSymbolKind::Constant,
                location: None,
            });
        }
        out
    }

    /// Compute `file`'s outgoing dependency edges and update the cache's
    /// reverse-dep graph in place. No-op if no cache is configured.
    fn update_reverse_deps_for(&self, file: &str) {
        let Some(cache) = self.cache.as_deref() else {
            return;
        };
        let db = self.snapshot_db();
        let targets = file_outgoing_dependencies(&db, file);
        cache.update_reverse_deps_for_file(file, &targets);
    }

    /// Cross-file inference sweep. For each `(file, source)` pair, runs the
    /// Pass 2 inference-only mode on a cloned db (parallel via rayon), then
    /// commits the collected inferred return types to the canonical db.
    ///
    /// Call this on idle / save / explicit user request, **not** on every
    /// keystroke — [`crate::FileAnalyzer::analyze`] deliberately skips
    /// inference sweep on the hot path. Files whose source contains parse
    /// errors are silently skipped.
    pub fn run_inference_sweep(&self, files: &[(Arc<str>, Arc<str>)]) {
        self.ensure_stubs_loaded();

        // The priming db lives only inside `gather_inferred_types`. After it
        // returns, all rayon-clone references to the salsa storage are dropped
        // — required so that the subsequent `commit_inferred_return_types`
        // call (which calls salsa's `cancel_others`) doesn't deadlock waiting
        // for outstanding db references.
        let (functions, methods) =
            gather_inferred_types(self.snapshot_db(), files, self.php_version);

        let mut guard = self.salsa.lock();
        guard.0.commit_inferred_return_types(functions, methods);
    }
}

/// Drive Pass 2 inference-only mode in parallel across `files`, accumulating
/// inferred function and method return types. The `db_priming` MirDb is
/// consumed (cloned per spawned task and dropped on return), so the caller's
/// canonical db can subsequently take exclusive access without deadlock.
///
/// Crate-internal so [`crate::project::ProjectAnalyzer`] can use the same
/// deadlock-safe helper for its lazy-load reanalysis sweep.
#[allow(clippy::type_complexity)]
pub(crate) fn gather_inferred_types(
    db_priming: MirDb,
    files: &[(Arc<str>, Arc<str>)],
    php_version: PhpVersion,
) -> (
    Vec<(Arc<str>, mir_types::Union)>,
    Vec<(Arc<str>, Arc<str>, mir_types::Union)>,
) {
    use crate::pass2::Pass2Driver;
    use mir_types::Union;

    type Functions = Vec<(Arc<str>, Union)>;
    type Methods = Vec<(Arc<str>, Arc<str>, Union)>;
    let functions: Arc<Mutex<Functions>> = Arc::new(Mutex::new(Vec::new()));
    let methods: Arc<Mutex<Methods>> = Arc::new(Mutex::new(Vec::new()));

    rayon::in_place_scope(|s| {
        for (file, source) in files {
            let db = db_priming.clone();
            let functions = Arc::clone(&functions);
            let methods = Arc::clone(&methods);
            let file = file.clone();
            let source = source.clone();

            s.spawn(move |_| {
                let arena = bumpalo::Bump::new();
                let parsed = php_rs_parser::parse(&arena, source.as_ref());
                if !parsed.errors.is_empty() {
                    return;
                }
                let driver = Pass2Driver::new_inference_only(&db as &dyn MirDatabase, php_version);
                driver.analyze_bodies(&parsed.program, file, source.as_ref(), &parsed.source_map);
                let inferred = driver.take_inferred_types();
                {
                    let mut f = functions.lock();
                    f.extend(inferred.functions);
                }
                {
                    let mut m = methods.lock();
                    m.extend(inferred.methods);
                }
            });
        }
    });

    let functions = Arc::try_unwrap(functions)
        .map(|m| m.into_inner())
        .unwrap_or_else(|arc| arc.lock().clone());
    let methods = Arc::try_unwrap(methods)
        .map(|m| m.into_inner())
        .unwrap_or_else(|arc| arc.lock().clone());

    (functions, methods)
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

    let imports = db.file_imports(file);
    for fqcn in imports.values() {
        add_target(fqcn);
    }

    for fqcn in db.symbols_defined_in_file(file) {
        let Some(node) = db.lookup_class_node(fqcn.as_ref()) else {
            continue;
        };
        if let Some(parent) = node.parent(db) {
            add_target(parent.as_ref());
        }
        for iface in node.interfaces(db).iter() {
            add_target(iface.as_ref());
        }
        for tr in node.traits(db).iter() {
            add_target(tr.as_ref());
        }
    }

    targets
}
