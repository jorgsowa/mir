use super::*;

impl AnalysisSession {
    /// Run the full batch analysis pipeline on a set of file paths.
    pub fn analyze_paths(&self, paths: &[PathBuf], opts: &BatchOptions) -> AnalysisResult {
        let php_version = self.batch_php_version(opts);
        let mut all_issues = Vec::new();
        let _t0 = std::time::Instant::now();

        // ---- Load PHP built-in stubs (before definition collection so user code can override)
        self.load_batch_stubs(php_version, !opts.skip_builtin_stubs);
        // Index vendor autoload.files (global function/constant helpers such as
        // Laravel's `confirm()`, `select()`, etc.) before body analysis so
        // calls to these functions resolve rather than emitting UndefinedFunction.
        self.ensure_vendor_eager_functions();
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
                    .filter(|err| !crate::parser::is_spurious_reserved_class_error(err))
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
                    let symbols = if opts.skip_symbols {
                        Vec::new()
                    } else {
                        symbols
                    };
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
                // Drop the per-file symbol vec inside the worker when the
                // consumer opted out — the orchestrator never accumulates.
                let symbols = if opts.skip_symbols {
                    Vec::new()
                } else {
                    symbols
                };
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
                opts.skip_symbols,
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
        let analyzed_files_vec: Vec<Arc<str>> = analyzed_file_set.iter().cloned().collect();
        self.apply_suppressions_and_emit_unused(&mut all_issues, &analyzed_files_vec);
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
                self.apply_suppressions_and_emit_unused(&mut issues, std::slice::from_ref(&file));
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
