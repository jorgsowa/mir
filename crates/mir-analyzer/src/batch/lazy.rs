use super::*;

impl AnalysisSession {
    pub(super) fn lazy_load_missing_classes(
        &mut self,
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
                    if scanned.contains(fqcn.as_str()) {
                        continue;
                    }
                    let here = crate::db::Fqcn::from_str(db, fqcn.as_str());
                    let Some(class) = crate::db::find_class_like(db, here) else {
                        continue;
                    };
                    scanned.insert(Arc::from(fqcn.as_str()));
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

            // Parse + collect in parallel on per-thread snapshots, then
            // register serially; `collect()` keeps input order, so issue
            // ordering is deterministic.
            let db_template = self.snapshot_db();
            let stub_cache = self.db.stub_cache.clone();
            let prepared: Vec<Option<(bool, crate::analyzer_db::PreparedIngest)>> = to_load
                .par_iter()
                .map_with(db_template, |db, (_, path)| {
                    let Ok(src) = std::fs::read_to_string(path) else {
                        return None;
                    };
                    let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
                    let is_vendor = file.contains("/vendor/") || file.contains("\\vendor\\");
                    let prepared = crate::analyzer_db::AnalyzerDb::prepare_ingest(
                        db,
                        stub_cache.as_ref(),
                        file,
                        &src,
                        php_version,
                    );
                    Some((is_vendor, prepared))
                })
                .collect();
            for entry in prepared {
                let Some((is_vendor, prepared)) = entry else {
                    continue;
                };
                let collected = self.db.commit_ingest(prepared);
                if !is_vendor {
                    all_issues.append(&mut Arc::unwrap_or_clone(collected.file_defs.issues));
                }
            }

            // Make the just-loaded classes visible to the next iteration's
            // transitive scan and to the caller's post-lazy-load snapshot.
            self.refresh_workspace_index();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn lazy_load_from_body_issues(
        &mut self,
        psr4: Arc<crate::composer::Psr4Map>,
        php_version: PhpVersion,
        file_data: &[(Arc<str>, Arc<str>)],
        files_with_parse_errors: &HashSet<Arc<str>>,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<crate::symbol::ResolvedSymbol>,
        skip_symbols: bool,
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

            let mut db_full = self.db.snapshot_db();
            // This round's index mutation is done (ingest + refresh +
            // lazy_load_missing_classes ran above). Freeze on the ephemeral
            // per-round clone, same as the main body pass in run.rs.
            db_full.freeze_workspace_index();

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
                if !skip_symbols {
                    all_symbols.extend(symbols);
                }
                reanalysis_ref_locs.extend(ref_locs);
            }
            self.db
                .salsa
                .commit_reference_locations_batch(reanalysis_ref_locs);
        }
    }
}
