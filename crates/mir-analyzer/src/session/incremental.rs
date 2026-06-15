use super::*;

impl AnalysisSession {
    /// Retrieve the source text the session has registered for `file`, if
    /// any. Returns `None` when the file has never been ingested. Used by
    /// the parallel re-analysis path to re-feed dependents to body analysis without
    /// the caller having to track sources independently.
    pub fn source_of(&self, file: &str) -> Option<Arc<str>> {
        let db = self.snapshot_db();
        let sf = db.lookup_source_file(file)?;
        Some(sf.text(&db))
    }

    /// Re-analyze every transitive dependent of `file` in parallel.
    ///
    /// When the user saves a file that other files depend on (e.g. editing
    /// a base class, an interface, or a trait), those dependents may have
    /// new diagnostics. This method computes them in parallel using rayon
    /// and returns the per-file analysis results so the LSP server can
    /// publish updated diagnostics in one batch.
    ///
    /// Source text for dependents is retrieved from the session's salsa
    /// inputs (set by previous `ingest_file` calls) — the caller doesn't
    /// need to track or re-read files. Files for which the session has no
    /// source are silently skipped (returns the analyzable subset).
    ///
    /// Cross-file inferred return types are resolved on demand via salsa.
    pub fn reanalyze_dependents(&self, file: &str) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        self.reanalyze_dependents_cancellable(file, &crate::IndexCancel::new())
    }

    /// Cancellable variant of [`Self::reanalyze_dependents`].
    ///
    /// The consumer flips `cancel` (typically because a newer edit arrived) to
    /// abandon the re-analysis; the flag is checked at each file boundary. Salsa
    /// cannot unwind the plain-Rust body-analysis walk mid-flight, so a file
    /// already in progress finishes, but no further files are started. Files
    /// skipped due to cancellation are simply absent from the returned vec —
    /// the consumer should drop a stale flag and start fresh work on each edit.
    pub fn reanalyze_dependents_cancellable(
        &self,
        file: &str,
        cancel: &crate::IndexCancel,
    ) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        use rayon::prelude::*;

        if cancel.is_cancelled() {
            return Vec::new();
        }

        // Phase 1: compute dependents outside the analysis loop.
        let dependents = self.dependency_graph().transitive_dependents(file);
        if dependents.is_empty() {
            return Vec::new();
        }
        let dependents: Vec<Arc<str>> = dependents
            .into_iter()
            .map(|path| Arc::from(path.as_str()))
            .collect();

        // Phase 2a: fault in each dependent's direct class references if the
        // background indexer hasn't reached them yet (mirrors the FileAnalyzer
        // warm-up behavior, avoiding transient false `UndefinedClass` during
        // index warm-up).
        //
        // This runs SERIALLY and *before* the parallel analyze loop below:
        // `prepare_ast_for_analysis` resolves and loads classes, and loading
        // mutates the shared session salsa storage (`load_class` →
        // `ingest_file` sets salsa inputs). Salsa input mutation cancels and
        // blocks until every other database handle is released, so it must run
        // with NO live snapshot in scope:
        //
        //  - in parallel (the v0.37.0 regression), sibling rayon workers held
        //    live snapshot clones mid-`analyze_file`, so the first warm-up
        //    write blocked on them forever — under high dependent fan-out this
        //    deadlocked the whole runtime; and
        //  - even serially, a snapshot held across the loop (e.g. one taken to
        //    parse the dependents) blocks the very first write.
        //
        // So each iteration takes a *scoped* snapshot to fetch the parsed AST,
        // drops it (the `Arc<ParseResult>` is owned), and only then warms up.
        for file in &dependents {
            if cancel.is_cancelled() {
                return Vec::new();
            }
            let parsed = {
                let db = self.snapshot_db();
                let Some(sf) = db.lookup_source_file(file.as_ref()) else {
                    continue;
                };
                crate::db::parse_file(&db as &dyn crate::db::MirDatabase, sf).0
            };
            self.prepare_ast_for_analysis(&parsed.program, file.as_ref());
        }

        // Phase 2b: drive each dependent through the `analyze_file` tracked
        // query in parallel. Salsa's memo validation does the real work
        // here: after a body-only edit, a dependent whose tracked inputs are
        // structurally unchanged (`FileDefinitions` backdating) returns its
        // cached output without re-running body analysis — re-analysis cost
        // scales with what actually changed, not with dependent count.
        //
        // The snapshot is taken AFTER the warm-up above so each worker observes
        // the freshly-loaded classes. This loop is read-only on salsa: no
        // worker mutates inputs, so the snapshots never contend on a write.
        //
        // Dependents' `FileAnalysis::symbols` are empty on this path:
        // per-expression symbols are intentionally not memoized (a typical
        // file resolves thousands; caching them balloons memory), and
        // diagnostics consumers don't read them. Hover / go-to-definition
        // flows analyze the open file directly via [`crate::FileAnalyzer`].
        //
        // Each worker short-circuits when cancellation has been requested.
        let db_main = self.snapshot_db();
        let results: Vec<(Arc<str>, std::sync::Arc<crate::db::AnalyzeOutput>)> = dependents
            .into_par_iter()
            .map_with(db_main, |db, file| {
                if cancel.is_cancelled() {
                    return None;
                }
                let sf = db.lookup_source_file(file.as_ref())?;
                let out = crate::db::analyze_file(&*db as &dyn crate::db::MirDatabase, sf);
                Some((file, out))
            })
            .flatten()
            .collect();

        // Serial commit: each dependent's output is its complete reference
        // set, so replace rather than append.
        {
            let guard = self.db.salsa.read();
            for (file, out) in &results {
                guard.set_file_reference_locations(file.as_ref(), out.ref_locs.to_vec());
            }
        }

        results
            .into_iter()
            .map(|(file, out)| {
                (
                    file,
                    crate::FileAnalysis {
                        issues: out.issues.to_vec(),
                        symbols: Vec::new(),
                    },
                )
            })
            .collect()
    }

    /// FQCNs that `file` imports via `use` statements but that aren't yet
    /// loaded in the session.
    ///
    /// Designed as the input to background prefetching: after the LSP server
    /// Return the `use`-import alias map for a file: a list of `(alias, fqcn)`
    /// pairs where `alias` is the local name (e.g. `"Str"`) and `fqcn` is the
    /// fully-qualified name (e.g. `"Illuminate\\Support\\Str"`).
    ///
    /// Completion handlers can use this to expand a short class name written
    /// before `::` into its FQN before looking up static members, mirroring the
    /// same alias expansion that go-to-definition already performs via
    /// `symbol_at` + `definition_of`.
    ///
    /// Returns an empty Vec if the file has not been ingested or has no use
    /// imports.
    pub fn class_imports(&self, file: &str) -> Vec<(Arc<str>, Arc<str>)> {
        let db = self.snapshot_db();
        let imports = db.file_imports(file);
        imports
            .iter()
            .map(|(alias, fqcn)| (Arc::from(alias.as_str()), Arc::from(fqcn.as_str())))
            .collect()
    }

    /// ingests an open buffer, it can call this and lazy-load the returned
    /// FQCNs on a worker thread so the user's first Cmd+Click into vendor
    /// code doesn't pay the file-read+parse cost.
    ///
    /// Returns an empty Vec if the file hasn't been ingested or has no
    /// unresolved imports.
    pub fn pending_lazy_loads(&self, file: &str) -> Vec<Arc<str>> {
        let db = self.snapshot_db();
        let imports = db.file_imports(file);
        if imports.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for fqcn in imports.values() {
            let here = crate::db::Fqcn::new(&db, *fqcn);
            if crate::db::find_class_like(&db, here).is_some() {
                continue;
            }
            if let Some(resolver) = &self.resolver {
                if resolver.resolve(fqcn.as_str()).is_some() {
                    out.push(Arc::from(fqcn.as_str()));
                }
            }
        }
        out
    }

    /// Convenience: synchronously lazy-load every import of `file` that
    /// isn't already in the codebase. Returns the number successfully loaded.
    ///
    /// For non-blocking prefetch, call this from a worker thread:
    ///
    /// ```ignore
    /// let s = session.clone();  // AnalysisSession is wrapped in Arc by callers
    /// std::thread::spawn(move || {
    ///     s.prefetch_imports(&file_path);
    /// });
    /// ```
    ///
    /// Uses a single shared-visited two-tier BFS across all pending imports
    /// (see [`Self::load_classes_transitive_bounded`]) with a shallow depth so
    /// member access on imported types type-checks without pulling in the
    /// entire vendor tree.
    pub fn prefetch_imports(&self, file: &str) -> usize {
        let pending = self.pending_lazy_loads(file);
        if pending.is_empty() {
            return 0;
        }
        // Fault in each imported FQCN directly (single-file load + tier-merge).
        // Inheritance ancestors / signature types resolve through the eagerly
        // built workspace symbol index — no transitive walk needed here.
        let mut loaded = 0;
        for fqcn in &pending {
            if self.load_class(fqcn.as_ref()).is_loaded() {
                loaded += 1;
            }
        }
        loaded
    }

    /// All class / interface / trait / enum FQCNs currently known to the
    /// session, each paired with the file that defines them when available.
    ///
    /// Use this to build workspace-wide views (outline, fuzzy search, etc.).
    /// Consumers implement their own search/match logic on top — the analyzer
    /// only exposes the iterator.
    pub fn all_classes(&self) -> Vec<(Arc<str>, Option<mir_types::Location>)> {
        let db = self.snapshot_db();
        crate::db::workspace_classes(&db)
            .iter()
            .filter_map(|fqcn| {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                crate::db::find_class_like(&db, here)
                    .map(|class| (fqcn.clone(), class.location().cloned()))
            })
            .collect()
    }

    /// All global function FQNs currently known to the session, each paired
    /// with their declaration location when available.
    pub fn all_functions(&self) -> Vec<(Arc<str>, Option<mir_types::Location>)> {
        let db = self.snapshot_db();
        crate::db::workspace_functions(&db)
            .iter()
            .filter_map(|fqn| {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                crate::db::find_function(&db, here).map(|f| (fqn.clone(), f.location.clone()))
            })
            .collect()
    }

    /// Compute `file`'s outgoing dependency edges and persist them to the
    /// disk cache's reverse-dep graph (if configured). The in-memory graph
    /// is no longer maintained imperatively: `dependency_graph()` derives
    /// structural edges from the memoized [`crate::db::file_structural_deps`]
    /// tracked query, so there is no second copy to drift out of sync.
    pub(super) fn update_reverse_deps_for(&self, file: &str) {
        if let Some(cache) = self.cache.as_deref() {
            let db = self.snapshot_db();
            let targets = file_outgoing_dependencies(&db, file);
            cache.update_reverse_deps_for_file(file, &targets);
        }
    }

    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// O(edges) — iterates the `file_references` forward index (file → symbol
    /// keys it references) which is always current, then resolves each symbol
    /// to its defining file via O(1) lookup.  Total cost is O(E) where E is the
    /// number of (file, symbol) reference edges, vs. the old O(F × S × R) scan.
    pub fn dependency_graph(&self) -> crate::DependencyGraph {
        let db = self.snapshot_db();

        let all_files: Vec<String> = db
            .source_file_paths()
            .iter()
            .map(|f| f.as_ref().to_string())
            .collect();

        let mut dependencies: HashMap<String, Vec<String>> = HashMap::default();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::default();

        for file in &all_files {
            // O(degree(file)) — forward index lookup, no full-table scan.
            let symbol_keys = db.file_referenced_symbols(file);
            let mut file_deps: HashSet<String> = HashSet::default();
            for symbol_key in &symbol_keys {
                let lookup: &str = match symbol_key.split_once("::") {
                    Some((class, _)) => class,
                    None => symbol_key.as_ref(),
                };
                if let Some(def_file) = db.symbol_defining_file(lookup) {
                    let def = def_file.as_ref().to_string();
                    if &def != file {
                        file_deps.insert(def);
                    }
                }
            }
            for dep in &file_deps {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(file.clone());
                dependencies
                    .entry(file.clone())
                    .or_default()
                    .push(dep.clone());
            }
        }

        // Merge structural deps derived from definition collection. The
        // forward pass above only captures bare-FQN references recorded
        // during body analysis; `file_structural_deps` covers imports, class
        // hierarchy (extends/implements/use), and type-hint-only references
        // that never appear in file_referenced_symbols. The query is salsa-
        // memoized, so the warm rebuild costs one map lookup per file rather
        // than a definition walk — and there is no imperatively-maintained
        // reverse map to drift out of sync with the definitions.
        for file in &all_files {
            let Some(sf) = db.lookup_source_file(file) else {
                continue;
            };
            for target in crate::db::file_structural_deps(&db, sf).iter() {
                let target = target.as_ref().to_string();
                if &target != file {
                    dependents
                        .entry(target.clone())
                        .or_default()
                        .push(file.clone());
                    dependencies.entry(file.clone()).or_default().push(target);
                }
            }
        }

        for deps in dependents.values_mut() {
            deps.sort();
            deps.dedup();
        }
        for deps in dependencies.values_mut() {
            deps.sort();
            deps.dedup();
        }

        // Augment with stale dependents: files referencing symbols that were
        // deleted from their defining file. These edges disappear from the
        // symbol_defining_file lookup but the referencing file still needs
        // re-analysis to surface the now-broken reference.
        {
            let stale = self.stale_defined_symbols.read();
            if !stale.is_empty() {
                for (file, deleted_syms) in stale.iter() {
                    for sym in deleted_syms {
                        let lookup: &str = match sym.split_once("::") {
                            Some((class, _)) => class,
                            None => sym.as_ref(),
                        };
                        for referencing_file in db.symbol_referencers_of(lookup) {
                            let ref_file = referencing_file.as_ref().to_string();
                            if &ref_file != file {
                                dependents
                                    .entry(file.clone())
                                    .or_default()
                                    .push(ref_file.clone());
                                dependencies.entry(ref_file).or_default().push(file.clone());
                            }
                        }
                    }
                }
                // Re-sort and dedup since we may have added entries.
                for deps in dependents.values_mut() {
                    deps.sort();
                    deps.dedup();
                }
                for deps in dependencies.values_mut() {
                    deps.sort();
                    deps.dedup();
                }
            }
        }

        crate::DependencyGraph {
            dependencies,
            dependents,
        }
    }
}
