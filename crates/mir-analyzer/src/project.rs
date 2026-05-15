/// Project-level orchestration: file discovery, pass 1, pass 2.
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use std::collections::{HashMap, HashSet};

use crate::cache::{hash_content, AnalysisCache};
use crate::db::{
    collect_file_definitions, collect_file_definitions_uncached, FileDefinitions, MirDatabase,
    MirDb, SourceFile,
};
use crate::pass2::{InferredTypes, Pass2Driver};
use crate::php_version::PhpVersion;
use crate::shared_db::SharedDb;
use mir_issues::Issue;

pub(crate) use crate::pass2::merge_return_types;

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
    /// When true, run dead code detection at the end of analysis.
    pub find_dead_code: bool,
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
            find_dead_code: false,
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
        }
    }

    /// Create a `ProjectAnalyzer` with a disk-backed cache stored under `cache_dir`.
    pub fn with_cache(cache_dir: &Path) -> Self {
        Self {
            shared_db: Arc::new(SharedDb::new()),
            cache: Some(AnalysisCache::open(cache_dir)),
            on_file_done: None,
            psr4: None,
            find_dead_code: false,
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
        }
    }

    /// Enable the disk-backed cache for an already-constructed analyzer.
    pub fn set_cache_dir(&mut self, cache_dir: &Path) {
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
            find_dead_code: false,
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

    /// Builder method: enable dead-code detection at the end of analysis.
    pub fn with_dead_code(mut self, enabled: bool) -> Self {
        self.find_dead_code = enabled;
        self
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

    /// Builder method: configure a disk-backed cache at the given directory.
    pub fn with_cache_dir(mut self, cache_dir: &Path) -> Self {
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

    fn type_exists(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::type_exists_via_db(&db, fqcn)
    }

    /// Returns `true` if a function with `fqn` is registered and active.
    pub fn contains_function(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        db.lookup_function_node(fqn).is_some_and(|n| n.active(&db))
    }

    /// Returns `true` if a class / interface / trait / enum is registered.
    pub fn contains_class(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        db.lookup_class_node(fqcn).is_some_and(|n| n.active(&db))
    }

    /// Returns `true` if `class` has a method named `name` (case-insensitive).
    pub fn contains_method(&self, class: &str, name: &str) -> bool {
        let db = self.snapshot_db();
        let name_lower = name.to_ascii_lowercase();
        db.lookup_method_node(class, &name_lower)
            .is_some_and(|n| n.active(&db))
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
        db.lookup_class_node(symbol)
            .filter(|n| n.active(&db))
            .and_then(|n| n.location(&db))
            .or_else(|| {
                db.lookup_function_node(symbol)
                    .filter(|n| n.active(&db))
                    .and_then(|n| n.location(&db))
            })
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
                let node = db
                    .lookup_class_node(fqcn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                node.location(&db)
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Function(fqn) => {
                let node = db
                    .lookup_function_node(fqn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                node.location(&db)
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

        // Load all built-in stubs for the configured PHP version
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.shared_db.ingest_stub_paths(&paths, php_version);

        // Load user-configured stubs
        self.shared_db
            .ingest_user_stubs(&self.stub_files, &self.stub_dirs);
    }

    fn collect_and_ingest_source(&self, file: Arc<str>, src: &str) -> FileDefinitions {
        self.shared_db.collect_and_ingest_file(file, src)
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
                        Issue::new(
                            mir_issues::IssueKind::ParseError {
                                message: err.to_string(),
                            },
                            mir_issues::Location {
                                file: parsed.file.clone(),
                                line: 1,
                                line_end: 1,
                                col_start: 0,
                                col_end: 0,
                            },
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
                    if matches!(issue.kind, mir_issues::IssueKind::ParseError { .. }) {
                        files_with_parse_errors.insert(issue.location.file.clone());
                    }
                }
                guard.ingest_stub_slice(&defs.slice);
                all_issues.extend(Arc::unwrap_or_clone(defs.issues));
            }
        }
        let _t_ingest = _t0.elapsed();

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
        let pass2_results: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = parsed_files
            .par_iter()
            .filter(|parsed| !files_with_parse_errors.contains(&parsed.file))
            .map_with(db_main, |db, parsed| {
                let driver =
                    Pass2Driver::new(&*db as &dyn MirDatabase, self.resolved_php_version());
                let result = if let Some(cache) = &self.cache {
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
                        let ref_locs = extract_reference_locations(&*db, &parsed.file);
                        cache.put(&parsed.file, h, issues.clone(), ref_locs);
                        (issues, symbols)
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
                if let Some(cb) = &self.on_file_done {
                    cb();
                }
                result
            })
            .collect();

        let _t_pass2 = _t0.elapsed();
        let mut all_symbols = Vec::new();
        for (issues, symbols) in pass2_results {
            all_issues.extend(issues);
            all_symbols.extend(symbols);
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
        if self.find_dead_code {
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
                for fqcn in db.active_class_node_fqcns() {
                    if scanned.contains(fqcn.as_ref()) {
                        continue;
                    }
                    let Some(node) = db.lookup_class_node(&fqcn) else {
                        continue;
                    };
                    scanned.insert(fqcn.clone());
                    if node.is_interface(db) {
                        for parent in node.extends(db).iter() {
                            inheritance_candidates.push(parent.to_string());
                        }
                    } else if node.is_enum(db) {
                        for iface in node.interfaces(db).iter() {
                            inheritance_candidates.push(iface.to_string());
                        }
                    } else if node.is_trait(db) {
                        for used in node.traits(db).iter() {
                            inheritance_candidates.push(used.to_string());
                        }
                    } else {
                        if let Some(parent) = node.parent(db) {
                            inheritance_candidates.push(parent.to_string());
                        }
                        for iface in node.interfaces(db).iter() {
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

            let reanalysis: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = file_data
                .par_iter()
                .filter(|(f, _)| {
                    !files_with_parse_errors.contains(f) && files_to_reanalyze.contains(f)
                })
                .map_with(db_full, |db, (file, src)| {
                    let driver =
                        Pass2Driver::new(&*db as &dyn MirDatabase, self.resolved_php_version());
                    let arena = crate::arena::create_parse_arena(src.len());
                    let parsed = php_rs_parser::parse(&arena, src);
                    driver.analyze_bodies(&parsed.program, file.clone(), src, &parsed.source_map)
                })
                .collect();

            for (issues, symbols) in reanalysis {
                all_issues.extend(issues);
                all_symbols.extend(symbols);
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
            if let Some((issues, ref_locs)) = cache.get(file_path, &h) {
                let file: Arc<str> = Arc::from(file_path);
                let guard = self.shared_db.salsa.read();
                guard.replay_reference_locations(file, &ref_locs);
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

        // --- S2 + Pass 2: hold the Salsa lock for ClassNode upserts and body
        // analysis so the db reference is live during Pass 2 (S5).
        let symbols = {
            let mut guard = self.shared_db.salsa.write();

            guard.ingest_stub_slice(&file_defs.slice);

            // Resolve any newly-collected @psalm-import-type declarations so
            // Pass 2 reads the imported aliases out of `type_aliases`.
            // Re-parse in the arena so Pass 2 can walk the AST.
            let arena = bumpalo::Bump::new();
            let parsed = php_rs_parser::parse(&arena, new_content);

            if parsed.errors.is_empty() {
                let db_ref: &dyn MirDatabase = &**guard;
                let driver = Pass2Driver::new(db_ref, self.resolved_php_version());
                let (body_issues, symbols) = driver.analyze_bodies(
                    &parsed.program,
                    file.clone(),
                    new_content,
                    &parsed.source_map,
                );
                all_issues.extend(body_issues);
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
        if all_issues
            .iter()
            .any(|issue| matches!(issue.kind, mir_issues::IssueKind::ParseError { .. }))
        {
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
    pub fn collect_types_only(&self, paths: &[PathBuf]) {
        let _timing = std::env::var("MIR_TIMING").is_ok();
        let _t0 = std::time::Instant::now();

        let file_data: Vec<(Arc<str>, Arc<str>)> = paths
            .par_iter()
            .filter_map(|path| {
                let src = std::fs::read_to_string(path).ok()?;
                Some((
                    Arc::from(path.to_string_lossy().as_ref()),
                    Arc::<str>::from(src),
                ))
            })
            .collect();
        let _t_read = _t0.elapsed();

        let source_files: Vec<SourceFile> = {
            let mut guard = self.shared_db.salsa.write();
            file_data
                .iter()
                .map(|(file, src)| guard.upsert_source_file(file.clone(), src.clone()))
                .collect()
        };
        let _t_reg = _t0.elapsed();

        let db_pass1 = {
            let guard = self.shared_db.salsa.read();
            (**guard).clone()
        };

        let file_defs: Vec<FileDefinitions> = source_files
            .par_iter()
            .map_with(db_pass1, |db, salsa_file| {
                collect_file_definitions_uncached(&*db, *salsa_file)
            })
            .collect();
        let _t_collect = _t0.elapsed();

        let mut guard = self.shared_db.salsa.write();
        for defs in file_defs {
            guard.ingest_stub_slice(&defs.slice);
        }
        drop(guard);
        let _t_ingest = _t0.elapsed();

        if _timing {
            eprintln!(
                "[vendor] read={:.0}ms reg={:.0}ms collect={:.0}ms ingest={:.0}ms total={:.0}ms",
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

    for fqcn in db.active_class_node_fqcns() {
        // Only true classes contribute class-direction edges in this loop.
        // Interface / trait / enum edges are not currently emitted here —
        // this function only ever read classes.
        let kind = match crate::db::class_kind_via_db(db, fqcn.as_ref()) {
            Some(k) if !k.is_interface && !k.is_trait && !k.is_enum => k,
            _ => continue,
        };
        let _ = kind;
        let Some(file) = db
            .symbol_defining_file(fqcn.as_ref())
            .map(|f| f.as_ref().to_string())
        else {
            continue;
        };

        let Some(node) = db.lookup_class_node(fqcn.as_ref()) else {
            continue;
        };
        if let Some(parent) = node.parent(db) {
            add_edge(parent.as_ref(), &file);
        }
        for iface in node.interfaces(db).iter() {
            add_edge(iface.as_ref(), &file);
        }
        for tr in node.traits(db).iter() {
            add_edge(tr.as_ref(), &file);
        }

        // Add types from properties
        for prop in db.class_own_properties(fqcn.as_ref()).iter() {
            if let Some(ty) = prop.ty(db) {
                for named in extract_named_objects(&ty) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }

        // Add types from methods
        for method in db.class_own_methods(fqcn.as_ref()).iter() {
            // Parameter types
            for param in method.params(db).iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_edge(named.as_ref(), &file);
                    }
                }
            }
            // Return type
            if let Some(rt) = method.return_type(db) {
                for named in extract_named_objects(rt.as_ref()) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
    }

    // Add types from global functions
    for fqn in db.active_function_node_fqns() {
        let Some(node) = db.lookup_function_node(fqn.as_ref()) else {
            continue;
        };
        let Some(file) = db
            .symbol_defining_file(fqn.as_ref())
            .map(|f| f.as_ref().to_string())
        else {
            continue;
        };

        // Parameter types
        for param in node.params(db).iter() {
            if let Some(ty) = &param.ty {
                for named in extract_named_objects(ty.as_ref()) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
        // Return type
        if let Some(rt) = node.return_type(db) {
            for named in extract_named_objects(rt.as_ref()) {
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
