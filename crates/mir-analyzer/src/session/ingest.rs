use super::*;

/// One file's disk-cache lookup result from `AnalysisSession::warm_start_files`'s
/// parallel read phase — everything the sequential apply phase needs, so it
/// never has to re-read anything.
struct WarmStartHit {
    file: Arc<str>,
    sf: crate::db::SourceFile,
    stored_text: Arc<str>,
    /// `(reference locations, resolved)` from a cached `AnalysisCache` hit.
    refs: Option<(Vec<RefLoc>, bool)>,
    /// `(subtype edges, declarations)` from a cached stub-slice hit.
    stub: Option<(
        Vec<crate::db::subtype_index::SubtypeEntry>,
        crate::db::FileDeclarations,
    )>,
}

/// Reconciliation rounds `settle_workspace_index_cancellable` performs before handing the pending set to the next caller.
const SETTLE_ROUNDS: usize = 4;

type ChangedInput = (Arc<str>, Arc<str>, bool);
type IndexedSources = (Vec<crate::db::SourceFile>, Vec<ChangedInput>, bool);

/// Existing definitions that lost ownership of a name to this mirror batch.
/// Cached resolved consumers point to the old owner, so invalidate those
/// dependents when a new file shadows it.
fn displaced_cache_owners(
    db: &MirDbStorage,
    old_index: &crate::db::WorkspaceSymbolIndex,
    decls: &[(crate::db::SourceFile, crate::db::FileDeclarations)],
) -> Vec<String> {
    let Some(singleton) = db.workspace_symbol_index_singleton() else {
        return Vec::new();
    };
    let new_index = singleton.index(db);
    let mut owners = HashSet::default();
    for (_, file_decls) in decls {
        for decl in file_decls.class_like() {
            let key = decl.lookup_key();
            if let (Some(old), Some(new)) =
                (old_index.class_like_loc(key), new_index.class_like_loc(key))
            {
                if old.file() != new.file() {
                    owners.insert(old.file().path(db).to_string());
                }
            }
        }
        for decl in file_decls.functions() {
            let key = decl.lookup_key();
            if let (Some(old), Some(new)) =
                (old_index.function_loc(key), new_index.function_loc(key))
            {
                if old.file() != new.file() {
                    owners.insert(old.file().path(db).to_string());
                }
            }
        }
        for decl in file_decls.constants() {
            let key = decl.lookup_key();
            if let (Some(old), Some(new)) =
                (old_index.constant_loc(key), new_index.constant_loc(key))
            {
                if old.file() != new.file() {
                    owners.insert(old.file().path(db).to_string());
                }
            }
        }
    }
    owners.into_iter().collect()
}

impl AnalysisSession {
    /// Cheap clone of the salsa db for a read-only query.
    ///
    /// The handle blocks this session's next input write until dropped.
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
        &mut self,
        path: Arc<str>,
        text: Arc<str>,
        durability: salsa::Durability,
    ) -> crate::db::SourceFile {
        self.index.clear_transient_batch_replay();
        let (sf, changed, was_registered, had_index) = {
            let db = &mut self.db.salsa;
            let had_index = db.workspace_symbol_index_singleton().is_some();
            let existing = db.lookup_source_file(path.as_ref());
            let changed = existing.is_none_or(|sf| sf.text(db).as_ref() != text.as_ref());
            let sf = db.upsert_source_file_with_durability(path.clone(), text.clone(), durability);
            (sf, changed, existing.is_some(), had_index)
        };
        if changed {
            self.index.clear_dependency_graph_cache();
            if let Some(cache) = self.cache.as_deref() {
                let invalidate_path = was_registered || !cache.matches_content(&path, &text);
                if invalidate_path {
                    cache.evict_files_and_dependents(&[path.to_string()]);
                }
                if !had_index {
                    cache.evict_unresolved();
                }
            }
            self.evict_unresolvable_for_file(&path);
        }
        sf
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
    pub fn remove_source_file_input(&mut self, path: &str) {
        self.index.clear_transient_batch_replay();
        self.db.remove_source_file(path);
    }

    /// Run `f` with exclusive `&mut` access to the shared salsa db, for a host
    /// that owns additional salsa ingredients (inputs/tracked fns) on this db
    /// and needs to create or mutate them.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn with_db_mut<R>(&mut self, f: impl FnOnce(&mut MirDbStorage) -> R) -> R {
        self.index.clear_transient_batch_replay();
        f(&mut self.db.salsa)
    }

    /// Run `f` with shared access to the canonical (non-snapshot) salsa db.
    /// For host-owned reads of off-salsa state that must observe the live db
    /// rather than a clone.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without notice.
    #[doc(hidden)]
    pub fn with_db_ref<R>(&self, f: impl FnOnce(&MirDbStorage) -> R) -> R {
        f(&self.db.salsa)
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
    pub fn ingest_file(&mut self, file: Arc<str>, source: Arc<str>) {
        self.ensure_all_stubs();
        let existing_text = self
            .lookup_source_file(file.as_ref())
            .map(|sf| sf.text(&self.db.salsa).clone());
        if existing_text
            .as_ref()
            .is_some_and(|text| text.as_ref() == source.as_ref())
            && self.last_ingested_symbols.contains_key(file.as_ref())
        {
            return;
        }
        self.index.clear_transient_batch_replay();

        // The symbols this file defined as of its last ingest. Read from the
        // explicit `last_ingested_symbols` map rather than re-deriving via
        // `file_defined_symbols` (a salsa query on the `SourceFile` input):
        // when a host drives the db directly it may have already updated that
        // input to the new text, which would make a re-derived "old" set equal
        // the new set and silently drop deletions.
        let old_symbols: HashSet<Arc<str>> = self
            .last_ingested_symbols
            .get(file.as_ref())
            .cloned()
            .unwrap_or_default();
        let old_structural_targets: HashSet<String> = self
            .last_structural_targets
            .get(file.as_ref())
            .cloned()
            .unwrap_or_default();

        // Postings for the new text are recomputed lazily; retiring also
        // refuses commits from snapshots still analyzing the old text.
        self.index.retire_references(&self.db.salsa, file.as_ref());
        let file_defs =
            self.db
                .collect_and_ingest_file(file.clone(), source.as_ref(), self.php_version);
        let (new_decls, stored_text) = {
            let sf = self
                .lookup_source_file(file.as_ref())
                .expect("collect_and_ingest_file must register SourceFile");
            (
                crate::db::decls_from_slice(&file_defs.slice, sf),
                sf.text(&self.db.salsa).clone(),
            )
        };

        // Derive this file's defined symbols from the `FileDefinitions` just
        // computed above rather than re-reading them via a salsa query —
        // `file_defs` already has them, so this needs no db access at all.
        let new_symbols: HashSet<Arc<str>> = file_defs.defined_symbols();
        self.last_ingested_symbols
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
            let stale = &mut self.stale_defined_symbols;
            let entry = stale.entry(file.as_ref().to_string()).or_default();
            for sym in &deleted {
                entry.insert(sym.clone());
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
            self.db.salsa.bump_workspace_revision();
        }

        // Structural edges depend only on declaration-shaped data already
        // committed above, so compute them now and only invalidate the cached
        // dependency graph when those edges actually changed.
        let new_structural_targets =
            file_outgoing_dependencies(&self.db.salsa, file.as_ref(), false);
        self.last_structural_targets
            .insert(file.as_ref().to_string(), new_structural_targets.clone());
        let dependency_graph_changed = old_structural_targets != new_structural_targets
            || !deleted.is_empty()
            || !re_added.is_empty();
        if dependency_graph_changed {
            self.index.clear_dependency_graph_cache();
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
            let db = &mut self.db.salsa;
            if db.workspace_symbol_index_singleton().is_some() {
                if let Some(sf) = db.lookup_source_file(file.as_ref()) {
                    if !db.update_workspace_index_for_file(sf, new_decls.clone()) {
                        db.rebuild_workspace_symbol_index();
                    }
                    db.clear_index_pending(file.as_ref());
                }
            }
        }

        // Class edges come straight from the definitions just collected.
        {
            let entries = crate::db::subtype_index::entries_from_slice(&file_defs.slice);
            let db = &self.db.salsa;
            let file_no = db.locked_ref_index().intern_path(&file);
            db.set_file_class_edges(file_no, entries);
        }
        // Freshness is keyed on the Arc actually stored on the input (the
        // upsert keeps the prior Arc when content is equal), so read it back.
        self.index.mark_defs_committed(&file, &stored_text);
    }

    /// [`Self::ingest_file`] followed by the file's Phase-1 warm-up
    /// ([`Self::prepare_file_for_analysis`]): its direct class references are
    /// resolved and lazy-loaded *now*, at write time, instead of serially at
    /// the front of the next references / re-analysis read.
    ///
    /// The host edit-path entry point: mutation happens only when text
    /// changes; requests are pure reads.
    /// Lazy loads triggered by the warm-up go through plain
    /// [`Self::ingest_file`], so faulting in a dependency never cascades into
    /// preparing *its* dependencies — the load frontier stays one file wide.
    pub fn ingest_file_prepared(&mut self, file: Arc<str>, source: Arc<str>) {
        self.index.clear_transient_batch_replay();
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
    pub fn set_file_text(&mut self, file: Arc<str>, source: Arc<str>) {
        self.index.clear_transient_batch_replay();
        let (changed, was_registered, index_was_initialized) = {
            let db = &mut self.db.salsa;
            let index_was_initialized = db.workspace_symbol_index_singleton().is_some();
            let existing = db.lookup_source_file(file.as_ref());
            let changed = existing.is_none_or(|sf| sf.text(db).as_ref() != source.as_ref());
            db.upsert_source_file(file.clone(), source.clone());
            (changed, existing.is_some(), index_was_initialized)
        };
        self.index.clear_dependency_graph_cache();
        // Before the workspace index singleton exists, mirror-only writes
        // cannot be queued for `settle_workspace_index`. A newly registered
        // file can nevertheless make an unresolved name in any cached
        // analysis resolvable, so invalidate those results immediately. Once
        // the singleton exists, reconciliation coalesces this invalidation
        // across the pending batch.
        if changed {
            if let Some(cache) = self.cache.as_deref() {
                let invalidate_path = was_registered || !cache.matches_content(&file, &source);
                if invalidate_path {
                    cache.evict_files_and_dependents(&[file.to_string()]);
                }
                if !index_was_initialized {
                    cache.evict_unresolved();
                }
            }
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
    pub fn set_vendor_files<I>(&mut self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        self.index.clear_transient_batch_replay();
        self.index.clear_dependency_graph_cache();
        // One revision bump for the batch, not one per registered file.
        let mut session = self.defer_revision_bumps();
        let db = &mut session.db.salsa;
        let index_was_initialized = db.workspace_symbol_index_singleton().is_some();
        let mut changed_inputs = Vec::new();
        for (file, source) in files {
            let existing = db.lookup_source_file(file.as_ref());
            if existing.is_none_or(|sf| sf.text(db).as_ref() != source.as_ref()) {
                changed_inputs.push((file.clone(), source.clone(), existing.is_some()));
            }
            db.upsert_source_file_with_durability(file, source, salsa::Durability::HIGH);
        }
        if let Some(cache) = session.cache.as_deref() {
            let paths: Vec<String> = changed_inputs
                .iter()
                .filter(|(file, source, existed)| *existed || !cache.matches_content(file, source))
                .map(|(file, _, _)| file.to_string())
                .collect();
            cache.evict_files_and_dependents(&paths);
            if !index_was_initialized && !changed_inputs.is_empty() {
                cache.evict_unresolved();
            }
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
    pub fn rebuild_workspace_symbol_index(&mut self) {
        self.db.salsa.rebuild_workspace_symbol_index();
    }

    /// Bulk variant of [`Self::set_file_text`], with one revision bump for the
    /// whole batch.
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
    pub fn set_workspace_files<I>(&mut self, files: I)
    where
        I: IntoIterator<Item = (Arc<str>, Arc<str>)>,
    {
        self.index.clear_dependency_graph_cache();
        // One revision bump for the batch, not one per registered file.
        let mut session = self.defer_revision_bumps();
        let index_was_initialized = session.workspace_symbol_index_ready();
        let (registered_paths, changed_inputs): (Vec<Arc<str>>, Vec<ChangedInput>) = {
            let db = &mut session.db.salsa;
            let mut registered = Vec::new();
            let mut changed = Vec::new();
            for (file, source) in files {
                let existing = db.lookup_source_file(file.as_ref());
                if existing.is_none_or(|sf| sf.text(db).as_ref() != source.as_ref()) {
                    changed.push((file.clone(), source.clone(), existing.is_some()));
                }
                db.upsert_source_file(file.clone(), source);
                registered.push(file);
            }
            (registered, changed)
        };
        if let Some(cache) = session.cache.as_deref() {
            let paths: Vec<String> = changed_inputs
                .iter()
                .filter(|(file, source, existed)| *existed || !cache.matches_content(file, source))
                .map(|(file, _, _)| file.to_string())
                .collect();
            cache.evict_files_and_dependents(&paths);
            if !index_was_initialized && !changed_inputs.is_empty() {
                cache.evict_unresolved();
            }
        }
        if !registered_paths.is_empty() && session.resolver.is_some() {
            session.evict_unresolvable_for_files(&registered_paths);
        }
    }

    /// The workspace generation epoch ("are we up to date" counter). Bumped whenever a file is added or removed. A consumer
    /// records this alongside the diagnostics it publishes for a file; when the
    /// value later advances (background indexing registered more files), those
    /// files become candidates for re-analysis + re-publish.
    pub fn index_generation(&self) -> u64 {
        self.db.salsa.workspace_revision_value()
    }

    /// The text-write epoch — salsa's own global revision, which bumps on
    /// every source-file add, edit, or removal (any input write at all),
    /// unlike [`Self::index_generation`] which only bumps on file
    /// add/remove/declaration changes (a body-only edit deliberately leaves
    /// it unchanged, to avoid over-invalidating workspace-enumeration
    /// queries). A cache of a per-file or per-query *result* — as opposed to
    /// declaration-shape state — must key on this instead: any text write
    /// can move or add/remove a reference location within that file, even
    /// when no declaration changed. See `MirDbStorage::current_revision`'s
    /// doc comment for why this must be salsa's own revision rather than a
    /// hand-rolled counter.
    pub fn text_revision(&self) -> salsa::Revision {
        self.db.salsa.current_revision()
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
    /// **Responsiveness:** parsing runs on a snapshot; only the cheap
    /// symbol-map merge writes salsa. Queries running on other threads'
    /// snapshots during that write may unwind with `salsa::Cancelled` and
    /// should be retried.
    pub fn index_batch(
        &mut self,
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
        self.index.clear_dependency_graph_cache();

        // 1. Register the chunk as HIGH-durability inputs, with one revision
        //    bump for the chunk rather than one per new file.
        let (sources, changed_inputs, had_index): IndexedSources = {
            let mut session = self.defer_revision_bumps();
            let db = &mut session.db.salsa;
            let had_index = db.workspace_symbol_index_singleton().is_some();
            let mut changed_inputs = Vec::new();
            let sources = files
                .iter()
                .map(|(file, source)| {
                    let existing = db.lookup_source_file(file.as_ref());
                    if existing.is_none_or(|sf| sf.text(db).as_ref() != source.as_ref()) {
                        changed_inputs.push((file.clone(), source.clone(), existing.is_some()));
                    }
                    db.upsert_source_file_with_durability(
                        file.clone(),
                        source.clone(),
                        salsa::Durability::HIGH,
                    )
                })
                .collect();
            (sources, changed_inputs, had_index)
        };
        let registered = sources.len();
        if let Some(cache) = self.cache.as_deref() {
            let paths: Vec<String> = changed_inputs
                .iter()
                .filter(|(file, source, existed)| *existed || !cache.matches_content(file, source))
                .map(|(file, _, _)| file.to_string())
                .collect();
            cache.evict_files_and_dependents(&paths);
            if !had_index && !changed_inputs.is_empty() {
                cache.evict_unresolved();
            }
        }

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
        let seed = self.db.salsa.workspace_symbol_index_singleton().is_none();
        let view = self.db_view();
        let snap = view.db();
        let to_collect: Vec<crate::db::SourceFile> = if seed {
            snap.all_source_files()
        } else {
            sources.clone()
        };

        // 2. Collect per-file declarations on a snapshot (this is where parsing
        //    happens); also primes the shared parse/disk caches.
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
                to_collect.iter().map(|&sf| collect_one(snap, sf)).collect()
            };
        drop(view);

        if cancel.is_cancelled() {
            return crate::IndexBatchOutcome {
                registered,
                cancelled: true,
                generation: self.index_generation(),
            };
        }

        // 3. Apply to the singleton under a SHORT write window — only cheap map
        //    construction / merge runs here (no parse).
        let (declarations_changed, displaced_owners) = {
            let db = &mut self.db.salsa;
            let declarations_changed = decls
                .iter()
                .any(|(sf, decls)| !db.file_declarations_match(*sf, decls));
            let old_index = if declarations_changed && self.cache.is_some() {
                db.workspace_symbol_index_singleton()
                    .map(|singleton| singleton.index(db).clone())
            } else {
                None
            };
            let candidates = old_index.as_ref().map(|_| decls.clone());
            if db.workspace_symbol_index_singleton().is_none() {
                db.build_workspace_index_from_decls(decls);
            } else {
                db.merge_precomputed_into_workspace_index(&decls);
            }
            let displaced_owners = old_index
                .zip(candidates)
                .map(|(old, candidates)| displaced_cache_owners(db, &old, &candidates))
                .unwrap_or_default();
            (declarations_changed, displaced_owners)
        };
        if let Some(cache) = self.cache.as_deref() {
            cache.evict_with_dependents(&displaced_owners);
            if declarations_changed {
                cache.evict_unresolved();
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
    pub fn finalize_index(&mut self) {
        self.db.salsa.rebuild_workspace_symbol_index();
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
    ///
    /// Returns every file whose replayed postings carry an unresolved name
    /// (`live_analyzed: false` in `RefCommit` terms) — these are seeded, but
    /// `indexed_references_to`'s freshness pass can never treat them as
    /// immune to growth, so the *first* query that touches one of them pays
    /// a full `analyze_file` synchronously on the request path (measured:
    /// ~1.3-1.5s on a distinctive static method in a 15K-file workspace).
    /// The subset is known right here, at warm-start time — a caller with
    /// idle time between warm-start and the first live query (e.g. an LSP
    /// server between `indexReady` and the user's first request) can close
    /// most of that gap by handing this list to
    /// [`Self::reanalyze_files_cancellable`] on a background thread, the
    /// same pattern [`Self::prefetch_imports`] uses for lazy-loaded
    /// vendor FQCNs:
    ///
    /// ```ignore
    /// let unresolved = session.warm_start_files(&files);
    /// let s = session.clone();
    /// std::thread::spawn(move || {
    ///     s.reanalyze_files_cancellable(&unresolved, &crate::IndexCancel::new());
    /// });
    /// ```
    ///
    /// This only relocates cost that would otherwise land on a user-facing
    /// query — it cannot eliminate the freshness check itself, since an
    /// unresolved posting is workspace-generation-sensitive by design.
    pub fn warm_start_files(&mut self, files: &[(Arc<str>, Arc<str>)]) -> Vec<Arc<str>> {
        let Some(cache) = self.cache.clone() else {
            return Vec::new();
        };
        let stub_cache = self.db.stub_cache.clone();
        let php_v = self.php_version.cache_byte();

        // Register the whole bundled stub set up front. A seeded symbol-index
        // singleton is only sound when no stub registration arrives after the
        // seed (`ensure_stubs_for_ast` upserts bypass index maintenance and
        // would leave the new stubs invisible); with everything registered
        // here, later lazy stub loads are no-ops. Same contract as
        // `index_batch`.
        self.ensure_all_stubs();

        {
            let db = &mut self.db.salsa;
            for (file, text) in files {
                db.upsert_source_file_with_durability(
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

        // Phase 1: read the disk-cache slices in parallel — the actual I/O
        // cost of a warm-start replay at scale (~0.8-0.9s of a 3.9s warm boot
        // at 15.4K files serially; this is the `index_batch` pattern already
        // used elsewhere in this file). Only cheap map construction/merge
        // runs in phase 2.
        let view = self.db_view();
        let hits: Vec<WarmStartHit> = {
            use rayon::prelude::*;
            files
                .par_iter()
                .map_with(view.db().clone(), |db, (file, _)| {
                    // Freshness is keyed on the Arc actually stored on the
                    // input — an upsert against already-registered,
                    // content-equal text keeps the prior Arc (see
                    // `ingest_file`), so read back what's really there rather
                    // than assume identity with the file's own `text`.
                    let sf = db.lookup_source_file(file.as_ref())?;
                    let stored_text = sf.text(db).clone();

                    let hex = crate::cache::hash_content(&stored_text);
                    let refs = cache.get(file, &hex).map(|(issues, ref_locs)| {
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
                        // Resolved from the cached issue set: a fully-resolved
                        // replay survives the registrations/lazy loads that
                        // follow warm-up instead of being invalidated by the
                        // first generation bump.
                        let resolved = !crate::db::issues_have_unresolved_names(&issues);
                        (locs, resolved)
                    });

                    let stub = stub_cache.as_ref().and_then(|stub_cache| {
                        let hash = crate::stub_cache::hash_source(&stored_text);
                        let (mut slice, _issues) = stub_cache.get(file, &hash, php_v)?;
                        crate::stub_cache::prepare_for_ingest(&mut slice);
                        let entries = crate::db::subtype_index::entries_from_slice(&slice);
                        let decls = crate::db::decls_from_slice(&slice, sf);
                        Some((entries, decls))
                    });

                    Some(WarmStartHit {
                        file: file.clone(),
                        sf,
                        stored_text,
                        refs,
                        stub,
                    })
                })
                .filter_map(|hit| hit)
                .collect()
        };
        drop(view);

        // Phase 2: apply — only cheap salsa input writes and map merges.
        let mut seed_decls: Vec<(crate::db::SourceFile, crate::db::FileDeclarations)> =
            Vec::with_capacity(hits.len());
        let mut unresolved: Vec<Arc<str>> = Vec::new();
        let mut structural_target_files: Vec<Arc<str>> = Vec::new();
        let mut dependency_graph_changed = false;
        {
            let db = &self.db.salsa;
            for hit in hits {
                let WarmStartHit {
                    file,
                    sf,
                    stored_text,
                    refs,
                    stub,
                } = hit;
                if let Some((locs, resolved)) = refs {
                    let file_no = db.locked_ref_index().intern_path(&file);
                    db.set_file_reference_locations(file_no, locs);
                    self.index
                        .mark_ref_committed(&file, &stored_text, None, commit_gen, resolved);
                    dependency_graph_changed = true;
                    if !resolved {
                        unresolved.push(file.clone());
                    }
                }
                if let Some((entries, decls)) = stub {
                    let file_no = db.locked_ref_index().intern_path(&file);
                    db.set_file_class_edges(file_no, entries);
                    self.index.mark_defs_committed(&file, &stored_text);
                    structural_target_files.push(file.clone());
                    dependency_graph_changed = true;
                    seed_decls.push((sf, decls));
                }
            }
        }
        for file in &structural_target_files {
            let targets = file_outgoing_dependencies(&self.db.salsa, file.as_ref(), false);
            self.last_structural_targets
                .insert(file.as_ref().to_string(), targets);
        }
        if dependency_graph_changed {
            self.index.clear_dependency_graph_cache();
        }

        self.seed_workspace_index_from_warm_start(seed_decls);
        unresolved
    }

    /// Seed the workspace symbol index singleton from warm-start declaration
    /// projections, so a returning session's first query answers from an O(1)
    /// map instead of the tracked O(all-files) `workspace_symbol_index` walk
    /// (~4s at 15K files: one `collect_file_definitions` slice-deserialization
    /// per file, re-validated after every prepare-loop revision bump).
    ///
    /// `covered` holds decls projected from content-hash-valid disk slices.
    /// Files without a valid slice (changed since last session, plus any stub
    /// not yet slice-cached) are collected in parallel — real parses, so the
    /// seed is skipped entirely when the gap is large (a first-ever boot,
    /// where the parse bill belongs to the background sweep, not startup).
    fn seed_workspace_index_from_warm_start(
        &mut self,
        covered: Vec<(crate::db::SourceFile, crate::db::FileDeclarations)>,
    ) {
        use rustc_hash::FxHashSet;
        if covered.is_empty() {
            return;
        }

        let view = self.db_view();
        let snap = view.db();
        let all = snap.all_source_files();
        let covered_set: FxHashSet<crate::db::SourceFile> =
            covered.iter().map(|(sf, _)| *sf).collect();
        let missing: Vec<crate::db::SourceFile> = all
            .iter()
            .copied()
            .filter(|sf| !covered_set.contains(sf))
            .collect();

        // Gap ceiling: beyond this the "fill" is a workspace-scale parse and
        // seeding stops being a warm start. 1024 absorbs a large changed set
        // plus the bundled stubs; the quarter bound keeps tiny workspaces
        // seedable even when most files changed.
        let threshold = 1024usize.max(all.len() / 4);
        if missing.len() > threshold {
            return;
        }

        // Parse/collect the gap in parallel on one snapshot, as `index_batch`
        // does. `collect_file_declarations`
        // is disk-slice-accelerated itself, so "missing" here often means a
        // cheap deserialization rather than a parse.
        let gap_decls: Vec<(crate::db::SourceFile, crate::db::FileDeclarations)> = {
            use rayon::prelude::*;
            missing
                .par_iter()
                .map_with(snap.clone(), |db, &sf| {
                    (sf, crate::db::collect_file_declarations(db, sf).clone())
                })
                .collect()
        };
        drop(view);

        let mut decls = covered;
        decls.extend(gap_decls);

        let db = &mut self.db.salsa;
        if db.workspace_symbol_index_singleton().is_none() {
            db.build_workspace_index_from_decls(decls);
        } else {
            // Something (e.g. a vendor-eager `index_batch`) seeded first; its
            // singleton is already maintained incrementally. Merge only files
            // it hasn't seen — `merge_precomputed_into_workspace_index` skips
            // files already snapshotted.
            db.merge_precomputed_into_workspace_index(&decls);
        }
    }

    /// Reconcile the symbol-index singleton with mirror-only text writes.
    ///
    /// Plain `upsert_source_file_with_durability` calls (an LSP host
    /// mirroring watcher-driven external edits or new files) bypass
    /// `ingest_file`'s incremental index maintenance; those files accumulate
    /// in a pending set while a singleton exists. Query entry points call
    /// this first so the singleton is never consulted stale. Declaration
    /// memos are pre-warmed on a snapshot in parallel, so the per-file merge
    /// is a memo hit.
    pub fn settle_workspace_index(&mut self) {
        let _ = self.settle_workspace_index_cancellable(&|| false);
    }

    /// The single write prelude every interactive query needs before it can
    /// read a consistent snapshot: reconcile the workspace index, then (if a
    /// specific file is the target of the query) warm up that file's direct
    /// dependencies via [`Self::prepare_file_for_analysis`].
    ///
    /// Run it before handing out an [`super::AnalysisSnapshot`]: snapshot
    /// queries load missing classes on demand, but workspace-wide ones only
    /// enumerate what this has settled. Pass the file a query is about, or
    /// `None` for workspace-wide queries.
    pub fn prepare_for_query(&mut self, file: Option<&Arc<str>>) {
        self.settle_workspace_index();
        if let Some(file) = file {
            self.prepare_file_for_analysis(file);
        }
    }

    /// Cancellable form of [`Self::settle_workspace_index`]. Returns `false`
    /// when the caller's request was cancelled before the pending index work
    /// could be reconciled.
    pub(crate) fn settle_workspace_index_cancellable(
        &mut self,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> bool {
        self.db.salsa.adopt_on_demand_files();
        // Bounded: snapshot readers keep queueing on-demand loads meanwhile.
        let mut rounds_left = SETTLE_ROUNDS;
        loop {
            if rounds_left == 0 {
                return true;
            }
            rounds_left -= 1;
            // Every early return below drops the claim, which re-queues its
            // paths for the next settle.
            let claim = {
                if should_cancel() {
                    return false;
                }
                let view = self.db_view();
                let db = view.db();
                if db.index_pending_is_empty() {
                    return true;
                }
                if should_cancel() {
                    return false;
                }
                db.claim_index_pending()
            };
            if claim.paths().is_empty() {
                return true;
            }

            if should_cancel() {
                return false;
            }
            let decls: Vec<(crate::db::SourceFile, crate::db::FileDeclarations)> = {
                use rayon::prelude::*;
                let view = self.db_view();
                let snap = view.db();
                let sfs: Vec<crate::db::SourceFile> = claim
                    .paths()
                    .iter()
                    .filter_map(|p| snap.lookup_source_file(p.as_ref()))
                    .collect();
                sfs.par_iter()
                    .map_with(snap.clone(), |db, &sf| {
                        (sf, crate::db::collect_file_declarations(db, sf).clone())
                    })
                    .collect()
            };

            if should_cancel() {
                return false;
            }
            let db = &mut self.db.salsa;
            let declarations_changed = decls
                .iter()
                .any(|(sf, decls)| !db.file_declarations_match(*sf, decls));
            let old_index = if declarations_changed && self.cache.is_some() {
                db.workspace_symbol_index_singleton()
                    .map(|singleton| singleton.index(db).clone())
            } else {
                None
            };
            let candidates = old_index.as_ref().map(|_| decls.clone());
            // `update_workspace_index_for_file` clones the singleton maps per
            // call; for a bulk arrival (branch switch) one full rebuild — memo
            // validations plus a single map build, since the decls were just
            // pre-warmed above — beats N clones.
            if decls.len() > 32 {
                db.rebuild_workspace_symbol_index();
            } else {
                for (sf, decls) in decls {
                    if !db.update_workspace_index_for_file(sf, decls) {
                        db.rebuild_workspace_symbol_index();
                    }
                }
            }
            let displaced_owners = old_index
                .zip(candidates)
                .map(|(old, candidates)| displaced_cache_owners(db, &old, &candidates))
                .unwrap_or_default();

            // New declarations can satisfy negative lookups with no reverse
            // dependency edge. Body-only edits leave these entries intact.
            if let Some(cache) = self.cache.as_deref() {
                cache.evict_with_dependents(&displaced_owners);
                if declarations_changed {
                    cache.evict_unresolved();
                }
            }
            claim.commit();
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
    pub fn invalidate_file(&mut self, file: &str) {
        self.index.clear_transient_batch_replay();
        self.index.clear_dependency_graph_cache();
        self.index.retire_file(&self.db.salsa, file);
        self.db.salsa.remove_source_file(file);
        // Outgoing structural edges disappear from the derived graph
        // automatically: the file is no longer in `source_file_paths()`, so
        // `dependency_graph()` stops iterating it.
        // Clear stale symbol tracking for this file — it's fully gone.
        self.stale_defined_symbols.remove(file);
        self.last_ingested_symbols.remove(file);
        self.last_structural_targets.remove(file);
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
        self.db.source_file_count()
    }
}
