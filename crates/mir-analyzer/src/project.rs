/// Project-level orchestration: file discovery, pass 1, pass 2.
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use std::collections::{HashMap, HashSet};

use crate::cache::{hash_content, AnalysisCache};
use crate::db::{
    collect_file_definitions, FileDefinitions, MirDatabase, MirDb, RefLoc, SourceFile,
};
use crate::pass2::{InferredTypes, Pass2Driver};
use crate::php_version::PhpVersion;
use crate::shared_db::SharedDb;
use crate::stub_cache::{hash_source, prepare_for_ingest};
use mir_issues::Issue;

pub(crate) use crate::pass2::merge_return_types;

/// Issue kinds emitted by [`crate::dead_code::DeadCodeAnalyzer`].
///
/// The dead-code pass is just an error group — these names participate in
/// `suppressed_issue_kinds` like any other [`IssueKind`]. If every kind
/// listed here is suppressed, the dead-code pass is skipped entirely (it
/// has nothing to contribute).
pub fn dead_code_issue_kinds() -> &'static [&'static str] {
    &["UnusedMethod", "UnusedProperty", "UnusedFunction"]
}

/// Batch-oriented analyzer: file discovery, parsing, and analysis.
///
/// ProjectAnalyzer is the primary entry point for analyzing a project as a whole.
/// It orchestrates parallel file discovery and parsing, using the same core
/// analysis engine as [`AnalysisSession`] (salsa database and Pass 2 driver).
///
/// **Unified Design:** ProjectAnalyzer and `AnalysisSession` now share the same
/// database management via [`SharedDb`]. ProjectAnalyzer is the batch API
/// (all files at once), while `AnalysisSession` is the incremental API (file-by-file).
/// Both use `Pass2Driver`, the same definition collection logic, and identical
/// database operations, eliminating code duplication.
///
/// [`AnalysisSession`]: crate::session::AnalysisSession
pub struct ProjectAnalyzer {
    /// Shared database management (salsa, file registry, stub tracking).
    /// Extracted to allow code sharing with AnalysisSession.
    shared_db: Arc<SharedDb>,
    /// Optional cache — when `Some`, Pass 2 results are read/written per file.
    cache: Option<AnalysisCache>,
    /// Called once after each file completes Pass 2 (used for progress reporting).
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    /// PSR-4 autoloader mapping from composer.json, if available.
    pub psr4: Option<Arc<crate::composer::Psr4Map>>,
    /// Names of `IssueKind` variants to drop from the final result, e.g.
    /// `["MissingThrowsDocblock", "UnusedMethod"]`. Applied as a final
    /// post-filter on every `analyze()` return path, so analyzer internals
    /// don't need to know which diagnostics the consumer cares about.
    ///
    /// Defaults to an empty set — nothing is suppressed unless the
    /// consumer (CLI, test fixture, programmatic caller) adds names. The
    /// dead-code pass is skipped automatically when every
    /// [`dead_code_issue_kinds`] entry is in this set.
    pub suppressed_issue_kinds: std::collections::HashSet<String>,
    /// Target PHP language version. `None` means "not configured"; resolved to
    /// `PhpVersion::LATEST` when passed down to `StatementsAnalyzer`.
    pub php_version: Option<PhpVersion>,
    /// Additional stub files to parse before analysis (absolute paths).
    pub stub_files: Vec<PathBuf>,
    /// Additional stub directories to walk and parse before analysis (absolute paths).
    pub stub_dirs: Vec<PathBuf>,
}

struct ParsedProjectFile {
    file: Arc<str>,
    source: Arc<str>,
    parsed: ManuallyDrop<php_rs_parser::ParseResult<'static, 'static>>,
    arena: ManuallyDrop<Box<bumpalo::Bump>>,
}

impl ParsedProjectFile {
    fn new(file: Arc<str>, source: Arc<str>) -> Self {
        let arena = Box::new(crate::arena::create_parse_arena(source.len()));
        let parsed = php_rs_parser::parse(&arena, &source);
        // SAFETY: `parsed` borrows from `arena` and `source`, both owned by this
        // struct and kept alive until `Drop`. `Drop` manually destroys `parsed`
        // before releasing either owner, so the widened lifetimes never escape.
        let parsed = unsafe {
            std::mem::transmute::<
                php_rs_parser::ParseResult<'_, '_>,
                php_rs_parser::ParseResult<'static, 'static>,
            >(parsed)
        };
        Self {
            file,
            source,
            parsed: ManuallyDrop::new(parsed),
            arena: ManuallyDrop::new(arena),
        }
    }

    fn source(&self) -> &str {
        self.source.as_ref()
    }

    fn parsed(&self) -> &php_rs_parser::ParseResult<'_, '_> {
        &self.parsed
    }
}

impl Drop for ParsedProjectFile {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.parsed);
            ManuallyDrop::drop(&mut self.arena);
        }
    }
}

// SAFETY: after construction the parsed AST and source map are read-only. The
// bump arena is never mutated again; it only owns backing storage for AST nodes
// and is dropped after all parallel analysis has completed.
unsafe impl Send for ParsedProjectFile {}
unsafe impl Sync for ParsedProjectFile {}

impl ProjectAnalyzer {
    pub fn new() -> Self {
        Self {
            shared_db: Arc::new(SharedDb::new()),
            cache: None,
            on_file_done: None,
            psr4: None,
            suppressed_issue_kinds: std::collections::HashSet::new(),
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
        }
    }

    /// Create a `ProjectAnalyzer` with a disk-backed cache stored under `cache_dir`.
    pub fn with_cache(cache_dir: &Path) -> Self {
        Self {
            shared_db: Arc::new(SharedDb::new().with_cache_dir(cache_dir)),
            cache: Some(AnalysisCache::open(cache_dir)),
            on_file_done: None,
            psr4: None,
            suppressed_issue_kinds: std::collections::HashSet::new(),
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
        }
    }

    /// Enable the disk-backed cache for an already-constructed analyzer.
    pub fn set_cache_dir(&mut self, cache_dir: &Path) {
        // Rebuild SharedDb to attach the Pass-1 stub cache. Must be called
        // before any file is ingested — a previously-populated SharedDb's
        // state would be silently discarded here, which is almost certainly
        // a caller bug rather than the intended behavior.
        debug_assert_eq!(
            self.shared_db.source_file_count(),
            0,
            "ProjectAnalyzer::set_cache_dir must be called before any file is ingested"
        );
        self.shared_db = Arc::new(SharedDb::new().with_cache_dir(cache_dir));
        self.cache = Some(AnalysisCache::open(cache_dir));
    }

    /// Create a `ProjectAnalyzer` from a project root containing `composer.json`.
    /// Returns the analyzer (with `psr4` set) and the `Psr4Map` so callers can
    /// call `map.project_files()` / `map.vendor_files()`.
    pub fn from_composer(
        root: &Path,
    ) -> Result<(Self, crate::composer::Psr4Map), crate::composer::ComposerError> {
        let map = crate::composer::Psr4Map::from_composer(root)?;
        let psr4 = Arc::new(map.clone());
        let analyzer = Self {
            shared_db: Arc::new(SharedDb::new()),
            cache: None,
            on_file_done: None,
            psr4: Some(psr4),
            suppressed_issue_kinds: std::collections::HashSet::new(),
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
        };
        Ok((analyzer, map))
    }

    /// Builder method: set the target PHP version.
    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version = Some(version);
        self
    }

    /// True iff at least one [`IssueKind`] emitted by the dead-code pass is
    /// not currently suppressed, so it's worth running.
    fn should_run_dead_code(&self) -> bool {
        dead_code_issue_kinds()
            .iter()
            .any(|k| !self.suppressed_issue_kinds.contains(*k))
    }

    /// Builder method: set a progress callback invoked once per analyzed file.
    pub fn with_progress_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_file_done = Some(callback);
        self
    }

    /// Builder method: add user stub files.
    pub fn with_stub_files(mut self, files: Vec<PathBuf>) -> Self {
        self.stub_files = files;
        self
    }

    /// Builder method: add user stub directories.
    pub fn with_stub_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.stub_dirs = dirs;
        self
    }

    /// Drop issues whose [`IssueKind::name()`] is listed in
    /// [`Self::suppressed_issue_kinds`]. Centralized post-filter so analyzer
    /// internals never need to know what the consumer cares about.
    fn apply_issue_suppressions(&self, issues: &mut Vec<mir_issues::Issue>) {
        if self.suppressed_issue_kinds.is_empty() {
            return;
        }
        issues.retain(|i| !self.suppressed_issue_kinds.contains(i.kind.name()));
    }

    /// Builder method: configure a disk-backed cache at the given directory.
    pub fn with_cache_dir(mut self, cache_dir: &Path) -> Self {
        debug_assert_eq!(
            self.shared_db.source_file_count(),
            0,
            "ProjectAnalyzer::with_cache_dir must be called before any file is ingested"
        );
        self.shared_db = Arc::new(SharedDb::new().with_cache_dir(cache_dir));
        self.cache = Some(AnalysisCache::open(cache_dir));
        self
    }

    /// Builder method: attach a PSR-4 autoloader map.
    pub fn with_psr4(mut self, map: Arc<crate::composer::Psr4Map>) -> Self {
        self.psr4 = Some(map);
        self
    }

    /// Resolve the configured PHP version, defaulting to `PhpVersion::LATEST`
    /// when none has been set.
    fn resolved_php_version(&self) -> PhpVersion {
        self.php_version.unwrap_or(PhpVersion::LATEST)
    }

    /// Cumulative hit / miss counts on the persistent Pass-1 cache attached
    /// to this analyzer. `(0, 0)` when no cache is configured. Used by
    /// integration tests and benchmarks to assert the cache actually fires.
    #[doc(hidden)]
    pub fn stub_cache_stats(&self) -> (u64, u64) {
        match self.shared_db.stub_cache.as_deref() {
            Some(c) => (c.hits(), c.misses()),
            None => (0, 0),
        }
    }

    fn type_exists(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::type_exists_via_db(&db, fqcn)
    }

    /// Returns `true` if a function with `fqn` is registered and active.
    pub fn contains_function(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::new(&db, Arc::<str>::from(fqn));
        crate::db::find_function(&db, here).is_some()
    }

    /// Returns `true` if a class / interface / trait / enum is registered.
    pub fn contains_class(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::new(&db, Arc::<str>::from(fqcn));
        crate::db::find_class_like(&db, here).is_some()
    }

    /// Returns `true` if `class` has a method named `name` (case-insensitive).
    pub fn contains_method(&self, class: &str, name: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::has_method_in_chain(&db, class, name)
    }

    /// Acquire a cheap clone of the salsa db for a read-only query.
    /// The lock is held only for the duration of the clone, so concurrent
    /// readers never serialize on each other or on writes longer than the
    /// clone itself.
    fn snapshot_db(&self) -> MirDb {
        self.shared_db.snapshot_db()
    }

    /// Internal: expose the salsa db for unit tests that need a `&dyn MirDatabase`.
    #[doc(hidden)]
    pub fn salsa_db_for_test(&self) -> parking_lot::MappedRwLockWriteGuard<'_, MirDb> {
        let guard = self.shared_db.salsa.write();
        parking_lot::RwLockWriteGuard::map(guard, |rw| &mut **rw)
    }

    /// Legacy: look up the source location of a class member by name.
    ///
    /// Prefer [`Self::definition_of`] with [`crate::Symbol::method`] etc.
    #[doc(hidden)]
    pub fn member_location(
        &self,
        fqcn: &str,
        member_name: &str,
    ) -> Option<mir_codebase::storage::Location> {
        let db = self.snapshot_db();
        crate::db::member_location_via_db(&db, fqcn, member_name)
    }

    /// Legacy: look up a top-level symbol location.
    ///
    /// Prefer [`Self::definition_of`] with [`crate::Symbol`].
    #[doc(hidden)]
    pub fn symbol_location(&self, symbol: &str) -> Option<mir_codebase::storage::Location> {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::new(&db, Arc::<str>::from(symbol));
        if let Some(class) = crate::db::find_class_like(&db, here) {
            if let Some(loc) = class.location() {
                return Some(loc.clone());
            }
        }
        crate::db::find_function(&db, here).and_then(|f| f.location.clone())
    }

    /// Legacy: raw reference locations as `(file, line, col_start, col_end)`.
    ///
    /// Prefer [`Self::references_to`] which returns `(Arc<str>, Range)` pairs
    /// and takes a strongly-typed [`crate::Symbol`].
    #[doc(hidden)]
    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// Resolve a symbol to its declaration location.
    ///
    /// Mirrors [`crate::AnalysisSession::definition_of`].
    pub fn definition_of(
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

    /// All recorded references to a symbol, as `(file, range)` pairs.
    ///
    /// Mirrors [`crate::AnalysisSession::references_to`].
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

    /// Load PHP built-in stubs. Called automatically by `analyze` if not done yet.
    /// Stubs are filtered against the configured target PHP version (or
    /// `PhpVersion::LATEST` if none was set).
    pub fn load_stubs(&self) {
        let php_version = self.resolved_php_version();

        // Wire the PHP version into the db before any SourceFile inputs are
        // registered — collect_file_definitions reads it for @since/@removed filtering.
        {
            let version_str = Arc::from(php_version.to_string().as_str());
            self.shared_db.salsa.write().set_php_version(version_str);
        }

        // Load all built-in stubs for the configured PHP version
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.shared_db.ingest_stub_paths(&paths, php_version);

        // Load user-configured stubs
        self.shared_db
            .ingest_user_stubs(&self.stub_files, &self.stub_dirs);

        // Ensure a resolver is configured so pull-path lookups (`find_class_like`,
        // `find_function`) can map built-in FQCNs to the stub VFS paths registered
        // as SourceFile inputs above. If a PSR-4 / user resolver is already wired
        // (e.g. via `from_composer`), it's chained with `StubClassResolver` at
        // session-construction time elsewhere.
        let mut guard = self.shared_db.salsa.write();
        if guard.current_resolver().is_none() {
            let resolver: Arc<dyn crate::ClassResolver> = Arc::new(crate::StubClassResolver);
            guard.set_resolver(Some(resolver));
        }
    }

    fn collect_and_ingest_source(&self, file: Arc<str>, src: &str) -> FileDefinitions {
        self.shared_db
            .collect_and_ingest_file(file, src, self.resolved_php_version())
    }

    /// Run the full analysis pipeline on a set of file paths.
    pub fn analyze(&self, paths: &[PathBuf]) -> AnalysisResult {
        let mut all_issues = Vec::new();
        let _t0 = std::time::Instant::now();

        // ---- Load PHP built-in stubs (before Pass 1 so user code can override)
        self.load_stubs();
        let _t_stubs = _t0.elapsed();

        // ---- Pass 1: read files in parallel ----------------------------------
        let parsed_files: Vec<ParsedProjectFile> = paths
            .par_iter()
            .filter_map(|path| match std::fs::read_to_string(path) {
                Ok(src) => {
                    let file = Arc::from(path.to_string_lossy().as_ref());
                    Some(ParsedProjectFile::new(file, Arc::from(src)))
                }
                Err(e) => {
                    eprintln!("Cannot read {}: {}", path.display(), e);
                    None
                }
            })
            .collect();
        let _t_read = _t0.elapsed();

        let file_data: Vec<(Arc<str>, Arc<str>)> = parsed_files
            .iter()
            .map(|parsed| (parsed.file.clone(), parsed.source.clone()))
            .collect();

        // ---- Pre-Pass-2 invalidation: evict dependents of changed files ------
        if let Some(cache) = &self.cache {
            let changed: Vec<String> = file_data
                .par_iter()
                .filter_map(|(f, src)| {
                    let h = hash_content(src.as_ref());
                    if cache.get(f, &h).is_none() {
                        Some(f.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            if !changed.is_empty() {
                cache.evict_with_dependents(&changed);
            }
        }

        // ---- Register Salsa source inputs for incremental follow-up calls ----
        {
            let mut guard = self.shared_db.salsa.write();
            for parsed in &parsed_files {
                guard.upsert_source_file(parsed.file.clone(), parsed.source.clone());
            }
        }
        let _t_salsa_reg = _t0.elapsed();

        // ---- Pass 1: definition collection from the already-parsed AST -------
        let file_defs: Vec<FileDefinitions> = parsed_files
            .par_iter()
            .map(|parsed| {
                let parse_result = parsed.parsed();
                let mut all_issues: Vec<Issue> = parse_result
                    .errors
                    .iter()
                    .map(|err| {
                        crate::parser::parse_error_to_issue(
                            err,
                            &parsed.file,
                            parsed.source(),
                            &parse_result.source_map,
                        )
                    })
                    .collect();
                let collector = crate::collector::DefinitionCollector::new_for_slice(
                    parsed.file.clone(),
                    parsed.source(),
                    &parse_result.source_map,
                );
                let (mut slice, collector_issues) = collector.collect_slice(&parse_result.program);
                all_issues.extend(collector_issues);
                mir_codebase::storage::deduplicate_params_in_slice(&mut slice);
                FileDefinitions {
                    slice: Arc::new(slice),
                    issues: Arc::new(all_issues),
                }
            })
            .collect();
        let _t_pass1 = _t0.elapsed();

        let mut files_with_parse_errors: std::collections::HashSet<Arc<str>> =
            std::collections::HashSet::new();
        {
            let mut guard = self.shared_db.salsa.write();
            for defs in file_defs {
                for issue in defs.issues.iter() {
                    if matches!(issue.kind, mir_issues::IssueKind::ParseError { .. })
                        && issue.severity == mir_issues::Severity::Error
                    {
                        files_with_parse_errors.insert(issue.location.file.clone());
                    }
                }
                guard.ingest_stub_slice(&defs.slice);
                all_issues.extend(Arc::unwrap_or_clone(defs.issues));
            }
        }
        let _t_ingest = _t0.elapsed();

        // ---- Pre-warm collect_file_definitions for project files -------------
        // After ingest, project SourceFiles have their text set in salsa.
        // Prime the tracked `collect_file_definitions` cache for each project
        // file in parallel so that `workspace_symbol_index` (called from class
        // analysis and Pass-2) finds all cache hits and doesn't need to
        // (re-)parse project files inline — eliminating a serial bottleneck on
        // cold starts, especially at high thread counts.
        {
            let db_prewarm = {
                let guard = self.shared_db.salsa.read();
                (**guard).clone()
            };
            let project_source_files: Vec<SourceFile> = {
                let guard = self.shared_db.salsa.read();
                parsed_files
                    .iter()
                    .filter_map(|p| (**guard).lookup_source_file(&p.file))
                    .collect()
            };
            project_source_files
                .into_par_iter()
                .for_each_with(db_prewarm, |db, sf| {
                    let _ = collect_file_definitions(db as &dyn MirDatabase, sf);
                });
        }

        // ---- Lazy-load unknown classes via PSR-4 (issue #50) ----------------
        if let Some(psr4) = &self.psr4 {
            self.lazy_load_missing_classes(psr4.clone(), &mut all_issues);
        }

        // ---- Resolve @psalm-import-type declarations now that all Pass 1
        // classes (including their `type_aliases`) are populated.
        // ---- Build reverse dep graph and persist it for the next run ---------
        if let Some(cache) = &self.cache {
            let db_snapshot = {
                let guard = self.shared_db.salsa.read();
                (**guard).clone()
            };
            let rev = build_reverse_deps(&db_snapshot);
            cache.set_reverse_deps(rev);
        }

        // ---- Class-level checks (M11) ----------------------------------------
        let analyzed_file_set: std::collections::HashSet<std::sync::Arc<str>> =
            file_data.iter().map(|(f, _)| f.clone()).collect();
        {
            let class_db = {
                let guard = self.shared_db.salsa.read();
                (**guard).clone()
            };
            let class_issues =
                crate::class::ClassAnalyzer::with_files(&class_db, analyzed_file_set, &file_data)
                    .analyze_all();
            all_issues.extend(class_issues);
        }

        // ---- Inference pre-sweep: prime inferred return types ----------------
        // Run an inference-only Pass 2 over each file in parallel using direct
        // rayon (no Salsa tracked-query overhead per file), collect the results,
        // then commit them to Salsa INPUT fields.  The full Pass 2 then reads
        // those fields via O(1) accesses with no lock contention.
        //
        // We use `Pass2Driver::new_inference_only` directly rather than the
        // Salsa-tracked `infer_file_return_types` query so that the batch path
        // avoids per-file Salsa lock acquisition and memo-table overhead on every
        // cold start.  `infer_file_return_types` is reserved for the incremental
        // LSP path (AnalysisSession) where Salsa cache hits across edits matter.
        //
        // `map_with` clones `db_priming` once per rayon worker thread (not once
        // per file as the old `in_place_scope` loop did). For N files on T threads
        // this reduces clones from N to T.  Results are returned by value and
        // flattened after `collect()`, replacing the Arc<Mutex<Vec>> accumulator.
        // All per-thread db clones are dropped when `collect()` returns, so
        // `commit_inferred_return_types` (which calls Salsa setters that wait for
        // strong_count == 1) cannot deadlock.
        {
            let db_priming = {
                let guard = self.shared_db.salsa.read();
                (**guard).clone()
            };
            let php_version = self.resolved_php_version();
            let all_inferred: Vec<InferredTypes> = parsed_files
                .par_iter()
                .filter(|parsed| !files_with_parse_errors.contains(&parsed.file))
                .map_with(db_priming, |db, parsed| {
                    let driver = Pass2Driver::new_inference_only(
                        db as &dyn crate::db::MirDatabase,
                        php_version,
                    );
                    let parse_result = parsed.parsed();
                    driver.analyze_bodies(
                        &parse_result.program,
                        parsed.file.clone(),
                        parsed.source(),
                        &parse_result.source_map,
                    );
                    driver.take_inferred_types()
                })
                .collect();
            // db_priming is consumed by map_with; per-thread clones dropped by collect().
            let mut functions = Vec::new();
            let mut methods = Vec::new();
            for inferred in all_inferred {
                functions.extend(inferred.functions);
                methods.extend(inferred.methods);
            }
            let mut guard = self.shared_db.salsa.write();
            guard.commit_inferred_return_types(functions, methods);
        }
        let _t_presweep = _t0.elapsed();

        let db_main = {
            let guard = self.shared_db.salsa.read();
            (**guard).clone()
        };

        // ---- Pass 2: analyze function/method bodies in parallel (M14) --------
        // Each worker db clone has its own `pending_ref_locs` buffer (custom
        // Clone returns empty).  Workers push reference locations there instead
        // of into the shared Arc<Mutex<...>> maps, eliminating cross-thread
        // contention.  After collect() we commit all batches serially in a
        // single lock acquisition per map.
        let pass2_results: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>, Vec<RefLoc>)> =
            parsed_files
                .par_iter()
                .filter(|parsed| !files_with_parse_errors.contains(&parsed.file))
                .map_with(db_main, |db, parsed| {
                    let driver =
                        Pass2Driver::new(&*db as &dyn MirDatabase, self.resolved_php_version());
                    let (issues, symbols) = if let Some(cache) = &self.cache {
                        let h = hash_content(parsed.source());
                        if let Some((cached_issues, ref_locs)) = cache.get(&parsed.file, &h) {
                            db.replay_reference_locations(parsed.file.clone(), &ref_locs);
                            (cached_issues, Vec::new())
                        } else {
                            let parse_result = parsed.parsed();
                            let (issues, symbols) = driver.analyze_bodies(
                                &parse_result.program,
                                parsed.file.clone(),
                                parsed.source(),
                                &parse_result.source_map,
                            );
                            let pending = db.take_pending_ref_locs();
                            let cache_locs = pending
                                .iter()
                                .map(|r| (r.symbol_key.to_string(), r.line, r.col_start, r.col_end))
                                .collect();
                            cache.put(&parsed.file, h, issues.clone(), cache_locs);
                            if let Some(cb) = &self.on_file_done {
                                cb();
                            }
                            return (issues, symbols, pending);
                        }
                    } else {
                        let parse_result = parsed.parsed();
                        driver.analyze_bodies(
                            &parse_result.program,
                            parsed.file.clone(),
                            parsed.source(),
                            &parse_result.source_map,
                        )
                    };
                    let pending = db.take_pending_ref_locs();
                    if let Some(cb) = &self.on_file_done {
                        cb();
                    }
                    (issues, symbols, pending)
                })
                .collect();

        let _t_pass2 = _t0.elapsed();

        // Serial commit: one lock acquisition per map for all files combined.
        let mut all_ref_locs: Vec<RefLoc> = Vec::new();
        let mut all_symbols = Vec::new();
        for (issues, symbols, ref_locs) in pass2_results {
            all_issues.extend(issues);
            all_symbols.extend(symbols);
            all_ref_locs.extend(ref_locs);
        }
        {
            let guard = self.shared_db.salsa.read();
            guard.commit_reference_locations_batch(all_ref_locs);
        }

        // ---- Post-Pass-2 lazy loading: FQCNs used without `use` imports ------
        // FQCNs in function/method bodies aren't visible until Pass 2 runs, so
        // the pre-Pass-2 lazy load misses them.  We collect UndefinedClass names,
        // resolve them via PSR-4, load those files, re-finalize, then re-analyze
        // only the affected files to clear the false positives.
        if let Some(psr4) = &self.psr4 {
            self.lazy_load_from_body_issues(
                psr4.clone(),
                &file_data,
                &files_with_parse_errors,
                &mut all_issues,
                &mut all_symbols,
            );
        }

        // Persist cache hits/misses to disk
        if let Some(cache) = &self.cache {
            cache.flush();
        }

        // ---- Compact the reference index ------------------------------------
        // ---- Dead-code detection (M18) --------------------------------------
        if self.should_run_dead_code() {
            let salsa = self.snapshot_db();
            let dead_code_issues = crate::dead_code::DeadCodeAnalyzer::new(&salsa).analyze();
            all_issues.extend(dead_code_issues);
        }

        let _t_total = _t0.elapsed();
        if std::env::var("MIR_TIMING").is_ok() {
            eprintln!(
                "[timing] stubs={:.0}ms read={:.0}ms salsa_reg={:.0}ms pass1={:.0}ms ingest={:.0}ms presweep={:.0}ms pass2={:.0}ms total={:.0}ms",
                _t_stubs.as_secs_f64() * 1000.0,
                (_t_read - _t_stubs).as_secs_f64() * 1000.0,
                (_t_salsa_reg - _t_read).as_secs_f64() * 1000.0,
                (_t_pass1 - _t_salsa_reg).as_secs_f64() * 1000.0,
                (_t_ingest - _t_pass1).as_secs_f64() * 1000.0,
                (_t_presweep - _t_ingest).as_secs_f64() * 1000.0,
                (_t_pass2 - _t_presweep).as_secs_f64() * 1000.0,
                _t_total.as_secs_f64() * 1000.0,
            );
        }

        self.apply_issue_suppressions(&mut all_issues);
        if let Some(dump) = crate::metrics::dump() {
            eprintln!("{dump}");
        }
        AnalysisResult::build(all_issues, std::collections::HashMap::new(), all_symbols)
    }

    fn lazy_load_missing_classes(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        all_issues: &mut Vec<Issue>,
    ) {
        use std::collections::HashSet;
        use std::sync::Arc;

        let max_depth = 10;
        let mut loaded: HashSet<String> = HashSet::new();
        let mut scanned: HashSet<Arc<str>> = HashSet::new();

        for _ in 0..max_depth {
            let mut to_load: Vec<(String, PathBuf)> = Vec::new();

            let mut try_queue = |fqcn: &str| {
                if !self.type_exists(fqcn) && !loaded.contains(fqcn) {
                    if let Some(path) = psr4.resolve(fqcn) {
                        to_load.push((fqcn.to_string(), path));
                    }
                }
            };

            // Collect inheritance and import candidates. Only scan classes that
            // haven't been scanned yet (optimization: avoid redundant full scans).
            let mut inheritance_candidates = Vec::new();
            let import_candidates = {
                let db_owned = self.snapshot_db();
                let db = &db_owned;
                for fqcn in crate::db::workspace_classes(db).iter() {
                    if scanned.contains(fqcn.as_ref()) {
                        continue;
                    }
                    let here = crate::db::Fqcn::new(db, fqcn.clone());
                    let Some(class) = crate::db::find_class_like(db, here) else {
                        continue;
                    };
                    scanned.insert(fqcn.clone());
                    if class.is_interface() {
                        for parent in class.extends().iter() {
                            inheritance_candidates.push(parent.to_string());
                        }
                    } else if class.is_enum() {
                        for iface in class.interfaces().iter() {
                            inheritance_candidates.push(iface.to_string());
                        }
                    } else if class.is_trait() {
                        for used in class.class_traits().iter() {
                            inheritance_candidates.push(used.to_string());
                        }
                    } else {
                        if let Some(parent) = class.parent() {
                            inheritance_candidates.push(parent.to_string());
                        }
                        for iface in class.interfaces().iter() {
                            inheritance_candidates.push(iface.to_string());
                        }
                    }
                }
                db.file_import_snapshots()
                    .into_iter()
                    .flat_map(|(_, imports)| imports.into_values())
                    .collect::<Vec<_>>()
            };
            for fqcn in inheritance_candidates {
                try_queue(&fqcn);
            }

            // Also lazy-load any type referenced via `use` imports that isn't yet
            // in the codebase (covers enums and classes used only in type hints or
            // static calls, which never appear in the inheritance scan above).
            for fqcn in import_candidates {
                try_queue(&fqcn);
            }

            if to_load.is_empty() {
                break;
            }

            for (fqcn, path) in to_load {
                loaded.insert(fqcn);
                if let Ok(src) = std::fs::read_to_string(&path) {
                    let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                    let defs = self.collect_and_ingest_source(file, &src);
                    all_issues.extend(Arc::unwrap_or_clone(defs.issues));
                }
            }
        }
    }

    fn lazy_load_from_body_issues(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        file_data: &[(Arc<str>, Arc<str>)],
        files_with_parse_errors: &HashSet<Arc<str>>,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<crate::symbol::ResolvedSymbol>,
    ) {
        use mir_issues::IssueKind;

        let max_depth = 5;
        let mut loaded: HashSet<String> = HashSet::new();

        for _ in 0..max_depth {
            // Deduplicate by FQCN: HashMap prevents loading the same class twice
            // when multiple files share the same UndefinedClass diagnostic.
            let mut to_load: HashMap<String, PathBuf> = HashMap::new();

            for issue in all_issues.iter() {
                if let IssueKind::UndefinedClass { name } = &issue.kind {
                    if !self.type_exists(name) && !loaded.contains(name) {
                        if let Some(path) = psr4.resolve(name) {
                            to_load.entry(name.clone()).or_insert(path);
                        }
                    }
                }
            }

            if to_load.is_empty() {
                break;
            }

            loaded.extend(to_load.keys().cloned());

            for path in to_load.values() {
                if let Ok(src) = std::fs::read_to_string(path) {
                    let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                    let _ = self.collect_and_ingest_source(file, &src);
                }
            }

            // Load inheritance deps of newly-added types and finalize.
            // This covers e.g. `class Helper extends \App\Base` where Base is
            // also not in the initial file set.
            self.lazy_load_missing_classes(psr4.clone(), all_issues);

            // Re-analyze every file that has an UndefinedClass for a type now
            // present in the codebase — covers both direct and transitive loads.
            let files_to_reanalyze: HashSet<Arc<str>> = all_issues
                .iter()
                .filter_map(|i| {
                    if let IssueKind::UndefinedClass { name } = &i.kind {
                        if self.type_exists(name) {
                            return Some(i.location.file.clone());
                        }
                    }
                    None
                })
                .collect();

            if files_to_reanalyze.is_empty() {
                break;
            }

            all_issues.retain(|i| !files_to_reanalyze.contains(&i.location.file));
            all_symbols.retain(|s| !files_to_reanalyze.contains(&s.file));

            let db_full = {
                let guard = self.shared_db.salsa.read();
                (**guard).clone()
            };

            let reanalysis: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>, Vec<RefLoc>)> =
                file_data
                    .par_iter()
                    .filter(|(f, _)| {
                        !files_with_parse_errors.contains(f) && files_to_reanalyze.contains(f)
                    })
                    .map_with(db_full, |db, (file, src)| {
                        let driver =
                            Pass2Driver::new(&*db as &dyn MirDatabase, self.resolved_php_version());
                        let arena = crate::arena::create_parse_arena(src.len());
                        let parsed = php_rs_parser::parse(&arena, src);
                        let (issues, symbols) = driver.analyze_bodies(
                            &parsed.program,
                            file.clone(),
                            src,
                            &parsed.source_map,
                        );
                        let pending = db.take_pending_ref_locs();
                        (issues, symbols, pending)
                    })
                    .collect();

            let mut reanalysis_ref_locs: Vec<RefLoc> = Vec::new();
            for (issues, symbols, ref_locs) in reanalysis {
                all_issues.extend(issues);
                all_symbols.extend(symbols);
                reanalysis_ref_locs.extend(ref_locs);
            }
            {
                let guard = self.shared_db.salsa.read();
                guard.commit_reference_locations_batch(reanalysis_ref_locs);
            }
        }
    }

    /// Re-analyze a single file within the existing codebase.
    ///
    /// This is the incremental analysis API for LSP:
    /// 1. Removes old definitions from this file
    /// 2. Re-runs Pass 1 (definition collection) on the new content
    /// 3. Resolves any newly-collected `@psalm-import-type` declarations
    /// 4. Re-runs Pass 2 (body analysis) on this file
    /// 5. Returns the analysis result for this file only
    pub fn re_analyze_file(&self, file_path: &str, new_content: &str) -> AnalysisResult {
        // Fast path: content unchanged and cache has a valid entry — skip full re-analysis.
        if let Some(cache) = &self.cache {
            let h = hash_content(new_content);
            if let Some((mut issues, ref_locs)) = cache.get(file_path, &h) {
                let file: Arc<str> = Arc::from(file_path);
                let guard = self.shared_db.salsa.read();
                guard.replay_reference_locations(file, &ref_locs);
                guard.commit_pending_to_maps();
                self.apply_issue_suppressions(&mut issues);
                return AnalysisResult::build(issues, HashMap::new(), Vec::new());
            }
        }

        let file: Arc<str> = Arc::from(file_path);

        {
            let mut guard = self.shared_db.salsa.write();
            guard.remove_file_definitions(file_path);
        }

        // --- Salsa-backed Pass 1: memoized parse + definition collection ------
        let file_defs = {
            let mut guard = self.shared_db.salsa.write();
            let salsa_file = guard.upsert_source_file(file.clone(), Arc::from(new_content));
            collect_file_definitions(&**guard, salsa_file)
        };

        let mut all_issues: Vec<Issue> = Arc::unwrap_or_clone(file_defs.issues.clone());

        let symbols = {
            let mut guard = self.shared_db.salsa.write();

            guard.ingest_stub_slice(&file_defs.slice);

            // Resolve any newly-collected @psalm-import-type declarations so
            // Pass 2 reads the imported aliases out of `type_aliases`.
            // Re-parse in the arena so Pass 2 can walk the AST.
            let arena = bumpalo::Bump::new();
            let parsed = php_rs_parser::parse(&arena, new_content);

            let has_hard_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);
            if !has_hard_errors {
                let db_ref: &dyn MirDatabase = &**guard;
                let driver = Pass2Driver::new(db_ref, self.resolved_php_version());
                let (body_issues, symbols) = driver.analyze_bodies(
                    &parsed.program,
                    file.clone(),
                    new_content,
                    &parsed.source_map,
                );
                all_issues.extend(body_issues);
                guard.commit_pending_to_maps();
                symbols
            } else {
                Vec::new()
            }
        };

        if let Some(cache) = &self.cache {
            let h = hash_content(new_content);
            cache.evict_with_dependents(&[file_path.to_string()]);
            let db = self.snapshot_db();
            let ref_locs = extract_reference_locations(&db, &file);
            cache.put(file_path, h, all_issues.clone(), ref_locs);
        }

        self.apply_issue_suppressions(&mut all_issues);
        AnalysisResult::build(all_issues, HashMap::new(), symbols)
    }

    /// Analyze a PHP source string without a real file path.
    /// Useful for tests and LSP single-file mode.
    pub fn analyze_source(source: &str) -> AnalysisResult {
        let analyzer = ProjectAnalyzer::new();
        let file: Arc<str> = Arc::from("<source>");
        let mut db = MirDb::default();
        for slice in crate::stubs::builtin_stub_slices_for_version(analyzer.resolved_php_version())
        {
            db.ingest_stub_slice(&slice);
        }
        let salsa_file = SourceFile::new(&db, file.clone(), Arc::from(source));
        let file_defs = collect_file_definitions(&db, salsa_file);
        db.ingest_stub_slice(&file_defs.slice);
        let mut all_issues = Arc::unwrap_or_clone(file_defs.issues);
        if all_issues.iter().any(|issue| {
            matches!(issue.kind, mir_issues::IssueKind::ParseError { .. })
                && issue.severity == mir_issues::Severity::Error
        }) {
            analyzer.apply_issue_suppressions(&mut all_issues);
            return AnalysisResult::build(all_issues, std::collections::HashMap::new(), Vec::new());
        }
        let mut type_envs = std::collections::HashMap::new();
        let mut all_symbols = Vec::new();
        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, source);

        let driver = Pass2Driver::new(&db, analyzer.resolved_php_version());
        all_issues.extend(driver.analyze_bodies_typed(
            &result.program,
            file.clone(),
            source,
            &result.source_map,
            &mut type_envs,
            &mut all_symbols,
        ));
        analyzer.apply_issue_suppressions(&mut all_issues);
        AnalysisResult::build(all_issues, type_envs, all_symbols)
    }

    /// Discover all `.php` files under a directory, recursively.
    pub fn discover_files(root: &Path) -> Vec<PathBuf> {
        if root.is_file() {
            return vec![root.to_path_buf()];
        }
        let mut files = Vec::new();
        collect_php_files(root, &mut files);
        files
    }

    /// Pass 1 only: collect type definitions from `paths` into the codebase without
    /// analyzing method bodies or emitting issues. Used to load vendor types.
    ///
    /// When [`Self::with_cache`] is enabled, per-file [`StubSlice`] results from
    /// previous runs are reused on a content-hash match, eliminating the
    /// parse + definition-collection step (which is ~95% of vendor wall-time
    /// on Laravel). Cache misses run the normal pipeline and write back so
    /// subsequent runs hit.
    ///
    /// [`StubSlice`]: mir_codebase::storage::StubSlice
    pub fn collect_types_only(&self, paths: &[PathBuf]) {
        let _timing = std::env::var("MIR_TIMING").is_ok();
        let _t0 = std::time::Instant::now();

        let php_v = self.resolved_php_version().cache_byte();

        // ---- Phase 1: read + try cache, in parallel ------------------------
        // Each entry carries either a ready-to-ingest cached slice, or the
        // source text + hash for the miss path that runs Pass 1.
        struct FileEntry {
            file: Arc<str>,
            src: Arc<str>,
            hash: [u8; 32],
            cached: Option<mir_codebase::storage::StubSlice>,
        }
        let entries: Vec<FileEntry> = paths
            .par_iter()
            .filter_map(|path| {
                let src = std::fs::read_to_string(path).ok()?;
                let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                let src: Arc<str> = Arc::from(src);
                let hash = hash_source(&src);
                let cached = self.shared_db.stub_cache.as_ref().and_then(|c| {
                    let mut slice = c.get(&file, &hash, php_v)?;
                    // Re-run dedup outside the serial ingest section so commit
                    // 3018a1d's parallel-dedup win is preserved on cache hits.
                    prepare_for_ingest(&mut slice);
                    Some(slice)
                });
                Some(FileEntry {
                    file,
                    src,
                    hash,
                    cached,
                })
            })
            .collect();
        let _t_read = _t0.elapsed();

        // ---- Phase 2: register all SourceFile inputs in salsa --------------
        // Lazy-load (e.g. UndefinedClass → vendor file) may later query any of
        // these as a salsa input, so we register both hits and misses.
        let source_files: Vec<SourceFile> = {
            let mut guard = self.shared_db.salsa.write();
            entries
                .iter()
                .map(|e| guard.upsert_source_file(e.file.clone(), e.src.clone()))
                .collect()
        };
        let _t_reg = _t0.elapsed();

        // ---- Phase 3: Pass 1 for misses, cache write-back, in parallel -----
        let db_pass1 = {
            let guard = self.shared_db.salsa.read();
            (**guard).clone()
        };
        let stub_cache = self.shared_db.stub_cache.clone();
        // Use the salsa-tracked `collect_file_definitions` for cache misses so
        // that `workspace_symbol_index` (called during project Pass-2) gets a
        // salsa cache HIT instead of re-parsing every vendor file a second time.
        // Clones share salsa storage, so results written here are visible to
        // the main db handle.  For stub-cache hits, the slice comes from disk and
        // salsa will parse on-demand (lazily) when the workspace index is built —
        // still only one parse per file total.
        let prepared: Vec<mir_codebase::storage::StubSlice> = entries
            .into_par_iter()
            .zip(source_files.into_par_iter())
            .map_with(db_pass1, |db, (mut entry, salsa_file)| {
                if let Some(slice) = entry.cached.take() {
                    return slice;
                }
                // Tracked version: result is memoized in the shared salsa
                // storage.  `workspace_symbol_index` will get a cache hit.
                let defs = collect_file_definitions(&*db, salsa_file);
                if let Some(cache) = stub_cache.as_ref() {
                    cache.put(&entry.file, &entry.hash, php_v, &defs.slice);
                }
                // Cheap clone: StubSlice now holds Vec<Arc<Storage>> items.
                (*defs.slice).clone()
            })
            .collect();
        let _t_collect = _t0.elapsed();

        let mut guard = self.shared_db.salsa.write();
        for slice in &prepared {
            guard.ingest_stub_slice(slice);
        }
        drop(guard);
        let _t_ingest = _t0.elapsed();

        if _timing {
            let (hits, misses) = self.stub_cache_stats();
            eprintln!(
                "[vendor] read={:.0}ms reg={:.0}ms collect={:.0}ms ingest={:.0}ms total={:.0}ms (cache hits={hits} misses={misses})",
                _t_read.as_secs_f64() * 1000.0,
                (_t_reg - _t_read).as_secs_f64() * 1000.0,
                (_t_collect - _t_reg).as_secs_f64() * 1000.0,
                (_t_ingest - _t_collect).as_secs_f64() * 1000.0,
                _t_ingest.as_secs_f64() * 1000.0,
            );
        }

        // Print profiling statistics for the collection phase.
        crate::collector::print_collector_stats();
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn collect_php_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(
                    name,
                    "vendor" | ".git" | "node_modules" | ".cache" | ".pnpm-store"
                ) {
                    continue;
                }
                collect_php_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("php") {
                out.push(path);
            }
        }
    }
}

// build_reverse_deps

fn build_reverse_deps(db: &dyn crate::db::MirDatabase) -> HashMap<String, HashSet<String>> {
    let mut reverse: HashMap<String, HashSet<String>> = HashMap::new();

    let mut add_edge = |symbol: &str, dependent_file: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            let def = defining_file.as_ref().to_string();
            if def != dependent_file {
                reverse
                    .entry(def)
                    .or_default()
                    .insert(dependent_file.to_string());
            }
        }
    };

    for (file, imports) in db.file_import_snapshots() {
        let file = file.as_ref().to_string();
        for fqcn in imports.values() {
            add_edge(fqcn, &file);
        }
    }

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

    for fqcn in crate::db::workspace_classes(db).iter() {
        let here = crate::db::Fqcn::new(db, fqcn.clone());
        let Some(class) = crate::db::find_class_like(db, here) else {
            continue;
        };
        // Only true classes contribute class-direction edges in this loop.
        if class.is_interface() || class.is_trait() || class.is_enum() {
            continue;
        }
        let Some(file) = db
            .symbol_defining_file(fqcn.as_ref())
            .map(|f| f.as_ref().to_string())
            .or_else(|| class.location().map(|l| l.file.as_ref().to_string()))
        else {
            continue;
        };

        if let Some(parent) = class.parent() {
            add_edge(parent.as_ref(), &file);
        }
        for iface in class.interfaces().iter() {
            add_edge(iface.as_ref(), &file);
        }
        for tr in class.class_traits().iter() {
            add_edge(tr.as_ref(), &file);
        }
        if let Some(props) = class.own_properties() {
            for (_, p) in props.iter() {
                if let Some(ty) = &p.ty {
                    for named in extract_named_objects(ty) {
                        add_edge(named.as_ref(), &file);
                    }
                }
            }
        }
        for (_, method) in class.own_methods().iter() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_edge(named.as_ref(), &file);
                    }
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                for named in extract_named_objects(rt) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
    }

    // Add types from global functions
    for fqn in crate::db::workspace_functions(db).iter() {
        let here = crate::db::Fqcn::new(db, fqn.clone());
        let Some(f) = crate::db::find_function(db, here) else {
            continue;
        };
        let Some(file) = db
            .symbol_defining_file(fqn.as_ref())
            .map(|f| f.as_ref().to_string())
            .or_else(|| f.location.as_ref().map(|l| l.file.as_ref().to_string()))
        else {
            continue;
        };

        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                for named in extract_named_objects(ty.as_ref()) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            for named in extract_named_objects(rt) {
                add_edge(named.as_ref(), &file);
            }
        }
    }

    // Also wire in bare-FQN references from Pass 2 (new \Foo(), \Foo::method(), \foo())
    // that do not appear in use-import statements.
    for (ref_file, symbol_key) in db.all_reference_location_pairs() {
        let file_str = ref_file.as_ref().to_string();
        let lookup: &str = match symbol_key.split_once("::") {
            Some((class, _)) => class,
            None => &symbol_key,
        };
        add_edge(lookup, &file_str);
    }

    reverse
}

fn extract_reference_locations(
    db: &dyn crate::db::MirDatabase,
    file: &Arc<str>,
) -> Vec<(String, u32, u16, u16)> {
    db.extract_file_reference_locations(file.as_ref())
        .into_iter()
        .map(|(sym, line, col_start, col_end)| (sym.to_string(), line, col_start, col_end))
        .collect()
}

pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    #[doc(hidden)]
    pub type_envs: std::collections::HashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
    /// Per-expression resolved symbols from Pass 2, sorted by file path.
    pub symbols: Vec<crate::symbol::ResolvedSymbol>,
    /// Maps each file path to the contiguous range within `symbols` that belongs
    /// to it. Built once after analysis; allows `symbol_at` to scan only the
    /// relevant file's slice rather than the entire codebase-wide vector.
    symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>>,
}

impl AnalysisResult {
    fn build(
        issues: Vec<Issue>,
        type_envs: std::collections::HashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        mut symbols: Vec<crate::symbol::ResolvedSymbol>,
    ) -> Self {
        symbols.sort_unstable_by(|a, b| a.file.as_ref().cmp(b.file.as_ref()));
        let mut symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>> = HashMap::new();
        let mut i = 0;
        while i < symbols.len() {
            let file = Arc::clone(&symbols[i].file);
            let start = i;
            while i < symbols.len() && symbols[i].file == file {
                i += 1;
            }
            symbols_by_file.insert(file, start..i);
        }
        Self {
            issues,
            type_envs,
            symbols,
            symbols_by_file,
        }
    }
}

impl AnalysisResult {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Warning)
            .count()
    }

    /// Group issues by source file.
    pub fn issues_by_file(&self) -> HashMap<std::sync::Arc<str>, Vec<&Issue>> {
        let mut map: HashMap<std::sync::Arc<str>, Vec<&Issue>> = HashMap::new();
        for issue in &self.issues {
            map.entry(issue.location.file.clone())
                .or_default()
                .push(issue);
        }
        map
    }

    /// Count issues by severity. Returned as `(severity, count)` pairs sorted
    /// by severity (Info, Warning, Error).
    pub fn count_by_severity(&self) -> Vec<(mir_issues::Severity, usize)> {
        let mut counts: std::collections::BTreeMap<mir_issues::Severity, usize> =
            std::collections::BTreeMap::new();
        for issue in &self.issues {
            *counts.entry(issue.severity).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }

    /// Total number of issues across all severities and files.
    pub fn total_issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Iterator of issues matching `predicate`. Useful for filtering by
    /// severity, kind, or file without materializing intermediate vectors.
    pub fn filter_issues<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a Issue>
    where
        F: Fn(&Issue) -> bool + 'a,
    {
        self.issues.iter().filter(move |i| predicate(i))
    }

    /// Return the innermost resolved symbol whose span contains `byte_offset`
    /// in `file`, or `None` if no symbol was recorded at that position.
    pub fn symbol_at(
        &self,
        file: &str,
        byte_offset: u32,
    ) -> Option<&crate::symbol::ResolvedSymbol> {
        let range = self.symbols_by_file.get(file)?;
        self.symbols[range.clone()]
            .iter()
            .filter(|s| s.span.start <= byte_offset && byte_offset < s.span.end)
            .min_by_key(|s| s.span.end - s.span.start)
    }
}
