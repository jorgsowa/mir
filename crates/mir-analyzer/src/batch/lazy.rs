use super::*;

impl AnalysisSession {
    pub(super) fn lazy_load_missing_classes(
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn lazy_load_from_body_issues(
        &self,
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
                if !skip_symbols {
                    all_symbols.extend(symbols);
                }
                reanalysis_ref_locs.extend(ref_locs);
            }
            {
                let guard = self.db.salsa.read();
                guard.commit_reference_locations_batch(reanalysis_ref_locs);
            }
        }
    }
}
