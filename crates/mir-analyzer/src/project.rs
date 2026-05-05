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
use crate::pass2::Pass2Driver;
use crate::php_version::PhpVersion;
use mir_issues::Issue;
use salsa::Setter as _;

// Re-exports for downstream callers in this crate.
pub use crate::pass2::merge_return_types;

// ---------------------------------------------------------------------------
// ProjectAnalyzer
// ---------------------------------------------------------------------------

pub struct ProjectAnalyzer {
    /// Optional cache — when `Some`, Pass 2 results are read/written per file.
    pub cache: Option<AnalysisCache>,
    /// Called once after each file completes Pass 2 (used for progress reporting).
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    /// PSR-4 autoloader mapping from composer.json, if available.
    pub psr4: Option<Arc<crate::composer::Psr4Map>>,
    /// Whether stubs have already been loaded (to avoid double-loading).
    stubs_loaded: std::sync::atomic::AtomicBool,
    /// When true, run dead code detection at the end of analysis.
    pub find_dead_code: bool,
    /// Target PHP language version. `None` means "not configured"; resolved to
    /// `PhpVersion::LATEST` when passed down to `StatementsAnalyzer`.
    pub php_version: Option<PhpVersion>,
    /// Additional stub files to parse before analysis (absolute paths).
    pub stub_files: Vec<PathBuf>,
    /// Additional stub directories to walk and parse before analysis (absolute paths).
    pub stub_dirs: Vec<PathBuf>,
    /// Salsa database for incremental Pass-1 memoization.
    /// `MirDb` is `Send` but `!Sync` (thread-local query state); `Mutex`
    /// provides the `Sync` bound rayon requires without needing `T: Sync`.
    salsa: std::sync::Mutex<(MirDb, HashMap<Arc<str>, SourceFile>)>,
}

struct ParsedProjectFile {
    file: Arc<str>,
    source: Arc<str>,
    parsed: ManuallyDrop<php_rs_parser::ParseResult<'static, 'static>>,
    arena: ManuallyDrop<Box<bumpalo::Bump>>,
}

impl ParsedProjectFile {
    fn new(file: Arc<str>, source: Arc<str>) -> Self {
        let arena = Box::new(bumpalo::Bump::new());
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
            cache: None,
            on_file_done: None,
            psr4: None,
            stubs_loaded: std::sync::atomic::AtomicBool::new(false),
            find_dead_code: false,
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
            salsa: std::sync::Mutex::new((MirDb::default(), HashMap::new())),
        }
    }

    /// Create a `ProjectAnalyzer` with a disk-backed cache stored under `cache_dir`.
    pub fn with_cache(cache_dir: &Path) -> Self {
        Self {
            cache: Some(AnalysisCache::open(cache_dir)),
            on_file_done: None,
            psr4: None,
            stubs_loaded: std::sync::atomic::AtomicBool::new(false),
            find_dead_code: false,
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
            salsa: std::sync::Mutex::new((MirDb::default(), HashMap::new())),
        }
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
            cache: None,
            on_file_done: None,
            psr4: Some(psr4),
            stubs_loaded: std::sync::atomic::AtomicBool::new(false),
            find_dead_code: false,
            php_version: None,
            stub_files: Vec::new(),
            stub_dirs: Vec::new(),
            salsa: std::sync::Mutex::new((MirDb::default(), HashMap::new())),
        };
        Ok((analyzer, map))
    }

    /// Set the target PHP version.
    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version = Some(version);
        self
    }

    /// Resolve the configured PHP version, defaulting to `PhpVersion::LATEST`
    /// when none has been set.
    fn resolved_php_version(&self) -> PhpVersion {
        self.php_version.unwrap_or(PhpVersion::LATEST)
    }

    fn type_exists(&self, fqcn: &str) -> bool {
        let guard = self.salsa.lock().expect("salsa lock poisoned");
        crate::db::type_exists_via_db(&guard.0, fqcn)
    }

    /// Internal: expose the salsa Mutex for unit tests that need a `&dyn MirDatabase`.
    #[doc(hidden)]
    pub fn salsa_db_for_test(&self) -> &std::sync::Mutex<(MirDb, HashMap<Arc<str>, SourceFile>)> {
        &self.salsa
    }

    /// Look up the source location of a class member (method, property, or
    /// class constant / enum case) by walking the inheritance chain through
    /// the salsa db.  Returns `None` if no member with that name exists, or
    /// if the member has no recorded location.
    pub fn member_location(
        &self,
        fqcn: &str,
        member_name: &str,
    ) -> Option<mir_codebase::storage::Location> {
        let guard = self.salsa.lock().expect("salsa lock poisoned");
        crate::db::member_location_via_db(&guard.0, fqcn, member_name)
    }

    pub fn symbol_location(&self, symbol: &str) -> Option<mir_codebase::storage::Location> {
        let guard = self.salsa.lock().expect("salsa lock poisoned");
        let db = &guard.0;
        db.lookup_class_node(symbol)
            .filter(|n| n.active(db))
            .and_then(|n| n.location(db))
            .or_else(|| {
                db.lookup_function_node(symbol)
                    .filter(|n| n.active(db))
                    .and_then(|n| n.location(db))
            })
    }

    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let guard = self.salsa.lock().expect("salsa lock poisoned");
        guard.0.reference_locations(symbol)
    }

    /// Load PHP built-in stubs. Called automatically by `analyze` if not done yet.
    /// Stubs are filtered against the configured target PHP version (or
    /// `PhpVersion::LATEST` if none was set).
    pub fn load_stubs(&self) {
        if !self
            .stubs_loaded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            let php_version = self.resolved_php_version();
            crate::stubs::stub_files()
                .par_iter()
                .for_each(|(filename, content)| {
                    let slice =
                        crate::stubs::stub_slice_from_source(filename, content, Some(php_version));
                    let mut guard = self.salsa.lock().expect("salsa lock poisoned");
                    guard.0.ingest_stub_slice(&slice);
                });

            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            for slice in crate::stubs::user_stub_slices(&self.stub_files, &self.stub_dirs) {
                guard.0.ingest_stub_slice(&slice);
            }
        }
    }

    fn collect_and_ingest_source(&self, file: Arc<str>, src: &str) -> FileDefinitions {
        let file_defs = {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, ref mut files) = *guard;
            let salsa_file = match files.get(&file) {
                Some(&sf) => {
                    if sf.text(db).as_ref() != src {
                        sf.set_text(db).to(Arc::from(src));
                    }
                    sf
                }
                None => {
                    let sf = SourceFile::new(db, file.clone(), Arc::from(src));
                    files.insert(file.clone(), sf);
                    sf
                }
            };
            collect_file_definitions(db, salsa_file)
        };

        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.ingest_stub_slice(&file_defs.slice);
        }
        file_defs
    }

    /// Run the full analysis pipeline on a set of file paths.
    pub fn analyze(&self, paths: &[PathBuf]) -> AnalysisResult {
        let mut all_issues = Vec::new();

        // ---- Load PHP built-in stubs (before Pass 1 so user code can override)
        self.load_stubs();

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
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, ref mut files) = *guard;
            for parsed in &parsed_files {
                match files.get(parsed.file.as_ref()) {
                    Some(&sf) => {
                        if sf.text(db).as_ref() != parsed.source() {
                            sf.set_text(db).to(parsed.source.clone());
                        }
                    }
                    None => {
                        let sf = SourceFile::new(db, parsed.file.clone(), parsed.source.clone());
                        files.insert(parsed.file.clone(), sf);
                    }
                }
            }
        }

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
                let (slice, collector_issues) = collector.collect_slice(&parse_result.program);
                all_issues.extend(collector_issues);
                FileDefinitions {
                    slice: Arc::new(slice),
                    issues: Arc::new(all_issues),
                }
            })
            .collect();

        let mut files_with_parse_errors: std::collections::HashSet<Arc<str>> =
            std::collections::HashSet::new();
        let mut files_needing_inference: std::collections::HashSet<Arc<str>> =
            std::collections::HashSet::new();
        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, _) = *guard;
            for defs in file_defs {
                for issue in defs.issues.iter() {
                    if matches!(issue.kind, mir_issues::IssueKind::ParseError { .. }) {
                        files_with_parse_errors.insert(issue.location.file.clone());
                    }
                }
                if stub_slice_needs_inference(&defs.slice) {
                    if let Some(file) = defs.slice.file.as_ref() {
                        files_needing_inference.insert(file.clone());
                    }
                }
                db.ingest_stub_slice(&defs.slice);
                all_issues.extend(Arc::unwrap_or_clone(defs.issues));
            }
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
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.clone()
            };
            let rev = build_reverse_deps(&db_snapshot);
            cache.set_reverse_deps(rev);
        }

        // ---- Class-level checks (M11) ----------------------------------------
        // `class_db` is scoped tightly: it must be dropped before the priming
        // sweep's `commit_inferred_return_types` call below, otherwise the
        // setter's `Storage::cancel_others` blocks waiting for this clone's
        // Arc to drop (strong-count==1 invariant).
        let analyzed_file_set: std::collections::HashSet<std::sync::Arc<str>> =
            file_data.iter().map(|(f, _)| f.clone()).collect();
        {
            let class_db = {
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.clone()
            };
            let class_issues =
                crate::class::ClassAnalyzer::with_files(&class_db, analyzed_file_set, &file_data)
                    .analyze_all();
            all_issues.extend(class_issues);
        }

        // ---- S5-PR10b: clone the salsa db once per parallel sweep so each
        // rayon worker gets its own clone (Salsa databases are `Send` but
        // `!Sync`; cloning shares the underlying memoization storage).
        let db_priming = {
            let guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.clone()
        };

        // ---- Pass 2 priming: populate inferred_return_type for all functions  --
        // Run a first inference-only sweep so that cross-file inferred return
        // types are available before the issue-emitting pass below (G6).
        //
        // Inferred types are also collected into a thread-safe buffer here and
        // committed to the Salsa db serially after the sweep returns.  Writing
        // setters from inside `for_each_with` would deadlock against
        // `Storage::cancel_others` (which waits for sibling worker clones to
        // drop); the post-sweep commit runs against the canonical db with
        // strong-count==1.  See `crate::db::InferredReturnTypes`.
        let inferred_buffer = crate::db::InferredReturnTypes::new();
        parsed_files
            .par_iter()
            .filter(|parsed| {
                !files_with_parse_errors.contains(&parsed.file)
                    && files_needing_inference.contains(&parsed.file)
            })
            .for_each_with(db_priming, |db, parsed| {
                let driver = Pass2Driver::new_inference_only(
                    &*db as &dyn MirDatabase,
                    self.resolved_php_version(),
                )
                .with_inferred_buffer(&inferred_buffer);
                let parse_result = parsed.parsed();
                driver.analyze_bodies(
                    &parse_result.program,
                    parsed.file.clone(),
                    parsed.source(),
                    &parse_result.source_map,
                );
            });

        // Sweep clones are dropped — commit inferred types into the Salsa db.
        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.commit_inferred_return_types(&inferred_buffer);
        }

        let db_main = {
            let guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.clone()
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
            let salsa = self.salsa.lock().unwrap();
            let dead_code_issues = crate::dead_code::DeadCodeAnalyzer::new(&salsa.0).analyze();
            drop(salsa);
            all_issues.extend(dead_code_issues);
        }

        AnalysisResult::build(all_issues, std::collections::HashMap::new(), all_symbols)
    }

    fn lazy_load_missing_classes(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        all_issues: &mut Vec<Issue>,
    ) {
        use std::collections::HashSet;

        let max_depth = 10;
        let mut loaded: HashSet<String> = HashSet::new();

        for _ in 0..max_depth {
            let mut to_load: Vec<(String, PathBuf)> = Vec::new();

            let mut try_queue = |fqcn: &str| {
                if !self.type_exists(fqcn) && !loaded.contains(fqcn) {
                    if let Some(path) = psr4.resolve(fqcn) {
                        to_load.push((fqcn.to_string(), path));
                    }
                }
            };

            // Drive the inheritance scan from already-ingested `ClassNode`s.
            let mut inheritance_candidates = Vec::new();
            let import_candidates = {
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                let db = &guard.0;
                for fqcn in db.active_class_node_fqcns() {
                    let Some(node) = db.lookup_class_node(&fqcn) else {
                        continue;
                    };
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

            let db_reanalysis = {
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.clone()
            };

            // Lazy-loaded files re-run Pass 2 to pick up the just-loaded
            // definitions; collect inferred return types for a serial commit
            // after the parallel sweep returns (same buffer-and-commit
            // pattern as the main batch priming sweep).
            let inferred_buffer = crate::db::InferredReturnTypes::new();
            let reanalysis: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = file_data
                .par_iter()
                .filter(|(f, _)| {
                    !files_with_parse_errors.contains(f) && files_to_reanalyze.contains(f)
                })
                .map_with(db_reanalysis, |db, (file, src)| {
                    let driver =
                        Pass2Driver::new(&*db as &dyn MirDatabase, self.resolved_php_version())
                            .with_inferred_buffer(&inferred_buffer);
                    let arena = bumpalo::Bump::new();
                    let parsed = php_rs_parser::parse(&arena, src);
                    driver.analyze_bodies(&parsed.program, file.clone(), src, &parsed.source_map)
                })
                .collect();

            {
                let mut guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.commit_inferred_return_types(&inferred_buffer);
            }

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
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.replay_reference_locations(file, &ref_locs);
                return AnalysisResult::build(issues, HashMap::new(), Vec::new());
            }
        }

        let file: Arc<str> = Arc::from(file_path);

        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, _) = *guard;
            db.remove_file_definitions(file_path);
        }

        // --- Salsa-backed Pass 1: memoized parse + definition collection ------
        let file_defs = {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, ref mut files) = *guard;
            let salsa_file = match files.get(&file) {
                Some(&sf) => {
                    sf.set_text(db).to(Arc::from(new_content));
                    sf
                }
                None => {
                    let sf = SourceFile::new(db, file.clone(), Arc::from(new_content));
                    files.insert(file.clone(), sf);
                    sf
                }
            };
            collect_file_definitions(db, salsa_file)
        };

        let mut all_issues: Vec<Issue> = Arc::unwrap_or_clone(file_defs.issues.clone());

        // --- S2 + Pass 2: hold the Salsa lock for ClassNode upserts and body
        // analysis so the db reference is live during Pass 2 (S5).
        let symbols = {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, _) = *guard;

            db.ingest_stub_slice(&file_defs.slice);

            // Resolve any newly-collected @psalm-import-type declarations so
            // Pass 2 reads the imported aliases out of `type_aliases`.
            // Re-parse in the arena so Pass 2 can walk the AST.
            let arena = bumpalo::Bump::new();
            let parsed = php_rs_parser::parse(&arena, new_content);

            if parsed.errors.is_empty() {
                // Priming sweep: populate inferred_return_type for this file's functions
                // before the issue-emitting pass so within-file cross-function calls see
                // the correct inferred return type rather than None.  The buffer +
                // commit pattern is overkill for the single-threaded LSP path but kept
                // for symmetry with the parallel batch path (and so the analyzer's
                // Salsa node reads see the inferred values).
                let inferred_buffer = crate::db::InferredReturnTypes::new();
                {
                    let db_ref: &dyn MirDatabase = db;
                    Pass2Driver::new_inference_only(db_ref, self.resolved_php_version())
                        .with_inferred_buffer(&inferred_buffer)
                        .analyze_bodies(
                            &parsed.program,
                            file.clone(),
                            new_content,
                            &parsed.source_map,
                        );
                }
                db.commit_inferred_return_types(&inferred_buffer);

                let db_ref: &dyn MirDatabase = db;
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
            let guard = self.salsa.lock().expect("salsa lock poisoned");
            let ref_locs = extract_reference_locations(&guard.0, &file);
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

        // Priming sweep: populate inferred_return_type on FunctionNode /
        // MethodNode before the issue-emitting pass so call sites see the
        // inferred values.  Single-threaded — no buffer / commit dance
        // needed in principle, but we use the same pattern for symmetry
        // with the parallel batch path.
        let inferred_buffer = crate::db::InferredReturnTypes::new();
        Pass2Driver::new_inference_only(&db, analyzer.resolved_php_version())
            .with_inferred_buffer(&inferred_buffer)
            .analyze_bodies(&result.program, file.clone(), source, &result.source_map);
        db.commit_inferred_return_types(&inferred_buffer);

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

        let source_files: Vec<SourceFile> = {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, ref mut files) = *guard;
            file_data
                .iter()
                .map(|(file, src)| match files.get(file) {
                    Some(&sf) => {
                        if sf.text(db).as_ref() != src.as_ref() {
                            sf.set_text(db).to(src.clone());
                        }
                        sf
                    }
                    None => {
                        let sf = SourceFile::new(db, file.clone(), src.clone());
                        files.insert(file.clone(), sf);
                        sf
                    }
                })
                .collect()
        };

        let db_pass1 = {
            let guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.clone()
        };

        let file_defs: Vec<FileDefinitions> = source_files
            .par_iter()
            .map_with(db_pass1, |db, salsa_file| {
                collect_file_definitions_uncached(&*db, *salsa_file)
            })
            .collect();

        let mut guard = self.salsa.lock().expect("salsa lock poisoned");
        let (ref mut db, _) = *guard;
        for defs in file_defs {
            db.ingest_stub_slice(&defs.slice);
        }
        drop(guard);
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------

fn stub_slice_needs_inference(slice: &mir_codebase::storage::StubSlice) -> bool {
    slice
        .functions
        .iter()
        .any(|func| func.return_type.is_none())
        || slice.classes.iter().any(|class| {
            class
                .own_methods
                .values()
                .any(|method| !method.is_abstract && method.return_type.is_none())
        })
        || slice.traits.iter().any(|tr| {
            tr.own_methods
                .values()
                .any(|method| !method.is_abstract && method.return_type.is_none())
        })
        || slice.enums.iter().any(|en| {
            en.own_methods
                .values()
                .any(|method| !method.is_abstract && method.return_type.is_none())
        })
}

// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// build_reverse_deps
// ---------------------------------------------------------------------------

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
    }

    reverse
}

// ---------------------------------------------------------------------------

fn extract_reference_locations(
    db: &dyn crate::db::MirDatabase,
    file: &Arc<str>,
) -> Vec<(String, u32, u16, u16)> {
    db.extract_file_reference_locations(file.as_ref())
        .into_iter()
        .map(|(sym, line, col_start, col_end)| (sym.to_string(), line, col_start, col_end))
        .collect()
}

// ---------------------------------------------------------------------------
// AnalysisResult
// ---------------------------------------------------------------------------

pub struct AnalysisResult {
    pub issues: Vec<Issue>,
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
