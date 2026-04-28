/// Project-level orchestration: file discovery, pass 1, pass 2.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use std::collections::{HashMap, HashSet};

use crate::cache::{hash_content, AnalysisCache};
use crate::pass2::Pass2Driver;
use crate::php_version::PhpVersion;
use mir_codebase::Codebase;
use mir_issues::Issue;

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

        // ---- Build reverse dep graph and persist it for the next run ---------
        if let Some(cache) = &self.cache {
            let rev = build_reverse_deps(&self.codebase);
            cache.set_reverse_deps(rev);
        }

        // ---- Class-level checks (M11) ----------------------------------------
        let analyzed_file_set: std::collections::HashSet<std::sync::Arc<str>> =
            file_data.iter().map(|(f, _)| f.clone()).collect();
        let class_issues =
            crate::class::ClassAnalyzer::with_files(&self.codebase, analyzed_file_set, &file_data)
                .analyze_all();
        all_issues.extend(class_issues);

        // ---- Pass 2 priming: populate inferred_return_type for all functions  --
        // Run a first inference-only sweep so that cross-file inferred return
        // types are available before the issue-emitting pass below (G6).
        file_data
            .par_iter()
            .filter(|(file, _)| !files_with_parse_errors.contains(file))
            .for_each(|(file, src)| {
                let driver =
                    Pass2Driver::new_inference_only(&self.codebase, self.resolved_php_version());
                let arena = bumpalo::Bump::new();
                let parsed = php_rs_parser::parse(&arena, src);
                driver.analyze_bodies(&parsed.program, file.clone(), src, &parsed.source_map);
            });

        // ---- Pass 2: analyze function/method bodies in parallel (M14) --------
        let pass2_results: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = file_data
            .par_iter()
            .filter(|(file, _)| !files_with_parse_errors.contains(file))
            .map(|(file, src)| {
                let driver = Pass2Driver::new(&self.codebase, self.resolved_php_version());
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
            let dead_code_issues =
                crate::dead_code::DeadCodeAnalyzer::new(&self.codebase).analyze();
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

            let reanalysis: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>)> = file_data
                .par_iter()
                .filter(|(f, _)| {
                    !files_with_parse_errors.contains(f) && files_to_reanalyze.contains(f)
                })
                .map(|(file, src)| {
                    let driver = Pass2Driver::new(&self.codebase, self.resolved_php_version());
                    let arena = bumpalo::Bump::new();
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

        let structural_snapshot = self.codebase.file_structural_snapshot(file_path);
        self.codebase.remove_file_definitions(file_path);

        let file: Arc<str> = Arc::from(file_path);
        let arena = bumpalo::Bump::new();
        let parsed = php_rs_parser::parse(&arena, new_content);

        let mut all_issues = Vec::new();

        for err in &parsed.errors {
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

        let collector = DefinitionCollector::new(
            &self.codebase,
            file.clone(),
            new_content,
            &parsed.source_map,
        );
        all_issues.extend(collector.collect(&parsed.program));

        if self
            .codebase
            .structural_unchanged_after_pass1(file_path, &structural_snapshot)
        {
            self.codebase
                .restore_all_parents(file_path, &structural_snapshot);
        } else {
            self.codebase.finalize();
        }

        let symbols = if parsed.errors.is_empty() {
            let driver = Pass2Driver::new(&self.codebase, self.resolved_php_version());
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
        let driver = Pass2Driver::new(&analyzer.codebase, analyzer.resolved_php_version());
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

fn build_reverse_deps(codebase: &Codebase) -> HashMap<String, HashSet<String>> {
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

    for entry in codebase.classes.iter() {
        let defining = {
            let fqcn = entry.key().as_ref();
            codebase
                .symbol_to_file
                .get(fqcn)
                .map(|f| f.as_ref().to_string())
        };
        let Some(file) = defining else { continue };

        let cls = entry.value();
        if let Some(ref parent) = cls.parent {
            add_edge(parent.as_ref(), &file);
        }
        for iface in &cls.interfaces {
            add_edge(iface.as_ref(), &file);
        }
        for tr in &cls.traits {
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
