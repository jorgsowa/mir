use super::*;

impl AnalysisSession {
    /// Cheap clone of the salsa db for a read-only query. The lock is held
    /// only for the duration of the clone, so concurrent readers never
    /// serialize on each other or on writes for longer than the clone itself.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice. Public consumers should use the typed query methods
    /// ([`Self::definition_of`], [`Self::hover`], etc.) instead.
    #[doc(hidden)]
    pub fn snapshot_db(&self) -> MirDbStorage {
        self.db.snapshot_db()
    }

    /// Commit a batch of reference locations from a db snapshot into the
    /// session's shared maps.  Called by [`crate::FileAnalyzer`] and
    /// [`crate::BatchFileAnalyzer`] after parallel body analysis to flush the pending
    /// buffers that accumulate in worker db clones.
    pub(crate) fn commit_ref_locs_batch(&self, locs: Vec<RefLoc>) {
        if locs.is_empty() {
            return;
        }
        let guard = self.db.salsa.read();
        guard.commit_reference_locations_batch(locs);
    }

    /// Run a closure with read access to a database snapshot.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice.
    #[doc(hidden)]
    pub fn read<R>(&self, f: impl FnOnce(&dyn MirDatabase) -> R) -> R {
        let db = self.snapshot_db();
        f(&db)
    }

    /// definition-collection ingestion. Updates the file's source text in the salsa db,
    /// runs definition collection, and ingests the resulting stub slice.
    /// Triggers stub loading on first call. Also updates the cache's reverse-
    /// dependency graph for `file` so cross-file invalidation stays correct
    /// across incremental edits — without rebuilding the graph from scratch.
    ///
    /// If `file` was previously ingested, its old definitions and reference
    /// locations are removed first so renames / deletions don't leave stale
    /// state in the codebase. (Without this, long-running sessions would
    /// accumulate dead reference-location entries indefinitely.)
    pub fn ingest_file(&self, file: Arc<str>, source: Arc<str>) {
        self.ensure_all_stubs();

        // Snapshot symbols defined before clearing — O(symbols_in_file) with forward index.
        let old_symbols: HashSet<Arc<str>> = {
            let guard = self.db.salsa.read();
            guard.file_defined_symbols(file.as_ref())
        };

        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file.as_ref());
        }
        let _file_defs =
            self.db
                .collect_and_ingest_file(file.clone(), source.as_ref(), self.php_version);

        // Snapshot symbols after ingesting — O(symbols_in_file).
        let new_symbols: HashSet<Arc<str>> = {
            let guard = self.db.salsa.read();
            guard.file_defined_symbols(file.as_ref())
        };

        // Symbols removed from this file must be tracked so dependency_graph()
        // can still produce edges to files referencing the now-gone symbols.
        let deleted: Vec<Arc<str>> = old_symbols.difference(&new_symbols).cloned().collect();
        let re_added: Vec<Arc<str>> = new_symbols.difference(&old_symbols).cloned().collect();
        if !deleted.is_empty() || !re_added.is_empty() {
            let mut stale = self.stale_defined_symbols.write();
            let entry = stale.entry(file.as_ref().to_string()).or_default();
            for sym in deleted {
                entry.insert(sym);
            }
            for sym in &re_added {
                entry.remove(sym);
            }
            if entry.is_empty() {
                stale.remove(file.as_ref());
            }
        }

        self.update_reverse_deps_for(&file);
        // Evict cached analysis results for files that depend on this one so
        // that the next re_analyze_file call re-analyses them rather than
        // replaying a stale cache entry. Mirrors the eviction in
        // `re_analyze_file` (batch.rs) but applies to the ingest path used by
        // LSP servers that edit a single file without re-analysing it.
        if let Some(cache) = self.cache.as_deref() {
            cache.evict_with_dependents(&[file.to_string()]);
        }
        // Only evict cache entries whose resolver-mapped path equals this
        // file. FQCNs the resolver can't map (psr4 miss) stay cached — no
        // ingest could change their fate. Avoids the per-keystroke storm
        // where wholesale clearing forces every unresolved FQCN to re-hit
        // the resolver on the next FileAnalyzer iteration.
        self.evict_unresolvable_for_file(&file);

        // If the workspace symbol index singleton has already been built, keep
        // it consistent with this edit *incrementally*: subtract the file's old
        // declarations and add its new ones (tier-aware). Body-only edits are a
        // no-op inside `update_workspace_index_for_file` (name-only
        // FileDeclarations equality → no singleton write → the HIGH-durability
        // dep does not invalidate body-analysis memos). Only the rare ambiguous
        // case (a removed name still declared by another file, where this file
        // owned the winning entry) falls back to a full O(N) rebuild.
        {
            let mut guard = self.db.salsa.write();
            if guard.workspace_symbol_index_singleton().is_some() {
                if let Some(sf) = guard.lookup_source_file(file.as_ref()) {
                    if !guard.update_workspace_index_for_file(sf) {
                        guard.rebuild_workspace_symbol_index();
                    }
                }
            }
        }
    }

    /// Register `source` as the text of `file` in the salsa input layer **without**
    /// parsing or running definition collection.
    ///
    /// This is the LSP-friendly bulk-population entry point: after a workspace
    /// scan, callers can feed every discovered file's text to the session
    /// cheaply (an Arc clone plus a HashMap insert per file). Name resolution
    /// then happens on demand via [`Self::load_class`], which reads
    /// the file from disk through the configured [`crate::ClassResolver`] and
    /// runs definition collection lazily when a class FQCN actually needs to resolve.
    ///
    /// Contrast with [`Self::ingest_file`], which eagerly parses, runs definition collection,
    /// and populates the symbol index. Use `ingest_file` for files the user is
    /// actively editing (where in-memory text diverges from disk); use
    /// `set_file_text` for files known only through the workspace scan.
    ///
    /// Clears the negative cache: a previously-unresolvable FQCN may now
    /// resolve if its defining file is among the newly-registered set.
    pub fn set_file_text(&self, file: Arc<str>, source: Arc<str>) {
        {
            let mut guard = self.db.salsa.write();
            guard.upsert_source_file(file.clone(), source);
        }
        self.evict_unresolvable_for_file(&file);
    }

    /// Bulk-register vendor / library files with HIGH salsa durability.
    ///
    /// HIGH-durability files are not expected to change during the session.
    /// When a LOW-durability project file is edited, salsa can skip O(N)
    /// dependency verification for every HIGH-durability file, reducing
    /// `workspace_symbol_index` re-verification cost to O(project files only).
    ///
    /// Definition collection runs lazily on first symbol access; no parsing at call time.
    pub fn set_vendor_files<I>(&self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        let mut guard = self.db.salsa.write();
        for (file, source) in files {
            guard.upsert_source_file_with_durability(file, source, salsa::Durability::HIGH);
        }
    }

    /// Build or refresh the `WorkspaceSymbolIndexSingleton` from all currently
    /// registered files.
    ///
    /// After this call, `find_class_like`, `find_function`, and
    /// `find_global_constant` read `singleton.index(db)` — a single
    /// `Durability::HIGH` tracked dep — instead of recomputing the full
    /// O(N_files) dep list via `workspace_symbol_index`. On subsequent
    /// LOW-durability (project-file) body edits the dep short-circuits in O(1).
    ///
    /// Call this once after all vendor + stub + project files have been
    /// ingested (end of workspace warm-up). Also called automatically by
    /// [`Self::ingest_file`] when a file's declared names change.
    pub fn rebuild_workspace_symbol_index(&self) {
        self.db.salsa.write().rebuild_workspace_symbol_index();
    }

    /// Bulk variant of [`Self::set_file_text`]. Acquires the salsa write lock
    /// once for the entire batch instead of once per file.
    ///
    /// The intended LSP scan loop is:
    /// ```text
    /// let files: Vec<_> = walk_workspace()
    ///     .map(|path| (path, fs::read(&path).unwrap()))
    ///     .collect();
    /// session.set_workspace_files(files);
    /// ```
    /// After this call, every file's source text is known to salsa. No
    /// parsing has happened yet — Definition collection runs per file on the first
    /// `load_class` that needs to consult it.
    pub fn set_workspace_files<I>(&self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        let registered_paths: Vec<Arc<str>> = {
            let mut guard = self.db.salsa.write();
            files
                .into_iter()
                .map(|(file, source)| {
                    guard.upsert_source_file(file.clone(), source);
                    file
                })
                .collect()
        };
        if !registered_paths.is_empty() && self.resolver.is_some() {
            self.evict_unresolvable_for_files(&registered_paths);
        }
    }

    /// The workspace generation epoch — the rust-analyzer-style "are we up to
    /// date" counter. Bumped whenever a file is added or removed. A consumer
    /// records this alongside the diagnostics it publishes for a file; when the
    /// value later advances (background indexing registered more files), those
    /// files become candidates for re-analysis + re-publish.
    pub fn index_generation(&self) -> u64 {
        self.db.salsa.read().workspace_revision_value()
    }

    /// Index one bounded chunk of `(path, text)` files — the chunked background
    /// indexing primitive.
    ///
    /// For each chunk this: (1) registers the files as `Durability::HIGH` salsa
    /// inputs in one short write window, (2) parses them to prime the in-process
    /// and on-disk declaration caches (in parallel when `parallelism ==
    /// `[`IndexParallelism::Rayon`]; sequentially for wasm / single-thread
    /// consumers), and (3) merges their declarations into the workspace symbol
    /// index singleton **incrementally** (no full rebuild) so partially-indexed
    /// symbols resolve immediately.
    ///
    /// The library spawns no thread: the consumer pumps chunks from its own
    /// driver (LSP worker thread, or one chunk per wasm event-loop tick),
    /// re-checking higher-priority work between calls. `cancel` is honoured at
    /// chunk boundaries so an edit can abandon queued indexing cheaply.
    ///
    /// **Contract:** index the workspace *incrementally* through this method;
    /// don't bulk-register the entire file set up front and then index — the
    /// first call lazily seeds the singleton from the currently-registered set
    /// (built-in stubs + this chunk), so keeping that initial set small keeps
    /// the first call cheap. Call [`Self::finalize_index`] once after the last
    /// chunk to reconcile authoritatively.
    ///
    /// **Responsiveness:** parsing / declaration collection happens off the
    /// salsa write lock (on a snapshot); only the cheap symbol-map merge runs
    /// under the lock, so the write window per chunk is short and an interactive
    /// read on another thread blocks at most that long. Note that, per salsa's
    /// snapshot model, a *cancellable query* in flight on another thread (e.g.
    /// `hover`, `definition_of`, `FileAnalyzer::analyze`) when this batch takes
    /// the write lock may unwind with `salsa::Cancelled`; a multi-threaded
    /// consumer should catch that and retry the request (the rust-analyzer
    /// pattern). A single-threaded consumer that interleaves requests *between*
    /// `index_batch` calls never observes cancellation.
    pub fn index_batch(
        &self,
        files: &[(Arc<str>, Arc<str>)],
        parallelism: crate::IndexParallelism,
        cancel: &crate::IndexCancel,
    ) -> crate::IndexBatchOutcome {
        if files.is_empty() || cancel.is_cancelled() {
            return crate::IndexBatchOutcome {
                registered: 0,
                cancelled: cancel.is_cancelled(),
                generation: self.index_generation(),
            };
        }
        self.ensure_all_stubs();

        // 1. Register the chunk as HIGH-durability inputs — one short write
        //    window, then release the lock so interactive requests interleave.
        let sources: Vec<crate::db::SourceFile> = {
            let mut guard = self.db.salsa.write();
            files
                .iter()
                .map(|(file, source)| {
                    guard.upsert_source_file_with_durability(
                        file.clone(),
                        source.clone(),
                        salsa::Durability::HIGH,
                    )
                })
                .collect()
        };
        let registered = sources.len();

        if cancel.is_cancelled() {
            return crate::IndexBatchOutcome {
                registered,
                cancelled: true,
                generation: self.index_generation(),
            };
        }

        // Is this the seed chunk (no singleton yet)? If so we must collect decls
        // for the whole currently-registered set (stubs + this chunk); otherwise
        // just this chunk.
        let seed = self
            .db
            .salsa
            .read()
            .workspace_symbol_index_singleton()
            .is_none();
        let snap = self.db.snapshot_db();
        let to_collect: Vec<crate::db::SourceFile> = if seed {
            snap.all_source_files()
        } else {
            sources.clone()
        };

        // 2. Collect per-file declarations OFF the write lock (on a snapshot).
        //    This is where parsing happens — crucially NOT while holding the
        //    write lock, so concurrent interactive reads are not blocked for the
        //    parse duration. Also primes the shared parse/disk caches.
        let collect_one = |db: &crate::db::MirDbStorage, sf: crate::db::SourceFile| {
            (sf, crate::db::collect_file_declarations(db, sf))
        };
        let decls: Vec<(crate::db::SourceFile, crate::db::FileDeclarations)> =
            if parallelism == crate::IndexParallelism::Rayon {
                use rayon::prelude::*;
                to_collect
                    .par_iter()
                    .map_with(snap.clone(), |db, &sf| collect_one(db, sf))
                    .collect()
            } else {
                to_collect
                    .iter()
                    .map(|&sf| collect_one(&snap, sf))
                    .collect()
            };
        drop(snap);

        if cancel.is_cancelled() {
            return crate::IndexBatchOutcome {
                registered,
                cancelled: true,
                generation: self.index_generation(),
            };
        }

        // 3. Apply to the singleton under a SHORT write window — only cheap map
        //    construction / merge runs here (no parse).
        {
            let mut guard = self.db.salsa.write();
            if guard.workspace_symbol_index_singleton().is_none() {
                guard.build_workspace_index_from_decls(decls);
            } else {
                guard.merge_precomputed_into_workspace_index(&decls);
            }
        }

        crate::IndexBatchOutcome {
            registered,
            cancelled: cancel.is_cancelled(),
            generation: self.index_generation(),
        }
    }

    /// Index every `autoload.files` entry from the attached PSR-4 map.
    ///
    /// Composer's `autoload.files` lists files that define global functions and
    /// constants (e.g. Laravel's `Arr::accessible` helpers). Unlike PSR-4
    /// classes, these are not reachable via the class resolver — they must be
    /// parsed and indexed up-front or calls to those functions will produce
    /// false-positive `UndefinedFunction` diagnostics.
    ///
    /// Call this once after [`Self::with_psr4`] and before your project-file
    /// [`Self::index_batch`] pass. Reads source from disk. No-op when no PSR-4
    /// map is attached.
    pub fn index_vendor_eager_files(
        &self,
        parallelism: crate::IndexParallelism,
        cancel: &crate::IndexCancel,
    ) -> crate::IndexBatchOutcome {
        let Some(psr4) = &self.psr4 else {
            return crate::IndexBatchOutcome {
                registered: 0,
                cancelled: false,
                generation: self.index_generation(),
            };
        };
        let files: Vec<(Arc<str>, Arc<str>)> = psr4
            .vendor_eager_files()
            .into_iter()
            .filter_map(|p| {
                let text = std::fs::read_to_string(&p).ok()?;
                Some((
                    Arc::from(p.to_string_lossy().as_ref()),
                    Arc::from(text.as_str()),
                ))
            })
            .collect();
        self.index_batch(&files, parallelism, cancel)
    }

    /// Authoritative full rebuild of the workspace symbol index. Call once
    /// after the consumer has pumped every [`Self::index_batch`] chunk (end of
    /// warm-up) to reconcile the incrementally-merged index against the full
    /// registered set. Cheap after indexing — every file's declarations are
    /// already cached.
    pub fn finalize_index(&self) {
        self.db.salsa.write().rebuild_workspace_symbol_index();
    }

    /// Drop a file's contribution to the session: codebase definitions,
    /// reference locations, salsa input handle, cache entry, and outgoing
    /// reverse-dependency edges. Cache entries of *dependent* files are
    /// also evicted (cross-file invalidation).
    ///
    /// Use this when a file is closed by the consumer, or before a re-ingest
    /// of substantially changed content. (Plain re-ingest via
    /// [`Self::ingest_file`] also drops old definitions, but does not
    /// remove the salsa input handle — call this for full cleanup.)
    pub fn invalidate_file(&self, file: &str) {
        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file);
            guard.remove_source_file(file);
        }
        // Outgoing structural edges disappear from the derived graph
        // automatically: the file is no longer in `source_file_paths()`, so
        // `dependency_graph()` stops iterating it.
        // Clear stale symbol tracking for this file — it's fully gone.
        self.stale_defined_symbols.write().remove(file);
        if let Some(cache) = &self.cache {
            cache.update_reverse_deps_for_file(file, &HashSet::default());
            cache.evict_with_dependents(&[file.to_string()]);
        }
        // The file is gone; cache entries that previously mapped to it stay
        // unresolvable until the file (or another with matching symbols) is
        // ingested again. Selective evict mirrors the ingest path.
        self.evict_unresolvable_for_file(file);
        // Vendor files are static in the eager-index model — closing a project
        // buffer never evicts them (no per-file pinning). Memory is bounded by
        // the LRU on `collect_file_definitions` and the parse cache instead.
    }

    /// Number of files currently tracked in this session's salsa input set.
    /// Stable across reads; useful for diagnostics and memory bounds checks.
    pub fn tracked_file_count(&self) -> usize {
        let guard = self.db.salsa.read();
        guard.source_file_count()
    }

    // -----------------------------------------------------------------------
    // Read-only codebase queries
    //
    // All take a brief lock to clone the db, then run the lookup against the
    // owned snapshot — concurrent edits proceed without blocking.
    // -----------------------------------------------------------------------
}
