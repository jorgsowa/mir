use super::*;

impl AnalysisSession {
    /// Deprecated — stub loading is now fully lazy per-AST.
    ///
    /// This is an alias for [`Self::ensure_all_stubs`] kept for API
    /// compatibility. Internal analysis paths use [`Self::prepare_ast_for_analysis`]
    /// which loads only the stubs referenced by the file under analysis.
    #[deprecated(note = "use ensure_all_stubs() or ensure_stubs_for_ast() instead")]
    pub fn ensure_essential_stubs(&self) {
        self.ensure_all_stubs();
    }

    /// Load every embedded PHP stub plus any configured user stubs.
    /// Use for batch tools (CLI, full project analysis) where comprehensive
    /// symbol coverage matters more than cold-start latency.
    pub fn ensure_all_stubs(&self) {
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.db.ingest_stub_paths(&paths, self.php_version);
        self.ensure_user_stubs_loaded();
    }

    /// Ensure the embedded stub that defines `name` (a function) is ingested.
    /// Returns `true` when a matching stub exists (whether or not it was
    /// already loaded), `false` when `name` isn't a known PHP built-in.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead —
    /// it auto-discovers needed stubs from a parsed file.
    #[doc(hidden)]
    pub fn ensure_stub_for_function(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_function(name) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `fqcn` (a class / interface /
    /// trait / enum) is ingested. Case-insensitive lookup with optional
    /// leading backslash.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead.
    #[doc(hidden)]
    pub fn ensure_stub_for_class(&self, fqcn: &str) -> bool {
        match crate::stubs::stub_path_for_class(fqcn) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Ensure the embedded stub that defines `name` (a constant) is ingested.
    ///
    /// Most callers should use [`Self::ensure_stubs_for_ast`] instead.
    #[doc(hidden)]
    pub fn ensure_stub_for_constant(&self, name: &str) -> bool {
        match crate::stubs::stub_path_for_constant(name) {
            Some(path) => {
                self.db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Number of distinct embedded stubs currently ingested into the session.
    /// Useful for diagnostics and bench reporting.
    pub fn loaded_stub_count(&self) -> usize {
        self.db.loaded_stubs.lock().len()
    }

    /// Auto-discover and ingest the embedded stubs needed to cover every
    /// built-in PHP function / class / constant referenced by `source`.
    ///
    /// Used by [`crate::FileAnalyzer::analyze`] to keep essentials-only mode
    /// correct without forcing callers to enumerate which stubs they need.
    /// Idempotent — already-loaded stubs are skipped via [`Self::loaded_stubs`].
    ///
    /// The discovery scan is a coarse identifier sweep (see
    /// [`crate::stubs::collect_referenced_builtin_paths`]) — it may pull in
    /// a slightly larger set than the file strictly needs, but never misses
    /// a referenced built-in. Cost is sub-millisecond per file.
    ///
    /// Fast path: if every embedded stub is already loaded (e.g. after a
    /// batch tool called [`Self::ensure_all_stubs`]), the source scan
    /// is skipped entirely.
    pub fn ensure_stubs_for_source(&self, source: &str) {
        // Cheap check first: skip the scan entirely when we already know we
        // have everything. Avoids a ~50-500µs source walk on every analyze
        // call in batch / warm-session scenarios.
        {
            let loaded = self.db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths(source);
        if paths.is_empty() {
            return;
        }
        self.db.ingest_stub_paths(&paths, self.php_version);
    }

    /// Discover and ingest stubs by walking the parsed AST of a PHP file.
    ///
    /// Similar to [`Self::ensure_stubs_for_source`], but takes an already-parsed
    /// AST instead of raw source text. Produces zero false positives since it
    /// only extracts identifiers from actual AST nodes (not from strings or
    /// comments). Preferred over `ensure_stubs_for_source` when the AST is
    /// already available (e.g., in [`crate::FileAnalyzer`]).
    ///
    /// Idempotent and skips the scan if all stubs are already loaded.
    pub fn ensure_stubs_for_ast(&self, program: &php_ast::owned::Program) {
        {
            let loaded = self.db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths_from_ast(program);
        if paths.is_empty() {
            return;
        }
        self.db.ingest_stub_paths(&paths, self.php_version);
    }

    /// Returns true if this session has a configured class resolver
    /// (typically a PSR-4 / classmap autoloader chained with the stub
    /// resolver). Used by `FileAnalyzer` to skip the AST-scan preload
    /// when no resolver is wired up.
    pub fn has_resolver(&self) -> bool {
        self.resolver.is_some()
    }

    /// Index vendor `autoload.files` entries the first time analysis runs.
    ///
    /// Composer's `autoload.files` lists files that define global functions and
    /// constants (e.g. Laravel helpers). These are invisible to the PSR-4 class
    /// resolver — there is no function-name → file-path mapping without
    /// parsing them first.  Rather than per-function lazy resolution, this
    /// loads all pending vendor eager files at once on the first
    /// [`Self::prepare_ast_for_analysis`] call.
    ///
    /// The mutex is held for the duration of the load, so concurrent callers
    /// block here until the files are indexed.  Subsequent calls see `None`
    /// and return immediately (O(1)).  Files are read via the session's
    /// [`crate::SourceProvider`], so LSP VFS overrides are respected.
    pub(crate) fn ensure_vendor_eager_functions(&self) {
        let mut guard = self.pending_eager_function_files.lock();
        let files = match guard.take() {
            None => return,
            Some(f) if f.is_empty() => return,
            Some(f) => f,
        };
        // Guard remains held (now `None`) — concurrent callers block here
        // until `index_batch` returns and all functions are indexed.
        let sources: Vec<(std::sync::Arc<str>, std::sync::Arc<str>)> = files
            .iter()
            .filter_map(|p| {
                let text = self.source_provider.read(p.to_string_lossy().as_ref())?;
                Some((std::sync::Arc::from(p.to_string_lossy().as_ref()), text))
            })
            .collect();
        if !sources.is_empty() {
            let cancel = crate::IndexCancel::new();
            self.index_batch(&sources, crate::IndexParallelism::Sequential, &cancel);
        }
    }

    /// Run all pre-passes (builtin-stub loading, vendor-eager-file loading,
    /// and PSR-4 class preloading) before body analysis of a single file.
    ///
    /// Replaces the two separate `ensure_stubs_for_ast` /
    /// `preload_psr4_classes_for_ast` calls at every `FileAnalyzer::analyze`
    /// site.
    pub fn prepare_ast_for_analysis(&self, program: &php_ast::owned::Program, file: &str) {
        self.ensure_stubs_for_ast(program);
        self.ensure_vendor_eager_functions();
        self.priority_index_for_ast(program, file);
    }

    /// Run `file`'s Phase-1 warm-up — resolve its direct class references and
    /// lazy-load any not yet indexed — and record it as prepared against its
    /// current text. No-op when the file is unknown or already prepared.
    ///
    /// This is the write-path home of the warm-up: hosts call it (via
    /// [`Self::ingest_file_prepared`]) when text lands, so read paths
    /// (`references_to_in_files`, `reanalyze_files_cancellable`) find every
    /// candidate prepared and stay pure. Loading mutates salsa inputs, so the
    /// parse snapshot is scoped and dropped before the warm-up runs — callers
    /// must not hold a live snapshot across this call.
    pub fn prepare_file_for_analysis(&self, path: &std::sync::Arc<str>) {
        let generation = self.prepare_generation_snapshot();
        let (parsed, text) = {
            let db = self.snapshot_db();
            let Some(sf) = db.lookup_source_file(path.as_ref()) else {
                return;
            };
            let text = sf.text(&db as &dyn crate::db::MirDatabase).clone();
            if self.is_prepared_for_analysis(path.as_ref(), &text, generation) {
                return;
            }
            (
                crate::db::parse_file(&db as &dyn crate::db::MirDatabase, sf)
                    .0
                    .clone(),
                text,
            )
        };
        self.prepare_ast_for_analysis(&parsed.program, path.as_ref());
        self.mark_prepared_for_analysis(path, text, generation);
    }

    /// Current warm-up generation; capture before a prepare + mark pair so a
    /// concurrent [`Self::bump_prepare_generation`] invalidates the mark.
    pub(crate) fn prepare_generation_snapshot(&self) -> u64 {
        self.prepare_generation
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// Whether `file`'s warm-up already ran against `current_text` in
    /// `generation`. See the `prepared_files` field docs for the invalidation
    /// rules.
    pub(crate) fn is_prepared_for_analysis(
        &self,
        file: &str,
        current_text: &std::sync::Arc<str>,
        generation: u64,
    ) -> bool {
        self.prepared_files
            .read()
            .get(file)
            .is_some_and(|(text, prepared_gen)| {
                *prepared_gen == generation && std::sync::Arc::ptr_eq(text, current_text)
            })
    }

    /// Record that `file`'s warm-up ran against `text`. `generation` must be
    /// the [`Self::prepare_generation_snapshot`] taken *before* the warm-up —
    /// a bump in between leaves the entry stale, which is the safe direction.
    pub(crate) fn mark_prepared_for_analysis(
        &self,
        file: &std::sync::Arc<str>,
        text: std::sync::Arc<str>,
        generation: u64,
    ) {
        self.prepared_files
            .write()
            .insert(file.clone(), (text, generation));
    }

    /// Drop `file`'s warm-up skip entry (its loaded state is being removed).
    pub(crate) fn forget_prepared(&self, file: &str) {
        self.prepared_files.write().remove(file);
    }

    /// Invalidate every warm-up skip entry. Call when previously loaded
    /// declarations may have been removed outside [`Self::ingest_file`] /
    /// [`Self::invalidate_file`] — e.g. a host that writes file text straight
    /// into the salsa layer and detects a declaration-level change. A prepared
    /// file might then need its warm-up re-run to lazy-load a replacement
    /// (a vendor class shadowed by a since-deleted project class).
    pub fn bump_prepare_generation(&self) {
        self.prepare_generation
            .fetch_add(1, std::sync::atomic::Ordering::Release);
    }

    /// Priority-index the classes directly referenced by `file`'s AST.
    ///
    /// In the eager-static-input model the background indexer
    /// ([`Self::index_batch`]) walks the whole vendor tree, but it may not have
    /// reached every file the open buffer references yet. To avoid a transient
    /// false `UndefinedClass` during the warm-up window, this **reorders** that
    /// static work: it resolves the buffer's *direct* class references and
    /// loads any not-yet-indexed ones immediately, jumping them to the front of
    /// the queue.
    ///
    /// This is bounded by the number of distinct direct references in **one**
    /// file — no transitive BFS, no depth/total budget, no pinning. Inheritance
    /// ancestors and signature types of those classes are picked up by the
    /// background walk (or, for navigation, by [`Self::hover`] /
    /// [`Self::definition_of`]). Because `bump_workspace_revision` no longer
    /// nulls the workspace index singleton, each [`Self::load_class`] here costs
    /// only a resolver lookup + parse (or cache hit) + one tier-aware merge,
    /// invalidating just the actively-analyzed file's memo once — not the whole
    /// cache. Once background indexing completes this is a no-op (every
    /// reference already resolves).
    pub fn priority_index_for_ast(&self, program: &php_ast::owned::Program, file: &str) {
        if self.resolver.is_none() {
            return;
        }
        let refs = collect_class_refs_from_ast(program);
        if refs.is_empty() {
            return;
        }
        // Resolve names against the file's namespace/imports up front, then
        // drop the snapshot before loading (which mutates inputs).
        let resolved: Vec<String> = {
            let db = self.snapshot_db();
            refs.into_iter()
                .map(|raw| crate::db::resolve_name(&db, file, &raw))
                .collect()
        };
        for fqcn in resolved {
            // load_class is a no-op when the class is already indexed (the
            // common case once the background walk has passed this file).
            self.load_class(&fqcn);
        }
    }

    fn ensure_user_stubs_loaded(&self) {
        self.db
            .ingest_user_stubs(&self.user_stub_files, &self.user_stub_dirs);
    }
}
