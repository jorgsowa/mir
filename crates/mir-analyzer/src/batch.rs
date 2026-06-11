//! Batch-oriented project analysis on [`AnalysisSession`].
//!
//! This module hosts the multi-file orchestration that used to live on the
//! retired `ProjectAnalyzer`: parallel definition collection, lazy class loading, dead-code
//! sweep, reverse-dependency index, and the [`AnalysisResult`] return type.
//! Per-file (LSP) entry points stay on `AnalysisSession` itself in
//! `session.rs`.
//!
//! All methods are `impl AnalysisSession`; configuration that's only
//! meaningful for batch runs (issue suppressions, progress callback, optional
//! PHP version override) is grouped in [`BatchOptions`] and passed in rather
//! than stored on the session.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use mir_issues::Issue;
use mir_types::{Atomic, Type};

use crate::body_analysis::BodyAnalyzer;
use crate::cache::hash_content;
use crate::db::{
    collect_file_definitions, FileDefinitions, MirDatabase, MirDbStorage, RefLoc, SourceFile,
};
use crate::php_version::PhpVersion;
use crate::session::AnalysisSession;
use crate::stub_cache::{hash_source, prepare_for_ingest};

/// Issue kinds emitted by [`crate::dead_code::DeadCodeAnalyzer`].
///
/// The dead-code pass is just an error group — these names participate in
/// [`BatchOptions::suppressed_issue_kinds`] like any other `IssueKind`. If
/// every kind listed here is suppressed, the dead-code pass is skipped
/// entirely.
pub fn dead_code_issue_kinds() -> &'static [&'static str] {
    &["UnusedMethod", "UnusedProperty", "UnusedFunction"]
}

/// Per-batch options for [`AnalysisSession::analyze_paths`] and friends.
///
/// Configuration that only makes sense for full-project (batch) analysis
/// lives here instead of on [`AnalysisSession`], so the per-file LSP API
/// isn't bloated with state nothing else reads.
#[derive(Clone, Default)]
pub struct BatchOptions {
    /// Names of `IssueKind` variants to drop from the final result, e.g.
    /// `["MissingThrowsDocblock", "UnusedMethod"]`. Applied as a final
    /// post-filter so analyzer internals don't need to know which
    /// diagnostics the consumer cares about. Empty by default.
    pub suppressed_issue_kinds: HashSet<String>,
    /// Called once after each file completes body analysis (progress reporting).
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Override the session's configured PHP version for this run. `None`
    /// uses the session's version.
    pub php_version_override: Option<PhpVersion>,
}

impl BatchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_suppressed<I, S>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.suppressed_issue_kinds = kinds.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_progress_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_file_done = Some(callback);
        self
    }

    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version_override = Some(version);
        self
    }

    /// True iff at least one dead-code [`IssueKind`] would be emitted (i.e.
    /// not all of them are suppressed).
    fn should_run_dead_code(&self) -> bool {
        dead_code_issue_kinds()
            .iter()
            .any(|k| !self.suppressed_issue_kinds.contains(*k))
    }

    /// Drop issues whose [`IssueKind::name()`] is listed in
    /// [`Self::suppressed_issue_kinds`].
    fn apply(&self, issues: &mut Vec<Issue>) {
        if self.suppressed_issue_kinds.is_empty() {
            return;
        }
        issues.retain(|i| !self.suppressed_issue_kinds.contains(i.kind.name()));
    }
}

struct ParsedProjectFile {
    file: Arc<str>,
    source: Arc<str>,
    parsed: php_rs_parser::ParseResult,
}

impl ParsedProjectFile {
    fn new(file: Arc<str>, source: Arc<str>) -> Self {
        let parsed = php_rs_parser::parse(source.as_ref());
        Self {
            file,
            source,
            parsed,
        }
    }

    fn source(&self) -> &str {
        self.source.as_ref()
    }

    fn source_map(&self) -> &php_rs_parser::source_map::SourceMap {
        &self.parsed.source_map
    }

    fn errors(&self) -> &[php_rs_parser::diagnostics::ParseError] {
        &self.parsed.errors
    }

    fn owned(&self) -> &php_ast::owned::Program {
        &self.parsed.program
    }
}

impl AnalysisSession {
    /// Cumulative hit / miss counts on the persistent definition cache attached
    /// to this session. `(0, 0)` when no cache is configured.
    #[doc(hidden)]
    pub fn stub_cache_stats(&self) -> (u64, u64) {
        match self.db.stub_cache.as_deref() {
            Some(c) => (c.hits(), c.misses()),
            None => (0, 0),
        }
    }

    fn batch_php_version(&self, opts: &BatchOptions) -> PhpVersion {
        opts.php_version_override.unwrap_or(self.php_version)
    }

    /// Mark issues silenced by inline suppression comments
    /// (`@mir-ignore`, `@psalm-suppress`, `@phpstan-ignore*`, …) as suppressed.
    ///
    /// Runs as a final post-filter over the merged issue list so it applies
    /// uniformly to every emitting pass — body analysis, the collector, class
    /// checks and dead-code detection — including diagnostics the per-statement
    /// `@psalm-suppress` path in `stmt/mod.rs` structurally cannot reach.
    ///
    /// Issues are *marked* rather than dropped, mirroring that per-statement
    /// path and the kind-level `mir.xml` suppress handler; every consumer (CLI,
    /// WASM, the test harness) already skips [`Issue::suppressed`].
    fn apply_inline_suppressions(&self, issues: &mut [Issue]) {
        use crate::suppression::SuppressionMap;
        if issues.iter().all(|i| i.suppressed) {
            return;
        }
        let db = self.snapshot_db();
        // One map per distinct file, built lazily; `None` once we know a file
        // has no source registered or no suppression comments.
        let mut cache: HashMap<Arc<str>, Option<SuppressionMap>> = HashMap::default();
        for issue in issues.iter_mut() {
            if issue.suppressed {
                continue;
            }
            let map = cache.entry(issue.location.file.clone()).or_insert_with(|| {
                db.lookup_source_file(&issue.location.file)
                    .map(|sf| SuppressionMap::from_source(&sf.text(&db)))
                    .filter(|m| !m.is_empty())
            });
            if let Some(map) = map.as_ref() {
                if map.is_suppressed(issue.location.line, issue.kind.name(), issue.kind.code()) {
                    issue.suppressed = true;
                }
            }
        }
    }

    fn type_exists(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::class_exists(&db, fqcn)
    }

    fn collect_and_ingest_source(
        &self,
        file: Arc<str>,
        src: &str,
        php_version: PhpVersion,
    ) -> FileDefinitions {
        self.db.collect_and_ingest_file(file, src, php_version)
    }

    /// Rebuild the workspace symbol index singleton from every registered source
    /// file. Required in the batch path because `workspace_index` reads the
    /// maintained singleton, and that singleton is built from vendor *before*
    /// `analyze_paths` registers project files (and before `lazy_load_*` faults
    /// in referenced classes). Without refreshing it, `find_class_like` /
    /// `class_exists` miss every project and lazy-loaded class, yielding false
    /// `UndefinedClass`. Cheap after the definition caches are warm (no parsing).
    fn refresh_workspace_index(&self) {
        let mut guard = self.db.salsa.write();
        guard.rebuild_workspace_symbol_index();
    }

    /// Load the configured PHP version + built-in stubs + user stubs into
    /// the shared db. Called by [`Self::analyze_paths`] and
    /// [`Self::collect_definitions`].
    fn load_batch_stubs(&self, php_version: PhpVersion) {
        // Wire the PHP version into the db before any SourceFile inputs are
        // registered — collect_file_definitions reads it for @since/@removed filtering.
        {
            let version_str = Arc::from(php_version.to_string().as_str());
            self.db.salsa.write().set_php_version(version_str);
        }

        // Built-in stubs for the configured PHP version.
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.db.ingest_stub_paths(&paths, php_version);

        // User-configured stubs.
        self.db
            .ingest_user_stubs(&self.user_stub_files, &self.user_stub_dirs);

        // Ensure a resolver is configured so pull-path lookups can map
        // built-in FQCNs to the stub VFS paths registered above.
        let mut guard = self.db.salsa.write();
        if guard.current_resolver().is_none() {
            let resolver: Arc<dyn crate::ClassResolver> = Arc::new(crate::StubClassResolver);
            guard.set_resolver(Some(resolver));
        }
    }

    /// Run the full batch analysis pipeline on a set of file paths.
    pub fn analyze_paths(&self, paths: &[PathBuf], opts: &BatchOptions) -> AnalysisResult {
        let php_version = self.batch_php_version(opts);
        let mut all_issues = Vec::new();
        let _t0 = std::time::Instant::now();

        // ---- Load PHP built-in stubs (before definition collection so user code can override)
        self.load_batch_stubs(php_version);
        let _t_stubs = _t0.elapsed();

        // ---- Read files in parallel ----------------------------------
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

        // ---- Pre-analysis invalidation: evict dependents of changed/removed files
        if let Some(cache) = &self.cache {
            let mut invalidated: Vec<String> = file_data
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

            // Files analyzed in a previous run but now gone from disk: their
            // dependents hold stale results that still assume the deleted
            // definitions exist. A file merely absent from this run's path set
            // (but still on disk) is NOT a deletion — checking disk existence
            // avoids evicting dependents during partial-path analysis.
            let current: std::collections::HashSet<&str> =
                file_data.iter().map(|(f, _)| f.as_ref()).collect();
            let removed: Vec<String> = cache
                .cached_files()
                .into_iter()
                .filter(|f| !current.contains(f.as_str()) && !std::path::Path::new(f).exists())
                .collect();
            for f in &removed {
                cache.evict(f);
            }
            invalidated.extend(removed);

            if !invalidated.is_empty() {
                cache.evict_with_dependents(&invalidated);
            }
        }

        // ---- Register Salsa source inputs for incremental follow-up calls ----
        {
            let mut guard = self.db.salsa.write();
            for parsed in &parsed_files {
                guard.upsert_source_file(parsed.file.clone(), parsed.source.clone());
            }
        }
        let _t_salsa_reg = _t0.elapsed();

        // ---- Definition collection from the already-parsed AST -------
        // Returns (FileDefinitions, content_hash, has_hard_parse_errors) so we
        // can prime the parse cache before the pre-warm loop below.
        type Pass1Entry = (FileDefinitions, [u8; 32], bool);
        let file_defs: Vec<Pass1Entry> = parsed_files
            .par_iter()
            .map(|parsed| {
                let content_hash = hash_source(parsed.source());
                let has_hard_parse_errors = parsed
                    .errors()
                    .iter()
                    .any(crate::parser::is_hard_parse_error);
                let mut all_issues: Vec<Issue> = parsed
                    .errors()
                    .iter()
                    .map(|err| {
                        crate::parser::parse_error_to_issue(
                            err,
                            &parsed.file,
                            parsed.source(),
                            parsed.source_map(),
                        )
                    })
                    .collect();
                let collector = crate::collector::DefinitionCollector::new_for_slice(
                    parsed.file.clone(),
                    parsed.source(),
                    parsed.source_map(),
                );
                let (mut slice, collector_issues) = collector.collect_slice(parsed.owned());
                all_issues.extend(collector_issues);
                mir_codebase::storage::deduplicate_params_in_slice(&mut slice);
                let defs = FileDefinitions {
                    slice: Arc::new(slice),
                    issues: Arc::new(all_issues),
                };
                (defs, content_hash, has_hard_parse_errors)
            })
            .collect();
        let _t_collect_defs = _t0.elapsed();

        // Prime the in-process parse cache so the pre-warm loop below avoids
        // re-parsing every project file through collect_file_definitions.
        {
            let guard = self.db.salsa.read();
            for (defs, hash, has_hard_parse_errors) in &file_defs {
                if !*has_hard_parse_errors {
                    guard.prime_parse_cache(*hash, Arc::clone(&defs.slice));
                }
            }
        }

        let mut files_with_parse_errors: HashSet<Arc<str>> = HashSet::default();
        for (defs, _hash, _hard_err) in file_defs {
            for issue in defs.issues.iter() {
                if matches!(issue.kind, mir_issues::IssueKind::ParseError { .. })
                    && issue.severity == mir_issues::Severity::Error
                {
                    files_with_parse_errors.insert(issue.location.file.clone());
                }
            }
            all_issues.extend(Arc::unwrap_or_clone(defs.issues));
        }
        let _t_ingest = _t0.elapsed();

        // ---- Pre-warm collect_file_definitions for project files -------------
        {
            let db_prewarm = {
                let guard = self.db.salsa.read();
                (**guard).clone()
            };
            let project_source_files: Vec<SourceFile> = {
                let guard = self.db.salsa.read();
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
        let _t_prewarm_ms = (_t0.elapsed() - _t_ingest).as_secs_f64() * 1000.0;

        // Fold the freshly-registered project files into the workspace symbol
        // index singleton. The singleton may have been built from vendor before
        // this run (CLI indexes vendor before analyze_paths); since adding files
        // no longer nulls it, project classes would otherwise be invisible to
        // find_class_like and reported as false UndefinedClass.
        self.refresh_workspace_index();

        // ---- Lazy-load unknown classes via PSR-4 ----------------------------
        let _t_before_lazy = _t0.elapsed();
        if let Some(psr4) = self.psr4.clone() {
            self.lazy_load_missing_classes(psr4, php_version, &mut all_issues);
        }
        let _t_lazyload_ms = (_t0.elapsed() - _t_before_lazy).as_secs_f64() * 1000.0;

        // ---- Class-level checks ---------------------------------------------
        let analyzed_file_set: HashSet<Arc<str>> =
            file_data.iter().map(|(f, _)| f.clone()).collect();
        let _t_class_analyzer = std::time::Instant::now();
        {
            let class_db = {
                let guard = self.db.salsa.read();
                (**guard).clone()
            };
            let class_issues = crate::class::ClassAnalyzer::with_files(
                &class_db,
                analyzed_file_set.clone(),
                &file_data,
            )
            .analyze_all();
            all_issues.extend(class_issues);
        }
        let _t_class_analyzer_ms = _t_class_analyzer.elapsed().as_secs_f64() * 1000.0;

        let _t_class_checks = _t0.elapsed();

        let mut db_main = {
            let guard = self.db.salsa.read();
            (**guard).clone()
        };
        // All index mutation for the body pass is done (lazy_load_missing_classes
        // + refresh ran above; lazy_load_from_body_issues runs *after* this pass
        // on a separate db). Freeze the index on this ephemeral clone so each
        // find_class_like borrows it instead of cloning the singleton's three
        // Arcs per call — the per-worker `map_with` clone bumps the refcount once.
        db_main.freeze_workspace_index();

        // ---- Body analysis: function/method bodies in parallel --------------
        type BodyResult = (
            Arc<str>,
            Vec<Issue>,
            Vec<crate::symbol::ResolvedSymbol>,
            Vec<RefLoc>,
        );
        let body_results: Vec<BodyResult> = parsed_files
            .par_iter()
            .filter(|parsed| !files_with_parse_errors.contains(&parsed.file))
            .map_with(db_main, |db, parsed| {
                let driver = BodyAnalyzer::new(&*db as &dyn MirDatabase, php_version);
                let (issues, symbols) = if let Some(cache) = &self.cache {
                    let h = hash_content(parsed.source());
                    if let Some((cached_issues, ref_locs)) = cache.get(&parsed.file, &h) {
                        // Cache replay: rebuild the file's complete reference
                        // set straight from the cached tuples — no pending-
                        // buffer detour.
                        let locs: Vec<RefLoc> = ref_locs
                            .iter()
                            .map(|(symbol, line, col_start, col_end)| RefLoc {
                                symbol_key: Arc::from(symbol.as_str()),
                                file: parsed.file.clone(),
                                line: *line,
                                col_start: *col_start,
                                col_end: *col_end,
                            })
                            .collect();
                        return (parsed.file.clone(), cached_issues, Vec::new(), locs);
                    }
                    let (issues, symbols) = driver.analyze_bodies(
                        parsed.owned(),
                        parsed.file.clone(),
                        parsed.source(),
                        parsed.source_map(),
                    );
                    let pending = db.take_pending_ref_locs();
                    let cache_locs = pending
                        .iter()
                        .map(|r| (r.symbol_key.to_string(), r.line, r.col_start, r.col_end))
                        .collect();
                    cache.put(&parsed.file, h, issues.clone(), cache_locs);
                    if let Some(cb) = &opts.on_file_done {
                        cb();
                    }
                    return (parsed.file.clone(), issues, symbols, pending);
                } else {
                    driver.analyze_bodies(
                        parsed.owned(),
                        parsed.file.clone(),
                        parsed.source(),
                        parsed.source_map(),
                    )
                };
                let pending = db.take_pending_ref_locs();
                if let Some(cb) = &opts.on_file_done {
                    cb();
                }
                (parsed.file.clone(), issues, symbols, pending)
            })
            .collect();

        let _t_body_analysis = _t0.elapsed();

        // Serial commit with replace semantics: each file's output (or cache
        // replay) is its complete reference set, so stale entries from a
        // prior run cannot survive an append.
        let mut all_symbols = Vec::new();
        {
            let guard = self.db.salsa.read();
            for (file, issues, symbols, ref_locs) in body_results {
                all_issues.extend(issues);
                all_symbols.extend(symbols);
                guard.set_file_reference_locations(file.as_ref(), ref_locs);
            }
        }

        // ---- Post-analysis lazy loading: FQCNs used without `use` imports ------
        if let Some(psr4) = self.psr4.clone() {
            self.lazy_load_from_body_issues(
                psr4,
                php_version,
                &file_data,
                &files_with_parse_errors,
                &mut all_issues,
                &mut all_symbols,
            );
        }

        // ---- Build reverse dep graph and persist it for the next run ---------
        // Must run AFTER `commit_reference_locations_batch` (above): the graph's
        // call-site / instantiation / inferred-return edges are derived from the
        // committed reference-location map. Built any earlier (the salsa db is
        // fresh each session) that map is empty, so only structural edges
        // (parent/interface/trait/declared types) survive — and any dependent
        // reachable only through a call site or inferred type goes stale.
        if let Some(cache) = &self.cache {
            let db_snapshot = {
                let guard = self.db.salsa.read();
                (**guard).clone()
            };
            let rev = build_reverse_deps(&db_snapshot);
            cache.set_reverse_deps(rev);
        }

        // Persist cache hits/misses to disk
        if let Some(cache) = &self.cache {
            cache.flush();
        }

        // ---- Dead-code detection -------------------------------------------
        if opts.should_run_dead_code() {
            let salsa = self.snapshot_db();
            let _t_dead_code = std::time::Instant::now();
            let dead_code_issues =
                crate::dead_code::DeadCodeAnalyzer::with_files(&salsa, analyzed_file_set.clone())
                    .analyze();
            all_issues.extend(dead_code_issues);
            if std::env::var("MIR_TIMING").is_ok() {
                eprintln!(
                    "[timing] dead_code_analyzer={:.0}ms",
                    _t_dead_code.elapsed().as_secs_f64() * 1000.0
                );
            }
        }

        let _t_total = _t0.elapsed();
        if std::env::var("MIR_TIMING").is_ok() {
            eprintln!(
                "[timing] stubs={:.0}ms read={:.0}ms salsa_reg={:.0}ms collect_defs={:.0}ms ingest={:.0}ms class_checks={:.0}ms (prewarm={:.0}ms lazy_load={:.0}ms class_analyzer={:.0}ms) body_analysis={:.0}ms total={:.0}ms",
                _t_stubs.as_secs_f64() * 1000.0,
                (_t_read - _t_stubs).as_secs_f64() * 1000.0,
                (_t_salsa_reg - _t_read).as_secs_f64() * 1000.0,
                (_t_collect_defs - _t_salsa_reg).as_secs_f64() * 1000.0,
                (_t_ingest - _t_collect_defs).as_secs_f64() * 1000.0,
                (_t_class_checks - _t_ingest).as_secs_f64() * 1000.0,
                _t_prewarm_ms,
                _t_lazyload_ms,
                _t_class_analyzer_ms,
                (_t_body_analysis - _t_class_checks).as_secs_f64() * 1000.0,
                _t_total.as_secs_f64() * 1000.0,
            );
        }

        opts.apply(&mut all_issues);
        self.apply_inline_suppressions(&mut all_issues);
        if let Some(dump) = crate::metrics::dump() {
            eprintln!("{dump}");
        }

        // ---- Build workspace symbol index singleton -------------------------
        {
            let mut guard = self.db.salsa.write();
            guard.rebuild_workspace_symbol_index();
        }

        AnalysisResult::build(all_issues, rustc_hash::FxHashMap::default(), all_symbols)
    }

    fn lazy_load_missing_classes(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        php_version: PhpVersion,
        all_issues: &mut Vec<Issue>,
    ) {
        let max_depth = 10;
        let mut loaded: HashSet<String> = HashSet::default();
        let mut scanned: HashSet<Arc<str>> = HashSet::default();

        for _ in 0..max_depth {
            let mut to_load: Vec<(String, PathBuf)> = Vec::new();

            let mut try_queue = |fqcn: &str| {
                if !self.type_exists(fqcn) && !loaded.contains(fqcn) {
                    if let Some(path) = psr4.resolve(fqcn) {
                        to_load.push((fqcn.to_string(), path));
                    }
                }
            };

            let mut candidates: Vec<String> = Vec::new();
            let import_candidates = {
                let db_owned = self.snapshot_db();
                let db = &db_owned;
                for fqcn in crate::db::workspace_classes(db).iter() {
                    if scanned.contains(fqcn.as_ref()) {
                        continue;
                    }
                    let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                    let Some(class) = crate::db::find_class_like(db, here) else {
                        continue;
                    };
                    scanned.insert(fqcn.clone());
                    collect_class_referenced_fqcns(&class, &mut candidates);
                }
                db.file_import_snapshots()
                    .into_iter()
                    .flat_map(|(_, imports)| {
                        imports
                            .values()
                            .map(|sym| sym.as_str().to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            for fqcn in candidates {
                try_queue(&fqcn);
            }
            for fqcn in import_candidates {
                try_queue(&fqcn);
            }

            if to_load.is_empty() {
                break;
            }

            // Mark everything queued as loaded up-front so a file that fails to
            // read isn't retried on the next depth iteration (matches the serial
            // behaviour, where `loaded.insert` ran before the read attempt).
            for (fqcn, _) in &to_load {
                loaded.insert(fqcn.clone());
            }

            // Read + parse + ingest the missing classes in parallel. The parse
            // and definition walk inside `collect_and_ingest_source` already run
            // off the salsa write lock (it takes the lock only for the brief
            // input upsert), so fanning the per-file work across the rayon pool
            // turns this previously-serial phase — the dominant cost on the lazy
            // path — concurrent. `collect()` on a rayon map preserves input
            // order, so the resulting issue ordering matches the serial version.
            let per_file_issues: Vec<Vec<Issue>> = to_load
                .par_iter()
                .map(|(_, path)| -> Vec<Issue> {
                    let Ok(src) = std::fs::read_to_string(path) else {
                        return Vec::new();
                    };
                    let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                    let is_vendor = file.contains("/vendor/") || file.contains("\\vendor\\");
                    let defs = self.collect_and_ingest_source(file, &src, php_version);
                    if is_vendor {
                        Vec::new()
                    } else {
                        Arc::unwrap_or_clone(defs.issues)
                    }
                })
                .collect();
            for mut issues in per_file_issues {
                all_issues.append(&mut issues);
            }

            // Make the just-loaded classes visible to the next iteration's
            // transitive scan and to the caller's post-lazy-load snapshot.
            self.refresh_workspace_index();
        }
    }

    fn lazy_load_from_body_issues(
        &self,
        psr4: Arc<crate::composer::Psr4Map>,
        php_version: PhpVersion,
        file_data: &[(Arc<str>, Arc<str>)],
        files_with_parse_errors: &HashSet<Arc<str>>,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<crate::symbol::ResolvedSymbol>,
    ) {
        use mir_issues::IssueKind;

        let max_depth = 5;
        let mut loaded: HashSet<String> = HashSet::default();

        for _ in 0..max_depth {
            let mut to_load: HashMap<String, PathBuf> = HashMap::default();

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
                    let _ = self.collect_and_ingest_source(file, &src, php_version);
                }
            }

            // Make the loaded classes visible to the type_exists() check below
            // (and to the reanalysis snapshot) so resolved files are detected.
            self.refresh_workspace_index();

            self.lazy_load_missing_classes(psr4.clone(), php_version, all_issues);

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
                let guard = self.db.salsa.read();
                (**guard).clone()
            };

            let reanalysis: Vec<(Vec<Issue>, Vec<crate::symbol::ResolvedSymbol>, Vec<RefLoc>)> =
                file_data
                    .par_iter()
                    .filter(|(f, _)| {
                        !files_with_parse_errors.contains(f) && files_to_reanalyze.contains(f)
                    })
                    .map_with(db_full, |db, (file, src)| {
                        let driver = BodyAnalyzer::new(&*db as &dyn MirDatabase, php_version);
                        let parsed = php_rs_parser::parse(src);
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
                let guard = self.db.salsa.read();
                guard.commit_reference_locations_batch(reanalysis_ref_locs);
            }
        }
    }

    /// Re-analyze a single file (definition collection + body analysis) within the batch context.
    ///
    /// Mirrors the old `ProjectAnalyzer::re_analyze_file` cache-aware path.
    /// Use [`Self::reanalyze_dependents`] for LSP-style per-file flows that
    /// don't need batch options.
    pub fn re_analyze_file(
        &self,
        file_path: &str,
        new_content: &str,
        opts: &BatchOptions,
    ) -> AnalysisResult {
        let php_version = self.batch_php_version(opts);

        // Fast path: content unchanged and cache has a valid entry.
        if let Some(cache) = &self.cache {
            let h = hash_content(new_content);
            if let Some((mut issues, ref_locs)) = cache.get(file_path, &h) {
                let file: Arc<str> = Arc::from(file_path);
                // Replace semantics: the cached set is the file's complete
                // reference set, so stale entries from a prior version are
                // cleared rather than appended over.
                let locs: Vec<RefLoc> = ref_locs
                    .iter()
                    .map(|(symbol, line, col_start, col_end)| RefLoc {
                        symbol_key: Arc::from(symbol.as_str()),
                        file: file.clone(),
                        line: *line,
                        col_start: *col_start,
                        col_end: *col_end,
                    })
                    .collect();
                let guard = self.db.salsa.read();
                guard.set_file_reference_locations(file_path, locs);
                drop(guard);
                opts.apply(&mut issues);
                self.apply_inline_suppressions(&mut issues);
                return AnalysisResult::build(issues, HashMap::default(), Vec::new());
            }
        }

        let file: Arc<str> = Arc::from(file_path);

        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file_path);
        }

        let file_defs = {
            let mut guard = self.db.salsa.write();
            let salsa_file = guard.upsert_source_file(file.clone(), Arc::from(new_content));
            collect_file_definitions(&**guard, salsa_file)
        };

        let mut all_issues: Vec<Issue> = Arc::unwrap_or_clone(file_defs.issues.clone());

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

        let symbols = {
            let guard = self.db.salsa.write();

            let parsed = php_rs_parser::parse(new_content);

            let has_hard_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);
            if !has_hard_errors {
                let db_ref: &dyn MirDatabase = &**guard;
                let driver = BodyAnalyzer::new(db_ref, php_version);
                let (body_issues, symbols) = driver.analyze_bodies(
                    &parsed.program,
                    file.clone(),
                    new_content,
                    &parsed.source_map,
                );
                all_issues.extend(body_issues);
                let pending = guard.take_pending_ref_locs();
                guard.set_file_reference_locations(file.as_ref(), pending);
                symbols
            } else {
                Vec::new()
            }
        };

        // Bake inline-suppression marks in *before* caching: suppression is a
        // pure function of file content (and the cache key hashes content), so
        // the cached issues should already carry their marks. The cache-hit
        // branch above replays this file's source without re-registering the
        // `SourceFile` input, so the db-backed post-filter cannot recompute
        // marks there — caching the canonical result is what keeps a fresh
        // process honoring `@mir-ignore` on an unchanged file.
        mark_suppressed(
            &mut all_issues,
            &crate::suppression::SuppressionMap::from_source(new_content),
        );

        if let Some(cache) = &self.cache {
            let h = hash_content(new_content);
            cache.evict_with_dependents(&[file_path.to_string()]);
            let db = self.snapshot_db();
            let ref_locs = extract_reference_locations(&db, &file);
            cache.put(file_path, h, all_issues.clone(), ref_locs);
        }

        opts.apply(&mut all_issues);
        AnalysisResult::build(all_issues, HashMap::default(), symbols)
    }

    /// Collect type definitions only from `paths` into the codebase
    /// without analyzing method bodies or emitting issues. Used to load
    /// vendor types.
    ///
    /// When a disk-backed cache is attached, per-file `StubSlice` results
    /// from previous runs are reused on a content-hash match, eliminating
    /// the parse + definition-collection step. Cache misses run the normal
    /// pipeline and write back so subsequent runs hit.
    pub fn collect_definitions(&self, paths: &[PathBuf]) {
        let _timing = std::env::var("MIR_TIMING").is_ok();
        let _t0 = std::time::Instant::now();

        let php_v = self.php_version.cache_byte();

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
                let cached = self.db.stub_cache.as_ref().and_then(|c| {
                    let mut slice = c.get(&file, &hash, php_v)?;
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

        let source_files: Vec<SourceFile> = {
            let mut guard = self.db.salsa.write();
            entries
                .iter()
                .map(|e| {
                    guard.upsert_source_file_with_durability(
                        e.file.clone(),
                        e.src.clone(),
                        salsa::Durability::HIGH,
                    )
                })
                .collect()
        };
        let _t_reg = _t0.elapsed();

        let db_pass1 = {
            let guard = self.db.salsa.read();
            (**guard).clone()
        };
        let stub_cache = self.db.stub_cache.clone();
        let prepared: Vec<mir_codebase::storage::StubSlice> = entries
            .into_par_iter()
            .zip(source_files.into_par_iter())
            .map_with(db_pass1, |db, (mut entry, salsa_file)| {
                if let Some(slice) = entry.cached.take() {
                    let slice_arc = Arc::new(slice);
                    db.parse_cache().insert(entry.hash, Arc::clone(&slice_arc));
                    return (*slice_arc).clone();
                }
                let defs = collect_file_definitions(&*db, salsa_file);
                if let Some(cache) = stub_cache.as_ref() {
                    cache.put(&entry.file, &entry.hash, php_v, &defs.slice);
                }
                (*defs.slice).clone()
            })
            .collect();
        let _t_collect = _t0.elapsed();
        drop(prepared);
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

        {
            let mut guard = self.db.salsa.write();
            guard.rebuild_workspace_symbol_index();
        }

        crate::collector::print_collector_stats();
    }
}

/// Analyze a PHP source string without a real file path. Useful for tests
/// and single-file LSP mode. Allocates a throwaway db; doesn't touch any
/// existing session.
pub fn analyze_source(source: &str) -> AnalysisResult {
    let php_version = PhpVersion::LATEST;
    let file: Arc<str> = Arc::from("<source>");
    let mut db = MirDbStorage::default();
    db.set_php_version(Arc::from(php_version.to_string().as_str()));
    crate::stubs::load_stubs_for_version(&mut db, php_version);
    let salsa_file = SourceFile::new(&db, file.clone(), Arc::from(source));
    let file_defs = collect_file_definitions(&db, salsa_file);
    let suppressions = crate::suppression::SuppressionMap::from_source(source);
    let mut all_issues = Arc::unwrap_or_clone(file_defs.issues);
    if all_issues.iter().any(|issue| {
        matches!(issue.kind, mir_issues::IssueKind::ParseError { .. })
            && issue.severity == mir_issues::Severity::Error
    }) {
        mark_suppressed(&mut all_issues, &suppressions);
        return AnalysisResult::build(all_issues, rustc_hash::FxHashMap::default(), Vec::new());
    }
    let mut type_envs = rustc_hash::FxHashMap::default();
    let mut all_symbols = Vec::new();
    let result = php_rs_parser::parse(source);

    let driver = BodyAnalyzer::new(&db, php_version);
    all_issues.extend(driver.analyze_bodies_typed(
        &result.program,
        file.clone(),
        source,
        &result.source_map,
        &mut type_envs,
        &mut all_symbols,
    ));
    mark_suppressed(&mut all_issues, &suppressions);
    AnalysisResult::build(all_issues, type_envs, all_symbols)
}

/// Mark issues silenced by a single file's [`SuppressionMap`]. Shared by the
/// in-memory [`analyze_source`] entry point, which has the source in hand and
/// does not go through the db-backed batch post-filter.
fn mark_suppressed(issues: &mut [Issue], suppressions: &crate::suppression::SuppressionMap) {
    if suppressions.is_empty() {
        return;
    }
    for issue in issues.iter_mut() {
        if !issue.suppressed
            && suppressions.is_suppressed(issue.location.line, issue.kind.name(), issue.kind.code())
        {
            issue.suppressed = true;
        }
    }
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
// FQCN reference walk — collects every class-name reference reachable from a
// ClassLike's signature surface. Used by lazy_load_missing_classes to chase
// transitive vendor types.
// ---------------------------------------------------------------------------

pub(crate) fn collect_class_referenced_fqcns(class: &crate::db::ClassLike, out: &mut Vec<String>) {
    if let Some(p) = class.parent() {
        out.push(p.to_string());
    }
    for i in class.interfaces() {
        out.push(i.to_string());
    }
    for e in class.extends() {
        out.push(e.to_string());
    }
    for t in class.class_traits() {
        out.push(t.to_string());
    }
    for m in class.mixins() {
        out.push(m.to_string());
    }
    for u in class.extends_type_args() {
        collect_fqcns_in_union(u, out);
    }
    for (iface, args) in class.implements_type_args() {
        out.push(iface.to_string());
        for u in args {
            collect_fqcns_in_union(u, out);
        }
    }
    for (_, m) in class.own_methods().iter() {
        for p in m.params.iter() {
            if let Some(t) = &p.ty {
                collect_fqcns_in_union(t, out);
            }
        }
        if let Some(t) = &m.return_type {
            collect_fqcns_in_union(t, out);
        }
        for thrown in m.throws.iter() {
            out.push(thrown.to_string());
        }
    }
    if let Some(props) = class.own_properties() {
        for (_, p) in props.iter() {
            if let Some(t) = &p.ty {
                collect_fqcns_in_union(t, out);
            }
        }
    }
    for (_, c) in class.own_constants().iter() {
        collect_fqcns_in_union(&c.ty, out);
    }
}

pub(crate) fn collect_fqcns_in_union(u: &Type, out: &mut Vec<String>) {
    for atom in u.types.iter() {
        collect_fqcns_in_atomic(atom, out);
    }
}

fn collect_fqcns_in_simple(t: &mir_types::compact::SimpleType, out: &mut Vec<String>) {
    if let mir_types::compact::SimpleType::Complex(u) = t {
        collect_fqcns_in_union(u, out);
    }
}

pub(crate) fn collect_fqcns_in_atomic(a: &Atomic, out: &mut Vec<String>) {
    match a {
        Atomic::TNamedObject { fqcn, type_params } => {
            out.push(fqcn.to_string());
            for tp in type_params.iter() {
                collect_fqcns_in_union(tp, out);
            }
        }
        Atomic::TStaticObject { fqcn } | Atomic::TSelf { fqcn } | Atomic::TParent { fqcn } => {
            out.push(fqcn.to_string());
        }
        Atomic::TLiteralEnumCase { enum_fqcn, .. } => {
            out.push(enum_fqcn.to_string());
        }
        Atomic::TClassString(Some(s)) => {
            out.push(s.to_string());
        }
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            collect_fqcns_in_union(key, out);
            collect_fqcns_in_union(value, out);
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
            collect_fqcns_in_union(value, out);
        }
        Atomic::TKeyedArray { properties, .. } => {
            for (_, kp) in properties.iter() {
                collect_fqcns_in_union(&kp.ty, out);
            }
        }
        Atomic::TClosure {
            params,
            return_type,
            this_type,
        } => {
            for p in params {
                if let Some(t) = &p.ty {
                    collect_fqcns_in_simple(t, out);
                }
            }
            collect_fqcns_in_union(return_type, out);
            if let Some(t) = this_type {
                collect_fqcns_in_union(t, out);
            }
        }
        Atomic::TCallable {
            params,
            return_type,
        } => {
            if let Some(ps) = params {
                for p in ps {
                    if let Some(t) = &p.ty {
                        collect_fqcns_in_simple(t, out);
                    }
                }
            }
            if let Some(rt) = return_type {
                collect_fqcns_in_union(rt, out);
            }
        }
        Atomic::TIntersection { parts } => {
            for p in parts.iter() {
                collect_fqcns_in_union(p, out);
            }
        }
        Atomic::TConditional {
            param_name: _,
            subject,
            if_true,
            if_false,
        } => {
            collect_fqcns_in_union(subject, out);
            collect_fqcns_in_union(if_true, out);
            collect_fqcns_in_union(if_false, out);
        }
        Atomic::TTemplateParam { as_type, .. } => {
            collect_fqcns_in_union(as_type, out);
        }
        _ => {}
    }
}

fn build_reverse_deps(db: &dyn crate::db::MirDatabase) -> HashMap<String, HashSet<String>> {
    let mut reverse: HashMap<String, HashSet<String>> = HashMap::default();

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
            add_edge(fqcn.as_str(), &file);
        }
    }

    let extract_named_objects = |union: &mir_types::Type| {
        union
            .types
            .iter()
            .filter_map(|atomic| match atomic {
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(*fqcn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    for fqcn in crate::db::workspace_classes(db).iter() {
        let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
        let Some(class) = crate::db::find_class_like(db, here) else {
            continue;
        };
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

    for fqn in crate::db::workspace_functions(db).iter() {
        let here = crate::db::Fqcn::from_str(db, fqn.as_ref());
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
    pub type_envs: rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
    /// Per-expression resolved symbols from body analysis, sorted by file path.
    pub symbols: Vec<crate::symbol::ResolvedSymbol>,
    /// Maps each file path to the contiguous range within `symbols` that
    /// belongs to it.
    symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>>,
}

impl AnalysisResult {
    fn build(
        issues: Vec<Issue>,
        type_envs: rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        mut symbols: Vec<crate::symbol::ResolvedSymbol>,
    ) -> Self {
        symbols.sort_unstable_by(|a, b| a.file.as_ref().cmp(b.file.as_ref()));
        let mut symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>> = HashMap::default();
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

    pub fn issues_by_file(&self) -> HashMap<Arc<str>, Vec<&Issue>> {
        let mut map: HashMap<Arc<str>, Vec<&Issue>> = HashMap::default();
        for issue in &self.issues {
            map.entry(issue.location.file.clone())
                .or_default()
                .push(issue);
        }
        map
    }

    pub fn count_by_severity(&self) -> Vec<(mir_issues::Severity, usize)> {
        let mut counts: std::collections::BTreeMap<mir_issues::Severity, usize> =
            std::collections::BTreeMap::new();
        for issue in &self.issues {
            *counts.entry(issue.severity).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }

    pub fn total_issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn filter_issues<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a Issue>
    where
        F: Fn(&Issue) -> bool + 'a,
    {
        self.issues.iter().filter(move |i| predicate(i))
    }

    pub fn symbol_at(
        &self,
        file: &str,
        byte_offset: u32,
    ) -> Option<&crate::symbol::ResolvedSymbol> {
        let range = self.symbols_by_file.get(file)?;
        let symbols = &self.symbols[range.clone()];

        // Primary: cursor is on an identifier token.
        if let Some(sym) = symbols
            .iter()
            .filter(|s| s.span.start <= byte_offset && byte_offset < s.span.end)
            .min_by_key(|s| s.span.end - s.span.start)
        {
            return Some(sym);
        }

        // Fallback: cursor is in a call-expression gap (e.g. the whitespace or
        // argument list between two chained method calls).  Match against the
        // full expression span recorded for call-like symbols and return the
        // innermost (smallest) enclosing call, mirroring what an AST-walk to
        // the innermost containing call expression would produce.
        symbols
            .iter()
            .filter(|s| {
                s.expr_span
                    .is_some_and(|es| es.start <= byte_offset && byte_offset < es.end)
            })
            .min_by_key(|s| {
                let es = s.expr_span.unwrap();
                es.end - es.start
            })
    }
}
