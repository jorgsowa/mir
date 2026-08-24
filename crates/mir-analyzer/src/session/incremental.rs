use super::*;

impl AnalysisSession {
    /// Retrieve the source text the session has registered for `file`, if
    /// any. Returns `None` when the file has never been ingested. Used by
    /// the parallel re-analysis path to re-feed dependents to body analysis without
    /// the caller having to track sources independently.
    pub fn source_of(&self, file: &str) -> Option<Arc<str>> {
        let db = self.snapshot_db();
        let sf = db.lookup_source_file(file)?;
        Some(sf.text(&db).clone())
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
        self.reanalyze_file_set(dependents, cancel)
    }

    /// Re-analyze an explicit file set — typically the editor's currently
    /// open files — after an edit elsewhere in the workspace.
    ///
    /// This is the rust-analyzer diagnostics model: instead of computing the
    /// edited file's transitive dependents (an O(all-ingested-files) graph
    /// rebuild on every keystroke), the caller passes the handful of files it
    /// actually publishes diagnostics for, and salsa memoization makes the
    /// unaffected ones ~free — `analyze_file` re-validates each file's memo
    /// against what actually changed and only re-executes bodies the edit
    /// reaches. Per-edit cost is O(open files), independent of workspace size.
    ///
    /// Files the session has no source for are silently skipped. Cancellation
    /// semantics match [`Self::reanalyze_dependents_cancellable`].
    pub fn reanalyze_files_cancellable(
        &self,
        files: &[Arc<str>],
        cancel: &crate::IndexCancel,
    ) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        if cancel.is_cancelled() || files.is_empty() {
            return Vec::new();
        }
        self.settle_workspace_index();
        self.reanalyze_file_set(files.to_vec(), cancel)
    }

    /// Shared body of [`Self::reanalyze_dependents_cancellable`] and
    /// [`Self::reanalyze_files_cancellable`]: warm up, analyze in parallel,
    /// commit reference locations.
    fn reanalyze_file_set(
        &self,
        files: Vec<Arc<str>>,
        cancel: &crate::IndexCancel,
    ) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        use rayon::prelude::*;

        let dependents = files;

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
        // `prepare_file_for_analysis` takes a *scoped* snapshot to fetch the
        // parsed AST, drops it (the `Arc<ParseResult>` is owned), and only
        // then warms up. Files already prepared against their current text
        // skip the parse + AST walk entirely — hosts on the
        // `ingest_file_prepared` write path pre-pay this per edit, making the
        // whole loop a map-lookup sweep.
        {
            // One revision bump for the whole warm-up sweep instead of one per
            // lazily-loaded class (each bump cancels every in-flight salsa
            // reader). Closed before `commit_gen` is read below, so commits
            // are stamped with the post-load generation as before.
            let _deferred_bumps = self.defer_revision_bumps();
            for file in &dependents {
                if cancel.is_cancelled() {
                    return Vec::new();
                }
                self.prepare_file_for_analysis(file);
            }
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
        // Generation before the snapshot: a file add racing the sweep leaves
        // the commits stale (self-healing), never wrongly fresh.
        let commit_gen = self.index_generation();
        // Freeze on the pass-scoped snapshot: warm-up (2a) completed every
        // lazy load, and a concurrent index write cancels the pass, so the
        // frozen view is never stale. Same discipline as the batch body pass.
        let mut db_main = self.snapshot_db();
        db_main.freeze_workspace_index();
        // Sweeps are the steady-state population path for the mention index:
        // every analyzed file gets a current mention scan alongside its
        // postings, so later reference-gate checks are set lookups.
        let mention_scanner = db_main.class_mention_scanner();
        type Analyzed = (
            Arc<str>,
            Arc<str>,
            std::sync::Arc<crate::db::AnalyzeOutput>,
            Vec<crate::db::SubtypeEntry>,
            Option<super::RefCachePut>,
            Option<Box<[mir_types::Name]>>,
        );
        let mut results: Vec<Analyzed> = dependents
            .into_par_iter()
            .map_with(db_main, |db, file| {
                if cancel.is_cancelled() {
                    return None;
                }
                let sf = db.lookup_source_file(file.as_ref())?;
                // Capture the text the analysis ran against: the freshness
                // marks below must record exactly this Arc, so a text write
                // racing the sweep leaves the file dirty rather than
                // wrongly marked fresh.
                let text = sf.text(&*db as &dyn crate::db::MirDatabase).clone();
                let out = crate::db::analyze_file(&*db as &dyn crate::db::MirDatabase, sf).clone();
                let defs =
                    crate::db::collect_file_definitions(&*db as &dyn crate::db::MirDatabase, sf);
                let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                // Stage the disk-cache write only when the postings commit
                // below will actually rewrite — a no-op re-sweep (current
                // commit) adds no hashing or parse-walk cost per file.
                let put = if self.ref_commit_is_current(file.as_ref(), &text, &out) {
                    None
                } else {
                    self.stage_ref_cache_put(
                        &*db as &dyn crate::db::MirDatabase,
                        sf,
                        file.as_ref(),
                        &text,
                        &out,
                    )
                };
                let mentions = mention_scanner.as_ref().and_then(|s| {
                    (!db.class_mentions_current(file.as_ref(), &text, s.epoch()))
                        .then(|| s.scan(&text))
                });
                Some((file, text, out, entries, put, mentions))
            })
            .flatten()
            .collect();

        // Serial commit: each dependent's output is its complete reference
        // set, so replace rather than append. Both inverted indexes and their
        // freshness marks update here — this is what keeps read queries
        // lookup-shaped instead of re-validating every candidate memo.
        // Unchanged files (same text, same memoized output) skip the rebuild
        // entirely, so a no-op re-sweep is a pointer compare per file.
        {
            let guard = self.db.salsa.read();
            for (file, text, out, entries, put, mentions) in results.iter_mut() {
                // Pointer-identical memo ⇒ identical postings: skip the
                // index rewrite. The mark is re-stamped unconditionally so a
                // no-op sweep still advances the commit's generation.
                if !self.ref_commit_is_current(file.as_ref(), text, out) {
                    guard.set_file_reference_locations(file.as_ref(), out.ref_locs.to_vec());
                }
                if let (Some(s), Some(m)) = (&mention_scanner, mentions.take()) {
                    guard.set_file_class_mentions(file, text, s.epoch(), m);
                }
                if let Some(put) = put.take() {
                    self.apply_ref_cache_put(file.as_ref(), out, put);
                }
                self.mark_ref_committed(
                    file,
                    text,
                    Some(out),
                    commit_gen,
                    !out.has_unresolved_names(),
                );
                if !self.is_defs_committed(file.as_ref(), text) {
                    guard.set_file_class_edges(file, entries.clone());
                    self.mark_defs_committed(file, text);
                }
            }
        }

        results
            .into_iter()
            .map(|(file, _, out, _, _, _)| {
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
        let imports = db.file_class_imports(file);
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
            let here = crate::db::Fqcn::interned(&db, *fqcn);
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
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_str());
                crate::db::find_class_like(&db, here)
                    .map(|class| (Arc::<str>::from(fqcn.as_str()), class.location().cloned()))
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
                let here = crate::db::Fqcn::from_str(&db, fqn.as_str());
                crate::db::find_function(&db, here)
                    .map(|f| (Arc::<str>::from(fqn.as_str()), f.location.clone()))
            })
            .collect()
    }

    /// Every class/interface/trait/enum in the workspace whose *own* short
    /// (unqualified) name is exactly `short_name`, each paired with its FQCN
    /// and declaration location.
    ///
    /// Compatibility helper for callers that surface a bare class name without
    /// first resolving it to a canonical FQCN.
    pub fn classes_named(&self, short_name: &str) -> Vec<(Arc<str>, Option<mir_types::Location>)> {
        let db = self.snapshot_db();
        let key = mir_types::Name::new(short_name).ascii_lowercase();
        crate::db::workspace_index(&db)
            .class_likes_named(key)
            .iter()
            .copied()
            .filter_map(|fqcn_key| {
                let here = crate::db::Fqcn::from_str(&db, fqcn_key.as_str());
                crate::db::find_class_like(&db, here)
                    .map(|class| (class.fqcn().clone(), class.location().cloned()))
            })
            .collect()
    }

    /// Every ancestor of `fqcn` — extended class, implemented interfaces,
    /// used traits, transitively — most-derived first. Does not include
    /// `fqcn` itself. Memoized per FQCN (shares `class_ancestors_by_fqcn`'s
    /// tracked cache with member-resolution paths).
    pub fn ancestors_of(&self, fqcn: &str) -> Vec<Arc<str>> {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::from_str(&db, fqcn.trim_start_matches('\\'));
        crate::db::class_ancestors_by_fqcn(&db, here)
            .iter()
            .skip(1)
            .cloned()
            .collect()
    }

    /// Full signature for a global function, resolved by FQN.
    pub fn function_signature(&self, fqn: &str) -> Option<Arc<crate::FunctionDef>> {
        let db = self.snapshot_db();
        let here = crate::db::Fqcn::from_str(&db, fqn);
        crate::db::find_function(&db, here)
    }

    /// Compute `file`'s outgoing dependency edges and persist them to the
    /// disk cache's reverse-dep graph (if configured). The in-memory graph
    /// is no longer maintained imperatively: `dependency_graph()` derives
    /// structural symbols from the memoized [`crate::db::file_structural_symbols`]
    /// tracked query, then projects them through the workspace symbol index
    /// alongside body-reference symbols.
    pub(super) fn update_reverse_deps_for(&self, file: &str) {
        if let Some(cache) = self.cache.as_deref() {
            let db = self.snapshot_db();
            let targets = file_outgoing_dependencies(&db, file, true);
            cache.update_reverse_deps_for_file(file, &targets);
        }
    }

    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// O(edges) — iterates symbol edges from the reference index and from
    /// `file_structural_symbols`, then resolves each symbol to its defining
    /// file via O(1) lookup. Total cost is O(E) where E is the number of
    /// (file, symbol) edges.
    pub fn dependency_graph(&self) -> crate::DependencyGraph {
        let db = self.snapshot_db();

        fn push_edge(
            dependencies: &mut [Vec<u32>],
            dependents: &mut [Vec<u32>],
            from: u32,
            to: u32,
        ) {
            if from == to {
                return;
            }
            dependents[to as usize].push(from);
            dependencies[from as usize].push(to);
        }

        fn sort_and_dedup(adjacency: &mut [Vec<u32>]) {
            for deps in adjacency {
                deps.sort();
                deps.dedup();
            }
        }

        fn symbol_defining_file_id(
            db: &dyn crate::db::MirDatabase,
            file_ids: &HashMap<Arc<str>, u32>,
            symbol_key: &str,
        ) -> Option<u32> {
            let lookup = crate::defining_file_lookup_key(symbol_key);
            db.symbol_defining_file(lookup)
                .and_then(|file| file_ids.get(file.as_ref()).copied())
        }

        let mut all_files: Vec<Arc<str>> = db.source_file_paths().to_vec();
        all_files.sort();
        assert!(
            u32::try_from(all_files.len()).is_ok(),
            "dependency graph file ids require at most u32::MAX files"
        );
        let file_ids: HashMap<Arc<str>, u32> = all_files
            .iter()
            .enumerate()
            .map(|(id, file)| (file.clone(), id as u32))
            .collect();

        let mut dependencies = vec![Vec::new(); all_files.len()];
        let mut dependents = vec![Vec::new(); all_files.len()];
        for (file_id, file) in all_files.iter().enumerate() {
            let file_id = file_id as u32;
            let mut file_deps: HashSet<u32> = HashSet::default();

            // O(degree(file)) — forward reference-index lookup, no full-table scan.
            for symbol_key in db.file_referenced_symbols(file.as_ref()) {
                if let Some(def_id) = symbol_defining_file_id(&db, &file_ids, &symbol_key) {
                    file_deps.insert(def_id);
                }
            }

            // Declaration-level symbol edges from Salsa. These cover imports,
            // class hierarchy edges, and type-hint-only references that never
            // appear in file_referenced_symbols.
            if let Some(sf) = db.lookup_source_file(file.as_ref()) {
                for symbol in crate::db::file_structural_symbols(&db, sf).iter() {
                    if let Some(def_id) = symbol_defining_file_id(&db, &file_ids, symbol) {
                        file_deps.insert(def_id);
                    }
                }
            }

            for dep_id in file_deps {
                push_edge(&mut dependencies, &mut dependents, file_id, dep_id);
            }
        }

        sort_and_dedup(&mut dependents);
        sort_and_dedup(&mut dependencies);

        // Augment with stale dependents: files referencing symbols that were
        // deleted from their defining file. These edges disappear from the
        // symbol_defining_file lookup but the referencing file still needs
        // re-analysis to surface the now-broken reference.
        {
            let stale = self.stale_defined_symbols.read();
            if !stale.is_empty() {
                for (file, deleted_syms) in stale.iter() {
                    let Some(&file_id) = file_ids.get(file.as_str()) else {
                        continue;
                    };
                    for sym in deleted_syms {
                        let lookup = crate::defining_file_lookup_key(sym);
                        // `defined_symbols()` only yields top-level FQ names
                        // (classes/interfaces/traits/enums, functions, global
                        // constants) — never knows here which kind `sym` was,
                        // so probe every prefix the reference index actually
                        // uses (see `Name::codebase_key`) rather than guessing
                        // one and silently missing referencers of the others.
                        for prefix in ["cls:", "fn:", "gcnst:"] {
                            for referencing_file in
                                db.symbol_referencers_of(&format!("{prefix}{lookup}"))
                            {
                                if let Some(&ref_id) = file_ids.get(referencing_file.as_ref()) {
                                    push_edge(&mut dependencies, &mut dependents, ref_id, file_id);
                                }
                            }
                        }
                    }
                }
                // Re-sort and dedup since we may have added entries.
                sort_and_dedup(&mut dependents);
                sort_and_dedup(&mut dependencies);
            }
        }

        crate::DependencyGraph::from_compact_parts(all_files, file_ids, dependencies, dependents)
    }
}
