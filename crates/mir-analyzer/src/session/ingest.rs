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

    /// Register or update a [`crate::db::SourceFile`] salsa input and return its
    /// handle, without running definition collection or reference recording.
    ///
    /// The write-path entry point for a host that drives this db's salsa inputs
    /// directly (the LSP database-convergence path) and pulls definitions
    /// lazily via tracked queries, rather than the eager [`Self::ingest_file`].
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn upsert_source_file(
        &self,
        path: Arc<str>,
        text: Arc<str>,
        durability: salsa::Durability,
    ) -> crate::db::SourceFile {
        self.db.upsert_source_file(path, text, durability)
    }

    /// Look up an existing [`crate::db::SourceFile`] handle by path.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn lookup_source_file(&self, path: &str) -> Option<crate::db::SourceFile> {
        self.db.lookup_source_file(path)
    }

    /// Mark a [`crate::db::SourceFile`] as removed from the workspace.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn remove_source_file_input(&self, path: &str) {
        self.db.remove_source_file(path);
    }

    /// Run `f` with exclusive `&mut` access to the shared salsa db, for a host
    /// that owns additional salsa ingredients (inputs/tracked fns) on this db
    /// and needs to create or mutate them. Held under the db write lock, so it
    /// serialises with all other writers.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn with_db_mut<R>(&self, f: impl FnOnce(&mut MirDbStorage) -> R) -> R {
        let mut guard = self.db.salsa.write();
        f(&mut guard)
    }

    /// Run `f` with shared access to the canonical (non-snapshot) salsa db,
    /// under the read lock. For host-owned reads of off-salsa state that must
    /// observe the live db rather than a clone.
    ///
    /// `f` MUST NOT run salsa queries/input reads (tracked fns, `X.field(db)`):
    /// the shared handle has one `ZalsaLocal` query stack, so doing so races any
    /// concurrent salsa read on this handle and aborts the process. Use
    /// [`Self::snapshot_db`] for salsa queries.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn with_db_ref<R>(&self, f: impl FnOnce(&MirDbStorage) -> R) -> R {
        let guard = self.db.salsa.read();
        f(&guard)
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

    /// Replace `file`'s reference postings with `locs` (its complete set from
    /// a fresh single-file analysis) and mark freshness against `text` and
    /// `generation` — both captured before the analysis, so a concurrent
    /// edit or file add leaves the mark stale, which is the safe direction.
    /// `resolved` follows [`Self::mark_ref_committed`]'s contract.
    pub(crate) fn commit_file_refs(
        &self,
        file: &Arc<str>,
        text: Option<Arc<str>>,
        locs: Vec<RefLoc>,
        generation: u64,
        resolved: bool,
    ) {
        {
            let guard = self.db.salsa.read();
            guard.set_file_reference_locations(file.as_ref(), locs);
        }
        if let Some(text) = text {
            // No memoized output on the imperative path — the empty weak
            // handle makes the next re-analysis sweep recommit once (and
            // record the real memo), which is the safe direction.
            self.mark_ref_committed(file, &text, None, generation, resolved);
        }
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

        // The symbols this file defined as of its last ingest. Read from the
        // explicit `last_ingested_symbols` map rather than re-deriving via
        // `file_defined_symbols` (a salsa query on the `SourceFile` input):
        // when a host drives the db directly it may have already updated that
        // input to the new text, which would make a re-derived "old" set equal
        // the new set and silently drop deletions.
        let old_symbols: HashSet<Arc<str>> = self
            .last_ingested_symbols
            .read()
            .get(file.as_ref())
            .cloned()
            .unwrap_or_default();

        {
            let mut guard = self.db.salsa.write();
            guard.remove_file_definitions(file.as_ref());
        }
        let file_defs =
            self.db
                .collect_and_ingest_file(file.clone(), source.as_ref(), self.php_version);

        // Derive this file's defined symbols from the `FileDefinitions` just
        // computed above — do NOT re-read them via a salsa query on the shared
        // `.salsa.read()` handle. That query (`collect_file_definitions`) borrows
        // the handle's single `ZalsaLocal` query stack, so two concurrent
        // `ingest_file` calls doing it would race and abort the process under
        // debug assertions. Reusing `file_defs` needs no db access at all.
        let new_symbols: HashSet<Arc<str>> = file_defs.defined_symbols();
        self.last_ingested_symbols
            .write()
            .insert(file.as_ref().to_string(), new_symbols.clone());

        // Symbols removed from this file must be tracked so dependency_graph()
        // can still produce edges to files referencing the now-gone symbols.
        let deleted: Vec<Arc<str>> = old_symbols.difference(&new_symbols).cloned().collect();
        let re_added: Vec<Arc<str>> = new_symbols.difference(&old_symbols).cloned().collect();
        if !deleted.is_empty() {
            // A deleted symbol may unshadow a lazy-loadable one (e.g. a vendor
            // class with the same FQCN); prepared files must re-run warm-up.
            self.bump_prepare_generation();
        }
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
        if !re_added.is_empty() {
            // A newly-defined symbol may resolve references other files'
            // commits left unresolved; advance the workspace generation so
            // their freshness passes re-verify. New-file registration bumps
            // on its own — this covers definitions appearing in an
            // already-registered file (edits, `set_file_text` lazy loads).
            self.db.salsa.write().bump_workspace_revision();
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

        // Keep the inverted indexes in step with the edit. Class edges come
        // straight from the definitions just collected; reference postings
        // for the new text are recomputed lazily (analysis has not run yet),
        // so the file's freshness mark is dropped rather than updated.
        {
            let entries = crate::db::subtype_index::entries_from_slice(&file_defs.slice);
            let guard = self.db.salsa.read();
            guard.set_file_class_edges(&file, entries);
        }
        // Freshness is keyed on the Arc actually stored on the input (the
        // upsert keeps the prior Arc when content is equal), so read it back.
        let stored_text = {
            let db = self.snapshot_db();
            db.lookup_source_file(file.as_ref())
                .map(|sf| sf.text(&db as &dyn MirDatabase).clone())
        };
        if let Some(text) = stored_text {
            self.mark_defs_committed(&file, &text);
        }
        // `remove_file_definitions` above cleared the file's postings, so the
        // freshness mark must drop unconditionally — even for unchanged text —
        // or a query would trust the now-empty posting lists.
        self.forget_ref_committed(file.as_ref());
    }

    /// [`Self::ingest_file`] followed by the file's Phase-1 warm-up
    /// ([`Self::prepare_file_for_analysis`]): its direct class references are
    /// resolved and lazy-loaded *now*, at write time, instead of serially at
    /// the front of the next references / re-analysis read.
    ///
    /// This is the host edit-path entry point (rust-analyzer's discipline:
    /// mutation happens only when text changes; requests are pure reads).
    /// Lazy loads triggered by the warm-up go through plain
    /// [`Self::ingest_file`], so faulting in a dependency never cascades into
    /// preparing *its* dependencies — the load frontier stays one file wide.
    pub fn ingest_file_prepared(&self, file: Arc<str>, source: Arc<str>) {
        self.ingest_file(file.clone(), source);
        self.prepare_file_for_analysis(&file);
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
            (sf, crate::db::collect_file_declarations(db, sf).clone())
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

    /// Authoritative full rebuild of the workspace symbol index. Call once
    /// after the consumer has pumped every [`Self::index_batch`] chunk (end of
    /// warm-up) to reconcile the incrementally-merged index against the full
    /// registered set. Cheap after indexing — every file's declarations are
    /// already cached.
    pub fn finalize_index(&self) {
        self.db.salsa.write().rebuild_workspace_symbol_index();
    }

    /// Replay disk-cached reference-location postings and subtype-index class
    /// edges for `files`, so a returning session's find-references /
    /// goto-implementation queries are answered from the index immediately
    /// instead of paying the on-demand analysis sweep the first time each
    /// file is queried (`indexed_references_to`/`indexed_subtype_classes`'s
    /// freshness pass already handles a miss correctly — this only shortens
    /// the common warm-start case).
    ///
    /// A no-op (per file) unless the disk cache from a *previous* run has an
    /// entry whose content hash matches `files`' current text: [`Self::with_cache`]/
    /// [`Self::with_cache_dir`] must be attached, and each file's reference
    /// locations ([`AnalysisCache`], populated by the CLI batch pipeline) or
    /// definitions ([`crate::stub_cache::StubSliceCache`], populated by
    /// [`Self::ingest_file`]/vendor ingestion) must already be on disk from
    /// some earlier run/tool invocation against this exact content. A first-
    /// ever run (nothing cached yet) is unaffected — every file simply falls
    /// through to the existing lazy on-demand paths, same as without this call.
    ///
    /// Registers `files` as `Durability::HIGH` salsa inputs (like
    /// [`Self::index_batch`]) if not already registered. Safe to call
    /// alongside `index_batch` in any order; both merge into the same
    /// maintained indexes.
    pub fn warm_start_files(&self, files: &[(Arc<str>, Arc<str>)]) {
        let Some(cache) = self.cache.clone() else {
            return;
        };
        let stub_cache = self.db.stub_cache.clone();
        let php_v = self.php_version.cache_byte();

        {
            let mut guard = self.db.salsa.write();
            for (file, text) in files {
                guard.upsert_source_file_with_durability(
                    file.clone(),
                    text.clone(),
                    salsa::Durability::HIGH,
                );
            }
        }

        // Generation after registration: replayed postings reflect a *prior*
        // session's workspace, so any later file/symbol add must re-verify
        // them (the None-output mark below also disables resolved immunity).
        let commit_gen = self.index_generation();

        for (file, _) in files {
            // Freshness is keyed on the Arc actually stored on the input — an
            // upsert against already-registered, content-equal text keeps the
            // prior Arc (see `ingest_file`), so read back what's really there
            // rather than assume identity with the `text` passed in above.
            let stored_text = {
                let db = self.snapshot_db();
                db.lookup_source_file(file.as_ref())
                    .map(|sf| sf.text(&db as &dyn MirDatabase).clone())
            };
            let Some(stored_text) = stored_text else {
                continue;
            };

            let hex = crate::cache::hash_content(&stored_text);
            if let Some((issues, ref_locs)) = cache.get(file, &hex) {
                let locs: Vec<RefLoc> = ref_locs
                    .iter()
                    .map(|(symbol, line, col_start, col_end)| RefLoc {
                        symbol_key: Arc::clone(symbol),
                        file: file.clone(),
                        line: *line,
                        col_start: *col_start,
                        col_end: *col_end,
                    })
                    .collect();
                // Resolved from the cached issue set: a fully-resolved replay
                // survives the registrations/lazy loads that follow warm-up
                // instead of being invalidated by the first generation bump.
                let resolved = !crate::db::issues_have_unresolved_names(&issues);
                self.commit_file_refs(file, Some(stored_text.clone()), locs, commit_gen, resolved);
            }

            if let Some(stub_cache) = &stub_cache {
                let hash = crate::stub_cache::hash_source(&stored_text);
                if let Some(mut slice) = stub_cache.get(file, &hash, php_v) {
                    crate::stub_cache::prepare_for_ingest(&mut slice);
                    let entries = crate::db::subtype_index::entries_from_slice(&slice);
                    {
                        let guard = self.db.salsa.read();
                        guard.set_file_class_edges(file, entries);
                    }
                    self.mark_defs_committed(file, &stored_text);
                }
            }
        }
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
            guard.clear_file_class_edges(file);
        }
        self.forget_ref_committed(file);
        self.forget_defs_committed(file);
        // Outgoing structural edges disappear from the derived graph
        // automatically: the file is no longer in `source_file_paths()`, so
        // `dependency_graph()` stops iterating it.
        // Clear stale symbol tracking for this file — it's fully gone.
        self.stale_defined_symbols.write().remove(file);
        self.last_ingested_symbols.write().remove(file);
        // Declarations this file provided are gone; other prepared files may
        // now need their warm-up re-run to lazy-load replacements.
        self.forget_prepared(file);
        self.bump_prepare_generation();
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
