/// Project-level orchestration: file discovery, pass 1, pass 2.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use std::collections::{HashMap, HashSet};

use crate::cache::{hash_content, AnalysisCache};
use crate::db::{class_ancestors, collect_file_definitions, MirDatabase, MirDb, SourceFile};
use crate::pass2::Pass2Driver;
use crate::php_version::PhpVersion;
use mir_codebase::Codebase;
use mir_issues::Issue;
use salsa::Setter as _;

use crate::collector::DefinitionCollector;

// Re-exports for downstream callers in this crate.
pub use crate::pass2::merge_return_types;

// ---------------------------------------------------------------------------
// ProjectAnalyzer
// ---------------------------------------------------------------------------

pub struct ProjectAnalyzer {
    pub codebase: Arc<Codebase>,
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

impl ProjectAnalyzer {
    pub fn new() -> Self {
        Self {
            codebase: Arc::new(Codebase::new()),
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
            codebase: Arc::new(Codebase::new()),
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
            codebase: Arc::new(Codebase::new()),
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

    /// Expose codebase for external use (e.g., pre-loading stubs from CLI).
    pub fn codebase(&self) -> &Arc<Codebase> {
        &self.codebase
    }

    /// Internal: expose the salsa Mutex for unit tests that need a `&dyn MirDatabase`.
    #[doc(hidden)]
    pub fn salsa_db_for_test(&self) -> &std::sync::Mutex<(MirDb, HashMap<Arc<str>, SourceFile>)> {
        &self.salsa
    }

    /// Load PHP built-in stubs. Called automatically by `analyze` if not done yet.
    /// Stubs are filtered against the configured target PHP version (or
    /// `PhpVersion::LATEST` if none was set).
    pub fn load_stubs(&self) {
        if !self
            .stubs_loaded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            crate::stubs::load_stubs_for_version(&self.codebase, self.resolved_php_version());
            crate::stubs::load_user_stubs(&self.codebase, &self.stub_files, &self.stub_dirs);
            // S5-PR8: mirror the loaded stubs into the Salsa db so
            // `type_exists_via_db` / `class_kind_via_db` / `class_template_params_via_db`
            // see them.
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.ingest_codebase(&self.codebase);
        }
    }

    /// Run the full analysis pipeline on a set of file paths.
    pub fn analyze(&self, paths: &[PathBuf]) -> AnalysisResult {
        let mut all_issues = Vec::new();
        let mut parse_errors = Vec::new();

        // ---- Load PHP built-in stubs (before Pass 1 so user code can override)
        self.load_stubs();

        // ---- Pass 1: read files in parallel ----------------------------------
        let file_data: Vec<(Arc<str>, String)> = paths
            .par_iter()
            .filter_map(|path| match std::fs::read_to_string(path) {
                Ok(src) => Some((Arc::from(path.to_string_lossy().as_ref()), src)),
                Err(e) => {
                    eprintln!("Cannot read {}: {}", path.display(), e);
                    None
                }
            })
            .collect();

        // ---- Pre-Pass-2 invalidation: evict dependents of changed files ------
        if let Some(cache) = &self.cache {
            let changed: Vec<String> = file_data
                .par_iter()
                .filter_map(|(f, src)| {
                    let h = hash_content(src);
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

        // ---- Pass 1: combined pre-index + definition collection (parallel) -----
        let pass1_results: Vec<(Vec<Issue>, Vec<Issue>)> = file_data
            .par_iter()
            .map(|(file, src)| {
                use php_ast::ast::StmtKind;
                let arena = bumpalo::Bump::new();
                let result = php_rs_parser::parse(&arena, src);

                // --- Pre-index: build FQCN index, file imports, and namespaces ---
                let mut current_namespace: Option<String> = None;
                let mut imports: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                let mut file_ns_set = false;

                let index_stmts =
                    |stmts: &[php_ast::ast::Stmt<'_, '_>],
                     ns: Option<&str>,
                     imports: &mut std::collections::HashMap<String, String>| {
                        for stmt in stmts.iter() {
                            match &stmt.kind {
                                StmtKind::Use(use_decl) => {
                                    for item in use_decl.uses.iter() {
                                        let full_name = crate::parser::name_to_string(&item.name)
                                            .trim_start_matches('\\')
                                            .to_string();
                                        let alias = item.alias.unwrap_or_else(|| {
                                            full_name.rsplit('\\').next().unwrap_or(&full_name)
                                        });
                                        imports.insert(alias.to_string(), full_name);
                                    }
                                }
                                StmtKind::Class(decl) => {
                                    if let Some(n) = decl.name {
                                        let fqcn = match ns {
                                            Some(ns) => format!("{ns}\\{n}"),
                                            None => n.to_string(),
                                        };
                                        self.codebase
                                            .known_symbols
                                            .insert(Arc::from(fqcn.as_str()));
                                    }
                                }
                                StmtKind::Interface(decl) => {
                                    let fqcn = match ns {
                                        Some(ns) => format!("{}\\{}", ns, decl.name),
                                        None => decl.name.to_string(),
                                    };
                                    self.codebase.known_symbols.insert(Arc::from(fqcn.as_str()));
                                }
                                StmtKind::Trait(decl) => {
                                    let fqcn = match ns {
                                        Some(ns) => format!("{}\\{}", ns, decl.name),
                                        None => decl.name.to_string(),
                                    };
                                    self.codebase.known_symbols.insert(Arc::from(fqcn.as_str()));
                                }
                                StmtKind::Enum(decl) => {
                                    let fqcn = match ns {
                                        Some(ns) => format!("{}\\{}", ns, decl.name),
                                        None => decl.name.to_string(),
                                    };
                                    self.codebase.known_symbols.insert(Arc::from(fqcn.as_str()));
                                }
                                StmtKind::Function(decl) => {
                                    let fqn = match ns {
                                        Some(ns) => format!("{}\\{}", ns, decl.name),
                                        None => decl.name.to_string(),
                                    };
                                    self.codebase.known_symbols.insert(Arc::from(fqn.as_str()));
                                }
                                _ => {}
                            }
                        }
                    };

                for stmt in result.program.stmts.iter() {
                    match &stmt.kind {
                        StmtKind::Namespace(ns) => {
                            current_namespace =
                                ns.name.as_ref().map(|n| crate::parser::name_to_string(n));
                            if !file_ns_set {
                                if let Some(ref ns_str) = current_namespace {
                                    self.codebase
                                        .file_namespaces
                                        .insert(file.clone(), ns_str.clone());
                                    file_ns_set = true;
                                }
                            }
                            if let php_ast::ast::NamespaceBody::Braced(inner_stmts) = &ns.body {
                                index_stmts(
                                    inner_stmts,
                                    current_namespace.as_deref(),
                                    &mut imports,
                                );
                            }
                        }
                        _ => index_stmts(
                            std::slice::from_ref(stmt),
                            current_namespace.as_deref(),
                            &mut imports,
                        ),
                    }
                }

                if !imports.is_empty() {
                    self.codebase.file_imports.insert(file.clone(), imports);
                }

                // --- Parse errors ---
                let file_parse_errors: Vec<Issue> = result
                    .errors
                    .iter()
                    .map(|err| {
                        Issue::new(
                            mir_issues::IssueKind::ParseError {
                                message: err.to_string(),
                            },
                            mir_issues::Location {
                                file: file.clone(),
                                line: 1,
                                line_end: 1,
                                col_start: 0,
                                col_end: 0,
                            },
                        )
                    })
                    .collect();

                // --- Definition collection ---
                let collector =
                    DefinitionCollector::new(&self.codebase, file.clone(), src, &result.source_map);
                let issues = collector.collect(&result.program);

                (file_parse_errors, issues)
            })
            .collect();

        let mut files_with_parse_errors: std::collections::HashSet<Arc<str>> =
            std::collections::HashSet::new();
        for (file_parse_errors, issues) in pass1_results {
            for issue in &file_parse_errors {
                files_with_parse_errors.insert(issue.location.file.clone());
            }
            parse_errors.extend(file_parse_errors);
            all_issues.extend(issues);
        }

        all_issues.extend(parse_errors);

        // ---- Lazy-load unknown classes via PSR-4 (issue #50) ----------------
        if let Some(psr4) = &self.psr4 {
            self.lazy_load_missing_classes(psr4.clone(), &mut all_issues);
        }

        // ---- Compute Codebase.all_parents for all classes/interfaces.
        // Pass 2 reads `cls.all_parents` via `has_magic_get`, `has_unknown_ancestor`,
        // `get_member_location`, and `get_inherited_template_bindings` — none of
        // those walk inheritance lazily anymore (`ensure_finalized` was removed
        // in S5-PR38), so the global walk has to run before Pass 2 starts.
        self.codebase.finalize();

        // ---- S5-PR9: mirror Pass 1 + lazy-loaded definitions into the Salsa
        // db.  Today the batch Pass 2 driver still passes `db: None`, so this
        // is preparatory — the db is populated and ready for the per-helper
        // fallback removal that follows once `Pass2Driver` is wired with a
        // shared db reference.
        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            guard.0.ingest_codebase(&self.codebase);
        }

        // ---- Build reverse dep graph and persist it for the next run ---------
        if let Some(cache) = &self.cache {
            let db_snapshot = {
                let guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.clone()
            };
            let rev = build_reverse_deps(&self.codebase, &db_snapshot);
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
            let class_issues = crate::class::ClassAnalyzer::with_files(
                &self.codebase,
                &class_db,
                analyzed_file_set,
                &file_data,
            )
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
        file_data
            .par_iter()
            .filter(|(file, _)| !files_with_parse_errors.contains(file))
            .for_each_with(db_priming, |db, (file, src)| {
                let driver = Pass2Driver::new_inference_only(
                    &self.codebase,
                    &*db as &dyn MirDatabase,
                    self.resolved_php_version(),
                )
                .with_inferred_buffer(&inferred_buffer);
                let arena = bumpalo::Bump::new();
                let parsed = php_rs_parser::parse(&arena, src);
                driver.analyze_bodies(&parsed.program, file.clone(), src, &parsed.source_map);
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
        let pass2_results: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = file_data
            .par_iter()
            .filter(|(file, _)| !files_with_parse_errors.contains(file))
            .map_with(db_main, |db, (file, src)| {
                let driver = Pass2Driver::new(
                    &self.codebase,
                    &*db as &dyn MirDatabase,
                    self.resolved_php_version(),
                );
                let result = if let Some(cache) = &self.cache {
                    let h = hash_content(src);
                    if let Some((cached_issues, ref_locs)) = cache.get(file, &h) {
                        self.codebase
                            .replay_reference_locations(file.clone(), &ref_locs);
                        (cached_issues, Vec::new())
                    } else {
                        let arena = bumpalo::Bump::new();
                        let parsed = php_rs_parser::parse(&arena, src);
                        let (issues, symbols) = driver.analyze_bodies(
                            &parsed.program,
                            file.clone(),
                            src,
                            &parsed.source_map,
                        );
                        let ref_locs = extract_reference_locations(&self.codebase, file);
                        cache.put(file, h, issues.clone(), ref_locs);
                        (issues, symbols)
                    }
                } else {
                    let arena = bumpalo::Bump::new();
                    let parsed = php_rs_parser::parse(&arena, src);
                    driver.analyze_bodies(&parsed.program, file.clone(), src, &parsed.source_map)
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
        self.codebase.compact_reference_index();

        // ---- Dead-code detection (M18) --------------------------------------
        if self.find_dead_code {
            let salsa = self.salsa.lock().unwrap();
            let dead_code_issues =
                crate::dead_code::DeadCodeAnalyzer::new(&self.codebase, &salsa.0).analyze();
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
                if !self.codebase.type_exists(fqcn) && !loaded.contains(fqcn) {
                    if let Some(path) = psr4.resolve(fqcn) {
                        to_load.push((fqcn.to_string(), path));
                    }
                }
            };

            // Iterate `Codebase` directly (not the salsa db).  Newly lazy-loaded
            // classes are added to `Codebase` by `DefinitionCollector::collect`
            // below but aren't upserted to the salsa db until after the lazy-load
            // loop finishes (`ingest_codebase` runs after this method returns).
            // Iterating the db here would miss classes loaded in earlier
            // iterations of this max-depth loop, breaking transitive ancestor
            // discovery (see `psr4_trait_fqcn_lazy_loaded` fixture).
            for entry in self.codebase.classes.iter() {
                let cls = entry.value();
                if let Some(parent) = &cls.parent {
                    try_queue(parent.as_ref());
                }
                for iface in &cls.interfaces {
                    try_queue(iface.as_ref());
                }
            }
            for entry in self.codebase.interfaces.iter() {
                for parent in &entry.value().extends {
                    try_queue(parent.as_ref());
                }
            }
            for entry in self.codebase.enums.iter() {
                for iface in &entry.value().interfaces {
                    try_queue(iface.as_ref());
                }
            }
            for entry in self.codebase.traits.iter() {
                for used in &entry.value().traits {
                    try_queue(used.as_ref());
                }
            }

            // Also lazy-load any type referenced via `use` imports that isn't yet
            // in the codebase (covers enums and classes used only in type hints or
            // static calls, which never appear in the inheritance scan above).
            for entry in self.codebase.file_imports.iter() {
                for fqcn in entry.value().values() {
                    try_queue(fqcn.as_str());
                }
            }

            if to_load.is_empty() {
                break;
            }

            for (fqcn, path) in to_load {
                loaded.insert(fqcn);
                if let Ok(src) = std::fs::read_to_string(&path) {
                    let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                    let arena = bumpalo::Bump::new();
                    let result = php_rs_parser::parse(&arena, &src);
                    let collector = crate::collector::DefinitionCollector::new(
                        &self.codebase,
                        file,
                        &src,
                        &result.source_map,
                    );
                    let issues = collector.collect(&result.program);
                    all_issues.extend(issues);
                }
            }

            self.codebase.invalidate_finalization();
            self.codebase.finalize();
        }
    }

    fn lazy_load_from_body_issues(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        file_data: &[(Arc<str>, String)],
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
                    if !self.codebase.type_exists(name) && !loaded.contains(name) {
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
                    let arena = bumpalo::Bump::new();
                    let result = php_rs_parser::parse(&arena, &src);
                    let collector = crate::collector::DefinitionCollector::new(
                        &self.codebase,
                        file,
                        &src,
                        &result.source_map,
                    );
                    let _ = collector.collect(&result.program);
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
                        if self.codebase.type_exists(name) {
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

            // S5-PR11a: mirror newly-loaded definitions into the salsa db
            // before re-analyzing, so the cloned db each rayon worker
            // receives sees them.
            let db_reanalysis = {
                let mut guard = self.salsa.lock().expect("salsa lock poisoned");
                guard.0.ingest_codebase(&self.codebase);
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
                    let driver = Pass2Driver::new(
                        &self.codebase,
                        &*db as &dyn MirDatabase,
                        self.resolved_php_version(),
                    )
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
    /// 3. Re-finalizes the codebase (rebuilds inheritance)
    /// 4. Re-runs Pass 2 (body analysis) on this file
    /// 5. Returns the analysis result for this file only
    pub fn re_analyze_file(&self, file_path: &str, new_content: &str) -> AnalysisResult {
        // Fast path: content unchanged and cache has a valid entry — skip full re-analysis.
        if let Some(cache) = &self.cache {
            let h = hash_content(new_content);
            if let Some((issues, ref_locs)) = cache.get(file_path, &h) {
                let file: Arc<str> = Arc::from(file_path);
                self.codebase.replay_reference_locations(file, &ref_locs);
                return AnalysisResult::build(issues, HashMap::new(), Vec::new());
            }
        }

        let file: Arc<str> = Arc::from(file_path);

        // --- S2: Capture old ancestors and mark old ClassNodes inactive --------
        // Collect FQCNs defined in this file before they are removed, then
        // record their current ancestor lists so we can detect structural
        // changes after re-running Pass 1.
        //
        // Priority: Salsa-memoized ancestors (warm path) > Codebase.all_parents
        // (cold path, first LSP edit for this file).
        let old_fqcns: Vec<Arc<str>> = self
            .codebase
            .symbol_to_file
            .iter()
            .filter(|e| e.value().as_ref() == file_path)
            .map(|e| e.key().clone())
            .collect();

        // Only track ancestry for classes and interfaces (not functions, traits,
        // enums, or constants — none of those participate in the inheritance graph).
        let old_ancestors: HashMap<Arc<str>, Vec<Arc<str>>> = {
            let guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref db, _) = *guard;
            old_fqcns
                .iter()
                .filter(|fqcn| {
                    crate::db::class_kind_via_db(db, fqcn.as_ref())
                        .is_some_and(|k| !k.is_trait && !k.is_enum)
                })
                .map(|fqcn| {
                    let salsa_ancs = db.lookup_class_node(fqcn).map(|n| class_ancestors(db, n).0);
                    let ancs = salsa_ancs.unwrap_or_else(|| {
                        // Cold path: use Codebase data as ground truth.
                        self.codebase
                            .classes
                            .get(fqcn.as_ref())
                            .map(|c| c.all_parents.clone())
                            .or_else(|| {
                                self.codebase
                                    .interfaces
                                    .get(fqcn.as_ref())
                                    .map(|i| i.all_parents.clone())
                            })
                            .unwrap_or_default()
                    });
                    (fqcn.clone(), ancs)
                })
                .collect()
        };

        // Mark removed classes, functions, methods, properties, and constants inactive.
        {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, _) = *guard;
            for fqcn in &old_fqcns {
                db.deactivate_class_node(fqcn);
                db.deactivate_function_node(fqcn);
                db.deactivate_class_methods(fqcn);
                db.deactivate_class_properties(fqcn);
                db.deactivate_class_constants(fqcn);
            }
        }

        self.codebase.remove_file_definitions(file_path);

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

        let mut all_issues: Vec<Issue> = (*file_defs.issues).clone();
        self.codebase.inject_stub_slice((*file_defs.slice).clone());

        // --- S2 + Pass 2: hold the Salsa lock for ClassNode upserts and body
        // analysis so the db reference is live during Pass 2 (S5).
        let symbols = {
            let mut guard = self.salsa.lock().expect("salsa lock poisoned");
            let (ref mut db, _) = *guard;

            // --- S2 + S5-PR5a: Upsert ClassNodes for all type kinds.  Traits and
            // enums are registered with empty ancestor data — `class_ancestors`
            // returns empty for them, matching `Codebase::ensure_finalized`.
            for cls in &file_defs.slice.classes {
                db.upsert_class_node(crate::db::ClassNodeFields {
                    is_abstract: cls.is_abstract,
                    parent: cls.parent.clone(),
                    interfaces: Arc::from(cls.interfaces.as_slice()),
                    traits: Arc::from(cls.traits.as_slice()),
                    template_params: Arc::from(cls.template_params.as_slice()),
                    mixins: Arc::from(cls.mixins.as_slice()),
                    deprecated: cls.deprecated.clone(),
                    is_final: cls.is_final,
                    is_readonly: cls.is_readonly,
                    location: cls.location.clone(),
                    extends_type_args: Arc::from(cls.extends_type_args.as_slice()),
                    implements_type_args: Arc::from(
                        cls.implements_type_args
                            .iter()
                            .map(|(iface, args)| (iface.clone(), Arc::from(args.as_slice())))
                            .collect::<Vec<_>>(),
                    ),
                    ..crate::db::ClassNodeFields::for_class(cls.fqcn.clone())
                });
            }
            for iface in &file_defs.slice.interfaces {
                db.upsert_class_node(crate::db::ClassNodeFields {
                    extends: Arc::from(iface.extends.as_slice()),
                    template_params: Arc::from(iface.template_params.as_slice()),
                    location: iface.location.clone(),
                    ..crate::db::ClassNodeFields::for_interface(iface.fqcn.clone())
                });
            }
            for tr in &file_defs.slice.traits {
                db.upsert_class_node(crate::db::ClassNodeFields {
                    traits: Arc::from(tr.traits.as_slice()),
                    template_params: Arc::from(tr.template_params.as_slice()),
                    require_extends: Arc::from(tr.require_extends.as_slice()),
                    require_implements: Arc::from(tr.require_implements.as_slice()),
                    location: tr.location.clone(),
                    ..crate::db::ClassNodeFields::for_trait(tr.fqcn.clone())
                });
            }
            for en in &file_defs.slice.enums {
                db.upsert_class_node(crate::db::ClassNodeFields {
                    interfaces: Arc::from(en.interfaces.as_slice()),
                    is_backed_enum: en.scalar_type.is_some(),
                    enum_scalar_type: en.scalar_type.clone(),
                    location: en.location.clone(),
                    ..crate::db::ClassNodeFields::for_enum(en.fqcn.clone())
                });
            }

            // --- S5-PR2: Upsert FunctionNodes ------------------------------------
            for func in &file_defs.slice.functions {
                db.upsert_function_node(func);
            }

            // --- S5-PR47: Upsert GlobalConstantNodes ------------------------------
            for (fqn, ty) in &file_defs.slice.constants {
                db.upsert_global_constant_node(fqn.clone(), ty.clone());
            }

            // --- S5-PR3: Upsert MethodNodes for all type members ------------------
            for cls in &file_defs.slice.classes {
                for method in cls.own_methods.values() {
                    db.upsert_method_node(method);
                }
            }
            for iface in &file_defs.slice.interfaces {
                for method in iface.own_methods.values() {
                    db.upsert_method_node(method);
                }
            }
            for tr in &file_defs.slice.traits {
                for method in tr.own_methods.values() {
                    db.upsert_method_node(method);
                }
            }
            for en in &file_defs.slice.enums {
                for method in en.own_methods.values() {
                    db.upsert_method_node(method);
                }
            }

            // --- S5-PR4: Upsert PropertyNodes and ClassConstantNodes --------------
            for cls in &file_defs.slice.classes {
                for prop in cls.own_properties.values() {
                    db.upsert_property_node(&cls.fqcn, prop);
                }
                for constant in cls.own_constants.values() {
                    db.upsert_class_constant_node(&cls.fqcn, constant);
                }
            }
            for iface in &file_defs.slice.interfaces {
                for constant in iface.own_constants.values() {
                    db.upsert_class_constant_node(&iface.fqcn, constant);
                }
            }
            for tr in &file_defs.slice.traits {
                for prop in tr.own_properties.values() {
                    db.upsert_property_node(&tr.fqcn, prop);
                }
                for constant in tr.own_constants.values() {
                    db.upsert_class_constant_node(&tr.fqcn, constant);
                }
            }
            for en in &file_defs.slice.enums {
                for constant in en.own_constants.values() {
                    db.upsert_class_constant_node(&en.fqcn, constant);
                }
            }

            let new_ancestors: HashMap<Arc<str>, Vec<Arc<str>>> = {
                let mut map = HashMap::new();
                for cls in &file_defs.slice.classes {
                    if let Some(node) = db.lookup_class_node(&cls.fqcn) {
                        map.insert(cls.fqcn.clone(), class_ancestors(db, node).0);
                    }
                }
                for iface in &file_defs.slice.interfaces {
                    if let Some(node) = db.lookup_class_node(&iface.fqcn) {
                        map.insert(iface.fqcn.clone(), class_ancestors(db, node).0);
                    }
                }
                map
            };

            // --- S2: Decide whether ancestry changed and update Codebase ------
            let structural_unchanged = old_ancestors.len() == new_ancestors.len()
                && new_ancestors
                    .iter()
                    .all(|(fqcn, new_ancs)| old_ancestors.get(fqcn) == Some(new_ancs));

            if structural_unchanged {
                // Fast path: restore ancestors from Salsa results directly.
                for (fqcn, ancestors) in &new_ancestors {
                    let arc: Arc<[Arc<str>]> = Arc::from(ancestors.as_slice());
                    self.codebase.restore_ancestors(fqcn, arc);
                }
                self.codebase.mark_finalized();
            } else {
                self.codebase.invalidate_finalization();
                self.codebase.finalize();
            }

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
                    Pass2Driver::new_inference_only(
                        &self.codebase,
                        db_ref,
                        self.resolved_php_version(),
                    )
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
                let driver = Pass2Driver::new(&self.codebase, db_ref, self.resolved_php_version());
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
            let ref_locs = extract_reference_locations(&self.codebase, &file);
            cache.put(file_path, h, all_issues.clone(), ref_locs);
        }

        AnalysisResult::build(all_issues, HashMap::new(), symbols)
    }

    /// Analyze a PHP source string without a real file path.
    /// Useful for tests and LSP single-file mode.
    pub fn analyze_source(source: &str) -> AnalysisResult {
        use crate::collector::DefinitionCollector;
        let analyzer = ProjectAnalyzer::new();
        analyzer.load_stubs();
        let file: Arc<str> = Arc::from("<source>");
        let arena = bumpalo::Bump::new();
        let result = php_rs_parser::parse(&arena, source);
        let mut all_issues = Vec::new();
        for err in &result.errors {
            all_issues.push(Issue::new(
                mir_issues::IssueKind::ParseError {
                    message: err.to_string(),
                },
                mir_issues::Location {
                    file: file.clone(),
                    line: 1,
                    line_end: 1,
                    col_start: 0,
                    col_end: 0,
                },
            ));
        }
        if !result.errors.is_empty() {
            return AnalysisResult::build(all_issues, std::collections::HashMap::new(), Vec::new());
        }
        let collector =
            DefinitionCollector::new(&analyzer.codebase, file.clone(), source, &result.source_map);
        all_issues.extend(collector.collect(&result.program));
        analyzer.codebase.finalize();
        let mut type_envs = std::collections::HashMap::new();
        let mut all_symbols = Vec::new();
        // Build a db that mirrors the just-collected definitions so the
        // analyzers' db reads see them.
        let mut db = MirDb::default();
        db.ingest_codebase(&analyzer.codebase);
        let driver = Pass2Driver::new(&analyzer.codebase, &db, analyzer.resolved_php_version());
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
        paths.par_iter().for_each(|path| {
            let Ok(src) = std::fs::read_to_string(path) else {
                return;
            };
            let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
            let arena = bumpalo::Bump::new();
            let result = php_rs_parser::parse(&arena, &src);
            let collector =
                DefinitionCollector::new(&self.codebase, file, &src, &result.source_map);
            let _ = collector.collect(&result.program);
        });
    }
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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

fn build_reverse_deps(
    codebase: &Codebase,
    db: &dyn crate::db::MirDatabase,
) -> HashMap<String, HashSet<String>> {
    let mut reverse: HashMap<String, HashSet<String>> = HashMap::new();

    let mut add_edge = |symbol: &str, dependent_file: &str| {
        if let Some(defining_file) = codebase.symbol_to_file.get(symbol) {
            let def = defining_file.as_ref().to_string();
            if def != dependent_file {
                reverse
                    .entry(def)
                    .or_default()
                    .insert(dependent_file.to_string());
            }
        }
    };

    for entry in codebase.file_imports.iter() {
        let file = entry.key().as_ref().to_string();
        for fqcn in entry.value().values() {
            add_edge(fqcn, &file);
        }
    }

    for fqcn in db.active_class_node_fqcns() {
        // Match `Codebase::classes` semantics: only true classes contribute
        // class-direction edges in this loop.  Interface / trait / enum edges
        // are handled by their own dedicated codebase iterators elsewhere if
        // needed (none currently — this function only ever read classes).
        let kind = match crate::db::class_kind_via_db(db, fqcn.as_ref()) {
            Some(k) if !k.is_interface && !k.is_trait && !k.is_enum => k,
            _ => continue,
        };
        let _ = kind;
        let Some(file) = codebase
            .symbol_to_file
            .get(fqcn.as_ref())
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
    codebase: &Codebase,
    file: &Arc<str>,
) -> Vec<(String, u32, u16, u16)> {
    codebase
        .extract_file_reference_locations(file.as_ref())
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
