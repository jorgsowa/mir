use super::*;

impl AnalysisSession {
    /// Resolve a top-level symbol (class or function) to its declaration
    /// location. Powers go-to-definition.
    ///
    /// **Side effects:** if the symbol isn't yet known, this may invoke the
    /// configured [`crate::SourceProvider`] to fault in additional files and
    /// mutate the salsa input set. Use [`Self::definition_of_cached`] for a
    /// pure variant that only consults already-loaded state.
    ///
    /// Returns:
    /// - `Ok(Location)` — symbol found with a source location
    /// - `Err(NotFound)` — no such symbol in the codebase
    /// - `Err(NoSourceLocation)` — symbol exists but has no recorded span
    ///   (e.g. some stub-only declarations)
    pub fn definition_of(
        &self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        // Trigger any necessary lazy-load mutations before snapshotting.
        match symbol {
            crate::Name::Class(fqcn) => {
                let _ = self.load_class(fqcn.as_ref());
            }
            crate::Name::Function(fqn) => {
                let _ = self.load_class(fqn.as_ref());
            }
            crate::Name::Method { class, .. }
            | crate::Name::Property { class, .. }
            | crate::Name::ClassConstant { class, .. } => {
                let _ = self.load_class(class.as_ref());
            }
            _ => {}
        }
        self.definition_of_cached(symbol)
    }

    /// Pure variant of [`Self::definition_of`]. Never invokes the
    /// [`crate::SourceProvider`] and never mutates salsa inputs; resolves
    /// only against state already loaded by `set_file_text` / `ingest_file`.
    /// Returns `Err(NotFound)` when the symbol isn't in the loaded set, even
    /// if a resolver could in principle map it.
    pub fn definition_of_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        let db = self.snapshot_db();
        match symbol {
            crate::Name::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                class
                    .location()
                    .cloned()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                f.location
                    .clone()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Method { class, name }
            | crate::Name::Property { class, name }
            | crate::Name::ClassConstant { class, name } => {
                crate::db::member_location(&db, class, name)
                    .ok_or(crate::SymbolLookupError::NotFound)
            }
            crate::Name::GlobalConstant(_) => Err(crate::SymbolLookupError::NoSourceLocation),
        }
    }

    /// Hover information for a symbol: type, docstring, and definition location.
    ///
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor
    /// position, then build a [`crate::Name`] from its `kind`. This method
    /// assembles the displayable hover data.
    ///
    /// **Side effects:** when `symbol`'s owning class isn't yet loaded, this
    /// may invoke the configured [`crate::SourceProvider`] to fault in
    /// dependencies. Use [`Self::hover_cached`] for a pure variant.
    ///
    /// Returns `Err(NotFound)` if the symbol doesn't exist. May still return
    /// `Ok` with `docstring: None` or `definition: None` if those specific
    /// pieces aren't available.
    pub fn hover(
        &self,
        symbol: &crate::Name,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        // Trigger lazy loading for class-rooted symbols before snapshotting.
        // No-op when the class is already known; ensures inherited member
        // lookups have the chain present.
        match symbol {
            crate::Name::Class(fqcn) => {
                self.load_class(fqcn.as_ref());
            }
            crate::Name::Method { class, .. }
            | crate::Name::Property { class, .. }
            | crate::Name::ClassConstant { class, .. } => {
                // Fault in the owning class for navigation if the background
                // indexer hasn't reached it yet. Its inheritance ancestors
                // resolve through the (eagerly-built) workspace symbol index.
                self.load_class(class.as_ref());
            }
            _ => {}
        }
        self.hover_cached(symbol)
    }

    /// Pure variant of [`Self::hover`]. Never invokes the
    /// [`crate::SourceProvider`]; consults only the already-loaded db.
    pub fn hover_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Type};
        let db = self.snapshot_db();
        match symbol {
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let f = crate::db::find_function(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = f
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Type::mixed);
                let docstring = f.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: f.location.clone(),
                })
            }
            crate::Name::Method { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, m) = crate::db::find_method_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = m
                    .return_type
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(Type::mixed);
                let docstring = m.docstring.as_ref().map(|s| s.to_string());
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition: m.location.clone(),
                })
            }
            crate::Name::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(&db, fqcn.as_ref());
                let class = crate::db::find_class_like(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = Type::single(Atomic::TNamedObject {
                    fqcn: mir_types::Name::from(fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                });
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: class.location().cloned(),
                })
            }
            crate::Name::Property { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, p) = crate::db::find_property_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: p.location.clone(),
                })
            }
            crate::Name::ClassConstant { class, name } => {
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let (_, c) = crate::db::find_class_constant_in_chain(&db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: c.ty.clone(),
                    docstring: None,
                    definition: c.location.clone(),
                })
            }
            crate::Name::GlobalConstant(fqn) => {
                let here = crate::db::Fqcn::from_str(&db, fqn.as_ref());
                let ty = crate::db::find_global_constant(&db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: (*ty).clone(),
                    docstring: None,
                    definition: None,
                })
            }
        }
    }

    /// Raw reference locations indexed by string symbol key, kept for tests
    /// that use the legacy stringly-typed API. Prefer [`Self::indexed_references_to`]
    /// with a typed [`crate::Name`].
    #[doc(hidden)]
    pub fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        use crate::db::MirDatabase;
        let db = self.snapshot_db();
        db.reference_locations(symbol)
    }

    /// Files declaring transitive subclasses of `class_fqn`, backed by the
    /// maintained subtype index (see [`Self::indexed_subtype_classes`]).
    /// Excludes `class_fqn`'s own declaring file — the caller adds it.
    ///
    /// Lets a reference-search caller scope a `protected` member to its class
    /// hierarchy without reconstructing that hierarchy from declaration text:
    /// subclasses are matched by resolved FQCN, so `extends \Ns\Base` and
    /// aliased `use` forms are all found. Read-only from the caller's
    /// perspective; may trigger an on-demand commit of stale/uncommitted
    /// candidates' class edges (same self-heal `indexed_subtype_classes` uses).
    pub fn subtype_files(&self, class_fqn: &str) -> Vec<Arc<str>> {
        self.settle_workspace_index();
        let files = self.snapshot_db().source_file_paths();
        let mut out: Vec<Arc<str>> = self
            .indexed_subtype_classes(class_fqn, &files, false)
            .into_iter()
            .map(|s| s.file)
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// `use`-import occurrences of `symbol` — the import statement's own name
    /// token (`use Foo\Bar;`, `use function ...;`, `use const ...;`), not a
    /// usage site. Recorded under a `use:`-prefixed posting distinct from the
    /// plain `cls:`/`fn:`/`gcnst:` keys [`Self::indexed_references_to`] reads,
    /// so a symbol rename can also find/update the import line without a
    /// plain find-references query suddenly including import statements.
    ///
    /// Read-only posting-list lookup, filtered to `files` — no freshness pass:
    /// callers that need guaranteed-fresh results for an uncommitted file
    /// should analyze it first (e.g. via [`Self::indexed_references_to`] on
    /// the same file set).
    ///
    /// Returned ranges use mir's native coordinates: 1-based lines and
    /// 0-based Unicode code-point columns (UTF-32/LSP `positionEncoding`
    /// `"utf-32"`), not UTF-8 byte offsets or UTF-16 code units.
    pub fn indexed_use_import_locations(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, crate::Range)> {
        self.settle_workspace_index();
        let key = format!("use:{}", symbol.codebase_key());
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let guard = self.db.salsa.read();
        let mut out: Vec<(Arc<str>, crate::Range)> = guard
            .reference_locations(&key)
            .into_iter()
            .filter(|(file, ..)| scope.contains(file.as_ref()))
            .map(|(file, line, col_start, col_end)| {
                (file, span_range(line, col_start as u32, col_end as u32))
            })
            .collect();
        out.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.start.line.cmp(&b.1.start.line))
                .then(a.1.start.column.cmp(&b.1.start.column))
        });
        out.dedup();
        out
    }

    /// Inverted-index find-references: posting-list lookup plus an on-demand
    /// freshness/completeness pass over `files` (the host's candidate scope
    /// — passing the whole workspace is fine; see the gate below).
    ///
    /// A candidate whose postings were committed from its current input text
    /// (Arc identity) is answered from the index with no salsa work at all.
    /// Stale or never-committed candidates are analyzed via the memoized
    /// `analyze_file` query and committed, so each file pays that cost once
    /// per text change — after a background warm sweep the steady state is a
    /// pure lookup, O(results) instead of O(candidates). Never-committed
    /// candidates are additionally gated on their raw text mentioning the
    /// symbol's name (whole-identifier, ASCII-case-insensitive), so hosts
    /// need no text prefilter of their own — and must not use one, since a
    /// host-side filter cannot know these matching semantics.
    ///
    /// Results are filtered to `files` (the host controls scope — e.g.
    /// workspace files only, excluding stubs/vendor). With
    /// `include_declaration`, the symbol's declaration name span is appended
    /// when it lies inside the scope.
    ///
    /// Returned ranges use mir's native coordinates: 1-based lines and
    /// 0-based Unicode code-point columns (UTF-32/LSP `positionEncoding`
    /// `"utf-32"`), not UTF-8 byte offsets or UTF-16 code units.
    ///
    /// `should_cancel` is polled at phase boundaries and between
    /// cancellation retries; `true` aborts with `None`.
    ///
    /// Memoized per `(symbol, files, include_declaration, text_revision)` —
    /// see [`RefQueryCacheKey`]. The freshness scan below still costs
    /// O(candidates) even when every candidate is already committed (it has
    /// to check), so a caller re-running the same query against unchanged
    /// state (e.g. a host recomputing reference counts for a code-lens
    /// refresh) would otherwise re-pay that scan on every call; this makes
    /// the repeat a single hashmap lookup instead.
    pub fn indexed_references_to(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Option<Vec<(Arc<str>, crate::Range)>> {
        // No `should_cancel()` check here: a cache hit does no analysis
        // work, so there's nothing to cancel, and calling it would consume
        // one of the caller's cancellation-probe invocations before the
        // uncached path's own checks ever run (some callers, e.g.
        // `session_sweep_persists_postings_for_next_launch`, count these to
        // prove a warm query needed no re-analysis).
        let cache_key = RefQueryCacheKey {
            symbol: symbol.codebase_key(),
            include_declaration,
            generation: self.query_cache_generation(),
            files_hash: hash_files(files),
        };
        if let Some(cached) = self.ref_query_cache.read().get(&cache_key) {
            self.ref_query_cache_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some((**cached).clone());
        }
        let result =
            self.indexed_references_to_uncached(symbol, files, include_declaration, should_cancel)?;
        if self.query_cache_generation() != cache_key.generation {
            // The generation moved while computing (an edit, or a defs
            // commit growing the subtype index — possibly this query's own).
            // The key can never be looked up again, so don't cache it; the
            // next identical query recomputes once against settled state
            // and caches then.
            return Some(result);
        }
        let mut cache = self.ref_query_cache.write();
        if !cache.advance_to(cache_key.generation, || {
            self.ref_query_cache_locations
                .store(0, std::sync::atomic::Ordering::Relaxed);
        }) {
            return Some(result);
        }
        let new_len = result.len();
        let prior = self
            .ref_query_cache_locations
            .fetch_add(new_len, std::sync::atomic::Ordering::Relaxed);
        if prior + new_len > REF_QUERY_CACHE_LOCATION_CAP {
            cache.map.clear();
            self.ref_query_cache_locations
                .store(new_len, std::sync::atomic::Ordering::Relaxed);
        }
        cache.map.insert(cache_key, Arc::new(result.clone()));
        Some(result)
    }

    /// Uncached implementation of [`Self::indexed_references_to`]. Callers
    /// should use the memoizing wrapper; this is split out only so the cache
    /// check/populate logic doesn't have to interleave with the retry loops
    /// below.
    fn indexed_references_to_uncached(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        should_cancel: &(dyn Fn() -> bool + Sync),
    ) -> Option<Vec<(Arc<str>, crate::Range)>> {
        self.settle_workspace_index();
        use std::panic::AssertUnwindSafe;

        use rayon::prelude::*;

        let key = symbol.codebase_key();

        // Freshness pass: candidates whose postings are not exact for their
        // current text. Files not registered as `SourceFile` inputs are
        // skipped. Never-committed files — no commit mark, hence no postings
        // at all (every mark drop accompanies a posting clear) — are further
        // gated on their text mentioning the symbol's name: such a file can
        // neither hold stale postings nor produce new ones, so a cold query
        // on a common name skips the bulk of the workspace instead of
        // analyzing it. A LIVE-analyzed file stale only by generation
        // (unresolved-name commit, text unchanged since) gets the same gate:
        // the current text is exactly what this session's own analysis
        // already scanned, so a needle miss is just as conclusive as for a
        // never-committed file. Everything else — a genuinely edited file, or
        // a commit seeded by an unverified disk-cache replay
        // (`warm_start_files`, never scanned by this session) — re-analyzes
        // unconditionally: an edited file's old postings are for different
        // text, and a replayed commit's postings are only as trustworthy as
        // the cache entry, so neither can be cleared by a textual gate alone.
        // Same discipline as `commit_defs_for_matching` on the defs index.
        //
        let gate = self.reference_gate(symbol);
        // The whole gate — identifier needles and raw call tokens alike —
        // answers from the persistent mention index: every needle is admitted
        // (a declared class-like short name is already in the universe;
        // member/function names and the two `__construct` call tokens enter
        // verbatim), so a recorded mention set answers with lookups and only
        // a never-scanned or since-edited file pays one scan against the
        // whole universe, recorded for every later consumer (including the
        // subtype-BFS gate). A needle new to the universe epoch-invalidates
        // older recordings for itself only — the first query on it rescans
        // uncovered files once, the same cost the per-query scan paid every
        // time before.
        let has_needles = !gate.idents.is_empty() || !gate.raw.is_empty();
        let (mention_queries, mention_scanner) = if has_needles {
            let guard = self.db.salsa.read();
            guard.add_literal_mention_names(gate.idents.iter().map(|s| s.as_str()));
            guard.add_raw_mention_needles(gate.raw.iter().map(|s| s.as_str()));
            let queries: Vec<_> = gate
                .idents
                .iter()
                .chain(gate.raw.iter())
                .filter_map(|s| guard.prepare_class_mention_query(s))
                .collect();
            (queries, guard.class_mention_scanner())
        } else {
            (Vec::new(), None)
        };
        // Admission guarantees a query per needle and a non-empty universe;
        // anything else is defensive — the gate then admits every candidate
        // (analyze rather than skip, the conservative direction).
        let gate_complete = mention_queries.len() == gate.idents.len() + gate.raw.len()
            && mention_scanner.is_some();
        let committed_any: rustc_hash::FxHashSet<Arc<str>> =
            self.ref_committed_keys().into_iter().collect();
        type MentionScanRec = (Arc<str>, Arc<str>, Box<[mir_types::Name]>);
        let (stale, scanned): (Vec<Arc<str>>, Vec<MentionScanRec>) = loop {
            if should_cancel() {
                return None;
            }
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let current_gen = self.index_generation();
                let db_main = self.snapshot_db();
                files
                    .par_iter()
                    .map_with(db_main, |db, f| {
                        let Some(sf) = db.lookup_source_file(f.as_ref()) else {
                            return (None, None);
                        };
                        let text = sf.text(&*db as &dyn MirDatabase);
                        if self.is_ref_committed(f.as_ref(), text, current_gen) {
                            return (None, None);
                        }
                        if committed_any.contains(f.as_ref())
                            && !self.ref_commit_stale_by_generation_only(f.as_ref(), text)
                        {
                            return (Some(f.clone()), None);
                        }
                        if has_needles && gate_complete {
                            // Any needle answering `true` admits the file; an
                            // unanswerable one forces the single recorded
                            // scan, which settles every needle at once.
                            let mut answer = Some(false);
                            for q in &mention_queries {
                                match db.class_mention_answer(f.as_ref(), q, text) {
                                    Some(true) => {
                                        answer = Some(true);
                                        break;
                                    }
                                    Some(false) => {}
                                    None => answer = None,
                                }
                            }
                            match answer {
                                Some(true) => {}
                                Some(false) => return (None, None),
                                None => {
                                    let scanner = mention_scanner
                                        .as_ref()
                                        .expect("gate_complete implies a scanner");
                                    let names = scanner.scan(text);
                                    let hit = mention_queries
                                        .iter()
                                        .any(|q| names.binary_search(&q.name).is_ok());
                                    let rec = (f.clone(), text.clone(), names);
                                    return (hit.then(|| f.clone()), Some(rec));
                                }
                            }
                        }
                        (Some(f.clone()), None)
                    })
                    .collect::<Vec<_>>()
            }));
            match attempt {
                Ok(v) => {
                    let mut stale = Vec::new();
                    let mut scanned = Vec::new();
                    for (s, rec) in v {
                        if let Some(s) = s {
                            stale.push(s);
                        }
                        if let Some(rec) = rec {
                            scanned.push(rec);
                        }
                    }
                    break (stale, scanned);
                }
                Err(_) if should_cancel() => return None,
                Err(_) => {}
            }
        };

        // Record the fallback scans regardless of how the query proceeds:
        // each is a complete, current mention set for its file.
        if let Some(scanner) = &mention_scanner {
            if !scanned.is_empty() {
                let guard = self.db.salsa.read();
                for (file, text, names) in scanned {
                    guard.set_file_class_mentions(&file, &text, scanner.epoch(), names);
                }
            }
        }

        if !stale.is_empty() {
            // Phase 1 (serial, no live snapshot held): warm up stale
            // candidates. `prepare_file_for_analysis` mutates salsa inputs
            // (via `load_class`), so a concurrent writer — the background
            // warm sweep, or another request — can raise `salsa::Cancelled`
            // partway through a file. Catch and retry the SAME file here
            // rather than letting the panic escape: uncaught, it would force
            // the caller's outer retry loop (`indexed_references`) to
            // re-enter from scratch, redoing the freshness pass and
            // re-walking every already-warmed file in `stale` (cheap no-ops
            // via the `prepared_files` cache, but not free) before it even
            // gets back to the file that was interrupted. This doesn't
            // change how many times a write is ultimately attempted (the
            // outer loop already retries indefinitely on `Cancelled`); it
            // only narrows what a single cancellation discards from "the
            // whole query so far" to "the one file that was mid-flight".
            //
            // Tried and reverted TWICE: running this loop itself in parallel
            // (rayon; per-file and whole-batch retry variants, and again as
            // a `try_for_each` after the deferred-bump scope landed). Each
            // file's warm-up is individually safe under concurrent access
            // (every shared registry it touches — `prepared_files`,
            // `unresolvable_fqcns`, `pending_eager_function_files`, the
            // salsa db via `with_db_mut` — is lock-protected), but under the
            // `concurrent_reference_cancel` stress test (sustained
            // multi-thread writers + a background indexer, both hammering
            // the same db while several readers each run this phase
            // concurrently) every parallel variant deadlocks: CPU usage
            // drops to ~0 while wall time keeps climbing — OS threads parked
            // on a lock, most likely the fixed-size rayon pool saturated
            // with workers blocked on `with_db_mut`'s `RwLock` write lock
            // (an OS-level block, invisible to rayon's cooperative
            // scheduler) while the thread that would release it is itself
            // queued waiting for a free pool worker. Coalescing the
            // per-load revision bumps into one per pass (the deferred scope
            // below) did NOT fix it — the re-attempt hung the same way
            // (>590s for a ~5s test), so the cancellation storm was not the
            // trigger. Serial execution never contends for the pool this
            // way, so it stays the safe choice here even though it forgoes
            // the extra wall-clock parallelism a large stale set could
            // otherwise use.
            {
                // One revision bump for the whole warm-up loop instead of one
                // per lazily-loaded class: each bump is a salsa input write
                // that cancels every in-flight reader (a concurrent request's
                // Phase 2 pass restarts per bump). The scope closes before
                // Phase 2 reads `index_generation`, so commits below are
                // stamped with the post-load generation as before.
                let _deferred_bumps = self.defer_revision_bumps();
                for path in &stale {
                    loop {
                        if should_cancel() {
                            return None;
                        }
                        match salsa::Cancelled::catch(AssertUnwindSafe(|| {
                            self.prepare_file_for_analysis(path)
                        })) {
                            Ok(()) => break,
                            Err(_) if should_cancel() => return None,
                            Err(_) => {}
                        }
                    }
                }
            }

            // Phase 2 (parallel, pure) under a cancellation retry loop, then
            // a serial commit into both inverted indexes.
            let (commit_gen, analyzed) = loop {
                if should_cancel() {
                    return None;
                }
                // Generation before the snapshot: a file add racing the
                // analysis leaves these commits stale (self-healing on the
                // next query), never wrongly fresh.
                let gen = self.index_generation();
                let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                    // Freeze on the pass-scoped snapshot (borrow-only symbol
                    // lookups + pass-shared subtype cache): all lazy-loading
                    // finished in Phase 1, and a concurrent index write
                    // cancels this attempt, so the frozen view is never
                    // stale. Same discipline as the batch body pass.
                    let mut db_main = self.snapshot_db();
                    db_main.freeze_workspace_index();
                    stale
                        .iter()
                        .filter_map(|path| {
                            let sf = db_main.lookup_source_file(path.as_ref())?;
                            let text = sf.text(&db_main as &dyn MirDatabase).clone();
                            let out =
                                crate::db::analyze_file(&db_main as &dyn MirDatabase, sf).clone();
                            let defs = crate::db::collect_file_definitions(
                                &db_main as &dyn MirDatabase,
                                sf,
                            );
                            let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                            // Stage the disk-cache write only when the commit
                            // below will rewrite postings (see the sweep in
                            // `reanalyze_file_set` for the cost rationale).
                            let put = if self.ref_commit_is_current(path.as_ref(), &text, &out) {
                                None
                            } else {
                                self.stage_ref_cache_put(
                                    &db_main as &dyn MirDatabase,
                                    sf,
                                    path.as_ref(),
                                    &text,
                                    &out,
                                )
                            };
                            // Mention scan piggybacks on the analysis pass
                            // (pure; committed serially below), skipped when
                            // the file already holds a current scan.
                            let mentions = mention_scanner.as_ref().and_then(|s| {
                                (!db_main.class_mentions_current(path.as_ref(), &text, s.epoch()))
                                    .then(|| s.scan(&text))
                            });
                            Some((path.clone(), text, out, entries, put, mentions))
                        })
                        .collect::<Vec<_>>()
                }));
                match attempt {
                    Ok(v) => break (gen, v),
                    Err(_) if should_cancel() => return None,
                    Err(_) => {}
                }
            };
            let mut analyzed = analyzed;
            let guard = self.db.salsa.read();
            for (file, text, out, entries, put, mentions) in analyzed.iter_mut() {
                // Pointer-identical memo ⇒ identical postings: skip the
                // index rewrite and only re-stamp the freshness mark.
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

        // Posting lookup, filtered to the candidate scope.
        //
        // Member symbols resolve against the queried class plus its hierarchy
        // (mir records member refs under the *declaring* class, so a query on
        // an interface method must include implementor keys and vice versa).
        // Name-only fallback postings — receivers whose type couldn't be
        // resolved — are consulted only when the typed keys produce nothing,
        // mirroring the pre-index two-tier behavior: exact results when
        // resolution succeeds, by-name matches when nothing resolves.
        // `__construct` stays exact: `new Sub()` invokes `Sub::__construct`
        // even when only a parent declares one, so hierarchy fan-out would
        // wrongly return subtype instantiation sites for a parent query.
        let hierarchy: Vec<String> = match symbol {
            crate::Name::Method { class, name } => {
                if name.as_ref() == "__construct" || class.is_empty() {
                    if class.is_empty() {
                        Vec::new()
                    } else {
                        vec![class.trim_start_matches('\\').to_string()]
                    }
                } else {
                    self.member_hierarchy_classes(class.as_ref())
                }
            }
            crate::Name::Property { class, .. } | crate::Name::ClassConstant { class, .. } => {
                if class.is_empty() {
                    Vec::new()
                } else {
                    self.member_hierarchy_classes(class.as_ref())
                }
            }
            _ => Vec::new(),
        };
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let read_symbol_key = |symbol_key: &str| -> Vec<(Arc<str>, crate::Range)> {
            let guard = self.db.salsa.read();
            guard
                .reference_locations(symbol_key)
                .into_iter()
                .filter(|(file, ..)| scope.contains(file.as_ref()))
                .map(|(file, line, col_start, col_end)| {
                    (file, span_range(line, col_start as u32, col_end as u32))
                })
                .collect()
        };
        let mut scratch_key = String::new();
        let mut read_composed_key = |prefix: &str, middle: &str, separator: &str, suffix: &str| {
            scratch_key.clear();
            scratch_key.reserve(prefix.len() + middle.len() + separator.len() + suffix.len());
            scratch_key.push_str(prefix);
            scratch_key.push_str(middle);
            scratch_key.push_str(separator);
            scratch_key.push_str(suffix);
            read_symbol_key(&scratch_key)
        };
        let mut out: Vec<(Arc<str>, crate::Range)> = Vec::new();
        match symbol {
            crate::Name::Method { name, .. } => {
                for class in &hierarchy {
                    out.extend(read_composed_key(
                        "meth:",
                        class.as_ref(),
                        "::",
                        name.as_ref(),
                    ));
                }
            }
            crate::Name::Property { name, .. } => {
                for class in &hierarchy {
                    out.extend(read_composed_key(
                        "prop:",
                        class.as_ref(),
                        "::",
                        name.as_ref(),
                    ));
                }
            }
            crate::Name::ClassConstant { name, .. } => {
                for class in &hierarchy {
                    out.extend(read_composed_key(
                        "cnst:",
                        class.as_ref(),
                        "::",
                        name.as_ref(),
                    ));
                }
            }
            _ => out.extend(read_symbol_key(key.as_str())),
        }
        if out.is_empty() {
            match symbol {
                crate::Name::Method { name, .. } => {
                    out = read_composed_key("methname:", name.as_ref(), "", "");
                }
                crate::Name::Property { name, .. } => {
                    out = read_composed_key("propname:", name.as_ref(), "", "");
                }
                _ => {}
            }
        }
        out.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.start.line.cmp(&b.1.start.line))
                .then(a.1.start.column.cmp(&b.1.start.column))
        });
        out.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

        if include_declaration {
            // Declaration lookup runs salsa queries (and may lazy-load); a
            // concurrent write cancels it — declarations are then simply
            // omitted rather than failing the whole request.
            let decls: Vec<(Arc<str>, crate::Range)> = match symbol {
                crate::Name::Method { class, .. }
                | crate::Name::Property { class, .. }
                | crate::Name::ClassConstant { class, .. } => {
                    if class.is_empty() {
                        // Unknown owner: declarations by name, recorded as
                        // `methdecl:`/`propdecl:`/`cnstdecl:` postings during
                        // class/trait/interface/enum analysis.
                        match symbol {
                            crate::Name::Method { name, .. } => {
                                read_composed_key("methdecl:", name.as_ref(), "", "")
                            }
                            crate::Name::Property { name, .. } => {
                                read_composed_key("propdecl:", name.as_ref(), "", "")
                            }
                            crate::Name::ClassConstant { name, .. } => {
                                read_composed_key("cnstdecl:", name.as_ref(), "", "")
                            }
                            _ => Vec::new(),
                        }
                    } else {
                        salsa::Cancelled::catch(AssertUnwindSafe(|| {
                            self.member_decl_sites(&hierarchy, symbol)
                        }))
                        .unwrap_or_default()
                    }
                }
                _ => salsa::Cancelled::catch(AssertUnwindSafe(|| {
                    self.declaration_name_range(symbol).into_iter().collect()
                }))
                .unwrap_or_default(),
            };
            for (file, range) in decls {
                if scope.contains(file.as_ref())
                    && !out.iter().any(|(f, r)| *f == file && *r == range)
                {
                    out.push((file, range));
                }
            }
        }
        Some(out)
    }

    /// The queried class plus every class its members' references could be
    /// keyed under: resolved ancestors (a call on a subtype instance records
    /// the declaring ancestor) and transitive subtypes including trait users
    /// (a call on a subtype that overrides records the subtype). Display-form
    /// FQCNs, deduplicated case-insensitively.
    fn member_hierarchy_classes(&self, class_fqn: &str) -> Vec<String> {
        use std::panic::AssertUnwindSafe;
        let target = class_fqn.trim_start_matches('\\').to_string();
        let mut out: Vec<String> = vec![target.clone()];
        let ancestors = salsa::Cancelled::catch(AssertUnwindSafe(|| {
            let db = self.snapshot_db();
            let here = crate::db::Fqcn::from_str(&db, &target);
            crate::db::class_ancestors_by_fqcn(&db, here)
                .iter()
                .skip(1)
                .map(|a| a.trim_start_matches('\\').to_string())
                .collect::<Vec<_>>()
        }))
        .unwrap_or_default();
        out.extend(ancestors);
        let subs = {
            let guard = self.db.salsa.read();
            guard.subtype_sites_of(&target, true)
        };
        out.extend(
            subs.into_iter()
                .map(|s| s.fqcn.trim_start_matches('\\').to_string()),
        );
        let mut seen: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        out.retain(|c| seen.insert(c.to_ascii_lowercase()));
        out
    }

    /// Own-member declaration sites for `symbol` across `classes`: each class
    /// that itself declares the member (not inherited) contributes its name
    /// token. Kind-specific lookups — a class often declares a property and a
    /// method with the same short name, and `member_location` can't tell them
    /// apart.
    fn member_decl_sites(
        &self,
        classes: &[String],
        symbol: &crate::Name,
    ) -> Vec<(Arc<str>, crate::Range)> {
        let mut out: Vec<(Arc<str>, crate::Range)> = Vec::new();
        let db = self.snapshot_db();
        for class in classes {
            let here = crate::db::Fqcn::from_str(&db, class);
            let (loc, needle) = match symbol {
                crate::Name::Method { name, .. } => {
                    let Some(m) = crate::db::find_method_in_class(&db, here, name) else {
                        continue;
                    };
                    (m.location.clone(), name.to_string())
                }
                crate::Name::Property { name, .. } => {
                    let Some(p) = crate::db::find_property_in_class(&db, here, name) else {
                        continue;
                    };
                    (p.location.clone(), name.to_string())
                }
                crate::Name::ClassConstant { name, .. } => {
                    let Some(c) = crate::db::find_class_constant_in_class(&db, here, name) else {
                        continue;
                    };
                    (c.location.clone(), name.to_string())
                }
                _ => continue,
            };
            let Some(loc) = loc else { continue };
            let range = self.refine_location_to_name(&loc, &needle);
            out.push((loc.file.clone(), range));
        }
        out
    }

    /// The symbol's declaration site, narrowed from the collector's
    /// whole-declaration span to the declared name's own token (matching the
    /// span shape of recorded references).
    pub fn declaration_name_range(&self, symbol: &crate::Name) -> Option<(Arc<str>, crate::Range)> {
        if let crate::Name::GlobalConstant(fqn) = symbol {
            return self.global_constant_decl_range(fqn);
        }
        let loc = self.definition_of(symbol).ok()?;
        let short = match symbol {
            crate::Name::Class(f) | crate::Name::Function(f) | crate::Name::GlobalConstant(f) => {
                crate::db::subtype_index::short_name_of(f)
            }
            crate::Name::Method { name, .. }
            | crate::Name::Property { name, .. }
            | crate::Name::ClassConstant { name, .. } => name.as_ref(),
        };
        // Property declarations carry a `$` sigil in source, but reference
        // ranges cover the bare name; the word-boundary search below lands on
        // the name right after the sigil.
        let file = loc.file.clone();
        let range = self.refine_location_to_name(&loc, short);
        Some((file, range))
    }

    /// Narrow a whole-declaration [`mir_types::Location`] to the first
    /// word-boundary occurrence of `needle` inside its line span. Falls back
    /// to the location's own coordinates when the text is unavailable or the
    /// name doesn't appear (e.g. stub-only declarations).
    fn refine_location_to_name(&self, loc: &mir_types::Location, needle: &str) -> crate::Range {
        let fallback = span_range(loc.line, loc.col_start as u32, loc.col_end as u32);
        let text = {
            let db = self.snapshot_db();
            db.lookup_source_file(loc.file.as_ref())
                .map(|sf| sf.text(&db as &dyn MirDatabase).clone())
        };
        let Some(text) = text else {
            return fallback;
        };
        let needle_chars = needle.chars().count() as u32;
        let first_line = loc.line.saturating_sub(1) as usize;
        // Exact-case first: PHP property/constant names are case-sensitive
        // and an early case-insensitive hit can land on an unrelated token
        // (a type hint sharing the name). Case-insensitive second, for
        // method/class needles that arrive lowercase-normalized.
        for case_insensitive in [false, true] {
            for (idx, line_text) in text.lines().enumerate().skip(first_line) {
                let line_no = idx as u32 + 1;
                if line_no > loc.line_end {
                    break;
                }
                let min_col = if line_no == loc.line {
                    loc.col_start as usize
                } else {
                    0
                };
                if let Some(col) = identifier_char_col(line_text, needle, min_col, case_insensitive)
                {
                    return span_range(line_no, col, col + needle_chars);
                }
            }
        }
        fallback
    }

    /// Transitive subtypes of `class_fqn` (classes/interfaces/enums whose
    /// resolved ancestor chain reaches it), answered from the maintained
    /// subtype edge index.
    ///
    /// `files` is the host's candidate scope for the on-demand completeness
    /// pass: per BFS round, not-yet-committed files whose text mentions a
    /// frontier name get their definitions committed, so results are complete
    /// even before a background sweep has covered the workspace. Committed
    /// files answer from the index with no parsing at all.
    ///
    /// `include_trait_users` also counts `use Trait;` composition as a
    /// subtype edge (visibility-scoping semantics); leave it off for
    /// goto-implementation semantics (extends/implements only).
    ///
    /// Memoized per `(class_fqn, include_trait_users, files, text_revision)`
    /// — same shape and rationale as [`Self::indexed_references_to`]'s
    /// cache: `commit_defs_for_matching`'s freshness pass costs O(candidates)
    /// on every call regardless of outcome, so a repeat query (e.g. a host
    /// resolving a protected/static method's reference scope on every
    /// code-lens refresh) would otherwise re-pay it every time.
    pub fn indexed_subtype_classes(
        &self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Vec<SubtypeClassSite> {
        let cache_key = SubtypeQueryCacheKey {
            class_fqn: class_fqn.trim_start_matches('\\').to_ascii_lowercase(),
            include_trait_users,
            generation: self.query_cache_generation(),
            files_hash: hash_files(files),
        };
        if let Some(cached) = self.subtype_query_cache.read().get(&cache_key) {
            self.subtype_query_cache_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return (**cached).clone();
        }
        let result = self.indexed_subtype_classes_uncached(class_fqn, files, include_trait_users);
        if self.query_cache_generation() != cache_key.generation {
            // The generation moved while computing (an edit, or a defs
            // commit growing the subtype index — possibly this query's own).
            // The key can never be looked up again, so don't cache it; the
            // next identical query recomputes once against settled state
            // and caches then.
            return result;
        }
        let mut cache = self.subtype_query_cache.write();
        if !cache.advance_to(cache_key.generation, || {
            self.subtype_query_cache_sites
                .store(0, std::sync::atomic::Ordering::Relaxed);
        }) {
            return result;
        }
        let new_len = result.len();
        let prior = self
            .subtype_query_cache_sites
            .fetch_add(new_len, std::sync::atomic::Ordering::Relaxed);
        if prior + new_len > SUBTYPE_QUERY_CACHE_SITE_CAP {
            cache.map.clear();
            self.subtype_query_cache_sites
                .store(new_len, std::sync::atomic::Ordering::Relaxed);
        }
        cache.map.insert(cache_key, Arc::new(result.clone()));
        result
    }

    /// Uncached implementation of [`Self::indexed_subtype_classes`]. Callers
    /// should use the memoizing wrapper.
    fn indexed_subtype_classes_uncached(
        &self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Vec<SubtypeClassSite> {
        self.settle_workspace_index();
        let mut scanned: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        let mut pending: Vec<String> = vec![class_fqn.trim_start_matches('\\').to_string()];
        let mut sites: Vec<crate::db::SubtypeSite> = Vec::new();
        while !pending.is_empty() {
            let needles: Vec<String> = pending
                .drain(..)
                .filter(|f| scanned.insert(f.clone()))
                .map(|f| crate::db::subtype_index::short_name_of(&f).to_string())
                .collect();
            if !needles.is_empty() {
                self.commit_defs_for_matching(files, &needles);
            }
            sites = {
                let guard = self.db.salsa.read();
                guard.subtype_sites_of_lenient(class_fqn, include_trait_users)
            };
            pending = sites
                .iter()
                .map(|s| s.fqcn.trim_start_matches('\\').to_string())
                .filter(|f| !scanned.contains(f))
                .collect();
        }
        let mut out: Vec<SubtypeClassSite> = sites
            .into_iter()
            .filter_map(|s| {
                let loc = s.location.as_ref()?;
                let short = crate::db::subtype_index::short_name_of(&s.fqcn).to_string();
                let range = self.refine_location_to_name(loc, &short);
                Some(SubtypeClassSite {
                    fqcn: s.fqcn,
                    kind: s.kind,
                    is_abstract: s.is_abstract,
                    file: s.file,
                    range,
                })
            })
            .collect();
        // Anonymous classes never reach the definition collector; their
        // `new class implements X {}` sites are recorded as `impl:` postings
        // during body analysis (exact FQCN key plus a short-name key for the
        // same written-form leniency named classes get above).
        let root_lc = class_fqn.trim_start_matches('\\').to_ascii_lowercase();
        let short_lc = crate::db::subtype_index::short_name_of(&root_lc).to_string();
        let scope: rustc_hash::FxHashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let anon: Vec<(Arc<str>, u32, u16, u16)> = {
            let guard = self.db.salsa.read();
            let mut key = String::with_capacity("implshort:".len() + root_lc.len());
            key.push_str("impl:");
            key.push_str(&root_lc);
            let mut v = guard.reference_locations(&key);
            key.clear();
            key.push_str("implshort:");
            key.push_str(&short_lc);
            v.extend(guard.reference_locations(&key));
            v.sort();
            v.dedup();
            v
        };
        for (file, line, cs, ce) in anon {
            if !scope.contains(file.as_ref()) {
                continue;
            }
            let range = span_range(line, cs as u32, ce as u32);
            if out.iter().any(|s| s.file == file && s.range == range) {
                continue;
            }
            out.push(SubtypeClassSite {
                fqcn: Arc::from("class@anonymous"),
                kind: crate::db::ClassLikeKind::Class,
                is_abstract: false,
                file,
                range,
            });
        }
        out
    }

    /// Concrete implementations of `class_fqn::method` across its transitive
    /// subtypes: the same-named non-abstract method available to each subtype
    /// (its own declaration, or one inherited/composed from a parent, trait,
    /// or mixin), as `(subtype fqcn, file, name range)`. Subtypes resolving to
    /// the same declaring location collapse to a single entry.
    pub fn indexed_method_implementations(
        &self,
        class_fqn: &str,
        method: &str,
        files: &[Arc<str>],
    ) -> Vec<(Arc<str>, Arc<str>, crate::Range)> {
        use std::panic::AssertUnwindSafe;
        let subs = self.indexed_subtype_classes(class_fqn, files, false);
        if subs.is_empty() {
            return Vec::new();
        }
        loop {
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let db = self.snapshot_db();
                let mut out: Vec<(Arc<str>, Arc<str>, crate::Range)> = Vec::new();
                for sub in &subs {
                    let here = crate::db::Fqcn::from_str(&db, sub.fqcn.as_ref());
                    let Some((_, m)) = crate::db::find_method_in_chain(&db, here, method) else {
                        continue;
                    };
                    if m.is_abstract {
                        continue;
                    }
                    let Some(loc) = m.location.as_ref() else {
                        continue;
                    };
                    let range = self.refine_location_to_name(loc, method);
                    out.push((sub.fqcn.clone(), loc.file.clone(), range));
                }
                out
            }));
            if let Ok(mut out) = attempt {
                out.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.start.line.cmp(&b.2.start.line)));
                out.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);
                return out;
            }
        }
    }

    /// Commit definitions (class edges + freshness) for every file in `files`
    /// that is stale (committed against older text) or that has never been
    /// committed and mentions one of `shorts` as a whole identifier.
    ///
    /// The textual gate answers from the shared per-file mention cache — the
    /// same one `indexed_references_to`'s gate populates — so a file scanned
    /// by either consumer answers the other with a set lookup instead of an
    /// O(text) rescan per BFS round. A file the cache can't answer for is
    /// scanned once against the whole name universe and recorded.
    fn commit_defs_for_matching(&self, files: &[Arc<str>], shorts: &[String]) {
        use std::panic::AssertUnwindSafe;

        use rayon::prelude::*;

        let committed_any: rustc_hash::FxHashSet<Arc<str>> = {
            let guard = self.defs_committed_keys();
            guard.into_iter().collect()
        };
        // Admit the frontier names before preparing, so every needle gets a
        // real query (a declared class's short name is already in the
        // universe from indexing — admission then changes nothing).
        let (queries, mention_scanner) = {
            let guard = self.db.salsa.read();
            guard.add_literal_mention_names(shorts.iter().map(|s| s.as_str()));
            let queries: Vec<_> = shorts
                .iter()
                .filter_map(|s| guard.prepare_class_mention_query(s))
                .collect();
            (queries, guard.class_mention_scanner())
        };
        // Admission guarantees a query per needle and a non-empty universe;
        // anything else is defensive — the gate then admits every candidate
        // (recommit rather than skip, the conservative direction).
        let use_mentions = queries.len() == shorts.len() && mention_scanner.is_some();
        type Work = (Arc<str>, Arc<str>, Vec<crate::db::SubtypeEntry>);
        type MentionScanRec = (Arc<str>, Arc<str>, Box<[mir_types::Name]>);
        let (work, scanned): (Vec<Work>, Vec<MentionScanRec>) = loop {
            let attempt = salsa::Cancelled::catch(AssertUnwindSafe(|| {
                let db_main = self.snapshot_db();
                files
                    .par_iter()
                    .map_with(db_main, |db, path| {
                        let Some(sf) = db.lookup_source_file(path.as_ref()) else {
                            return (None, None);
                        };
                        let text = sf.text(&*db as &dyn MirDatabase).clone();
                        if self.is_defs_committed(path.as_ref(), &text) {
                            return (None, None);
                        }
                        // Never-committed files must mention a frontier name;
                        // stale (previously committed) files recommit
                        // unconditionally — their classes may have re-parented.
                        let mut scan_rec: Option<MentionScanRec> = None;
                        if use_mentions && !committed_any.contains(path.as_ref()) {
                            let mut answer = Some(false);
                            for q in &queries {
                                match db.class_mention_answer(path.as_ref(), q, &text) {
                                    Some(true) => {
                                        answer = Some(true);
                                        break;
                                    }
                                    Some(false) => {}
                                    None => answer = None,
                                }
                            }
                            let hit = match answer {
                                Some(hit) => hit,
                                None => {
                                    // Uncoverable entry: one scan answers
                                    // every query and is recorded below.
                                    let scanner = mention_scanner.as_ref().unwrap();
                                    let names = scanner.scan(&text);
                                    let hit = queries
                                        .iter()
                                        .any(|q| names.binary_search(&q.name).is_ok());
                                    scan_rec = Some((path.clone(), text.clone(), names));
                                    hit
                                }
                            };
                            if !hit {
                                return (None, scan_rec);
                            }
                        }
                        let defs =
                            crate::db::collect_file_definitions(&*db as &dyn MirDatabase, sf);
                        let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                        (Some((path.clone(), text, entries)), scan_rec)
                    })
                    .collect::<Vec<_>>()
            }));
            if let Ok(v) = attempt {
                let mut work = Vec::new();
                let mut scanned = Vec::new();
                for (w, rec) in v {
                    if let Some(w) = w {
                        work.push(w);
                    }
                    if let Some(rec) = rec {
                        scanned.push(rec);
                    }
                }
                break (work, scanned);
            }
        };
        // Record the fallback scans regardless of hit/miss: each is a
        // complete, current mention set for its file, so the next round's
        // (and the references gate's) checks become set lookups.
        if let Some(scanner) = &mention_scanner {
            if !scanned.is_empty() {
                let guard = self.db.salsa.read();
                for (file, text, names) in scanned {
                    guard.set_file_class_mentions(&file, &text, scanner.epoch(), names);
                }
            }
        }
        if work.is_empty() {
            return;
        }
        let guard = self.db.salsa.read();
        for (file, text, entries) in &work {
            guard.set_file_class_edges(file, entries.clone());
            self.mark_defs_committed(file, text);
        }
    }

    /// Declaration name span for a global constant. Constant slices carry no
    /// stored location, so this finds the declaring file via the workspace
    /// constants index and locates the `const NAME` / `define('NAME'` token
    /// textually.
    fn global_constant_decl_range(&self, fqn: &str) -> Option<(Arc<str>, crate::Range)> {
        use std::panic::AssertUnwindSafe;
        let short = crate::db::subtype_index::short_name_of(fqn).to_string();
        salsa::Cancelled::catch(AssertUnwindSafe(|| {
            let db = self.snapshot_db();
            let index = crate::db::workspace_index(&db);
            let loc = index.constant_loc(mir_types::Name::from(fqn.trim_start_matches('\\')))?;
            let file = loc.file().path(&db).clone();
            let sf = db.lookup_source_file(file.as_ref())?;
            let text = sf.text(&db as &dyn MirDatabase);
            for (idx, line) in text.lines().enumerate() {
                let trimmed = line.trim_start();
                let is_decl_line = trimmed.starts_with("const ")
                    || trimmed.contains("define(")
                    || trimmed.contains("define (");
                if !is_decl_line {
                    continue;
                }
                if let Some(col) = identifier_char_col(line, &short, 0, false) {
                    let n = short.chars().count() as u32;
                    return Some((file, span_range(idx as u32 + 1, col, col + n)));
                }
            }
            None
        }))
        .ok()
        .flatten()
    }

    /// Class-level issues (inheritance violations, abstract-method gaps, override
    /// incompatibilities) for the given set of files.
    ///
    /// These checks are cross-file by nature and are not emitted by
    /// [`crate::FileAnalyzer::analyze`]. Call this after ingesting or
    /// re-analyzing a file and its dependents to get the full diagnostic picture.
    ///
    /// Circular-inheritance checks always run against the full workspace graph
    /// regardless of the `files` filter — a cycle is a workspace-wide problem.
    pub fn class_issues(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        self.settle_workspace_index();
        let db = self.snapshot_db();
        let file_set: HashSet<Arc<str>> = files.iter().cloned().collect();
        // Read source texts through the snapshot already in hand — calling
        // `source_of` here would re-enter the session RwLock while this
        // snapshot is live, and a concurrent salsa write (which blocks new
        // readers behind the fair write lock while waiting for existing
        // snapshots to drop) turns that into a deadlock.
        let file_data: Vec<(Arc<str>, Arc<str>)> = files
            .iter()
            .filter_map(|f| {
                let sf = db.lookup_source_file(f)?;
                Some((
                    f.clone(),
                    sf.text(&db as &dyn crate::db::MirDatabase).clone(),
                ))
            })
            .collect();
        crate::class::ClassAnalyzer::with_files(&db, file_set, &file_data).analyze_all()
    }

    /// Collector-phase issues (e.g. `BackedEnumCaseTypeMismatch`,
    /// `InvalidReadonlyPropertyDeclaration`, `InvalidDocblock`, and raw parse
    /// errors) for the given files.
    ///
    /// These are found while building a file's declaration slice
    /// ([`crate::db::collect_file_definitions`]), before body analysis or
    /// cross-file class checks ever run — neither [`crate::FileAnalyzer::analyze`]
    /// nor [`Self::class_issues`] reads them, so a caller merging just those
    /// two sources silently drops every collector-time diagnostic. Call this
    /// alongside them to get the full picture.
    ///
    /// A plain snapshot read through [`crate::db::collect_file_definitions`],
    /// same as [`Self::document_symbols`] — correct regardless of which path
    /// put the file's text into the db (`ingest_file`, `set_file_text`, lazy
    /// vendor load) or how many times. The parse-cache fast paths behind that
    /// query used to zero out `issues` on any hit, including re-collecting
    /// the *same* file after its salsa memo was invalidated; they now
    /// preserve (and, for a genuinely different file sharing content,
    /// re-point) the originally-computed issues instead.
    pub fn collector_issues(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        let db = self.snapshot_db();
        files
            .iter()
            .filter_map(|f| db.lookup_source_file(f))
            .flat_map(|sf| {
                crate::db::collect_file_definitions(&db, sf)
                    .issues
                    .as_ref()
                    .clone()
            })
            .collect()
    }

    /// All declarations defined in `file` as a **hierarchical tree**.
    ///
    /// Classes/interfaces/traits/enums are returned with their methods,
    /// properties, and constants nested in `children`. Top-level functions
    /// and constants are returned with empty `children`.
    pub fn document_symbols(&self, file: &str) -> Vec<crate::symbol::DocumentSymbol> {
        use crate::symbol::{DeclarationKind, DocumentSymbol};

        let db = self.snapshot_db();
        let Some(sf) = db.lookup_source_file(file) else {
            return Vec::new();
        };
        let defs = crate::db::collect_file_definitions(&db, sf);
        let mut out: Vec<DocumentSymbol> = Vec::new();

        let class_children = |methods: &mir_codebase::definitions::MemberMap<
            Arc<mir_codebase::definitions::MethodDef>,
        >,
                              props: Option<
            &mir_codebase::definitions::MemberMap<mir_codebase::definitions::PropertyDef>,
        >,
                              consts: &mir_codebase::definitions::MemberMap<
            mir_codebase::definitions::ConstantDef,
        >,
                              is_enum: bool|
         -> Vec<DocumentSymbol> {
            let mut out: Vec<DocumentSymbol> = Vec::new();
            for (_, m) in methods.iter() {
                out.push(DocumentSymbol {
                    name: m.name.clone(),
                    kind: DeclarationKind::Method,
                    location: m.location.clone(),
                    children: Vec::new(),
                });
            }
            if let Some(props) = props {
                for (_, p) in props.iter() {
                    out.push(DocumentSymbol {
                        name: p.name.clone(),
                        kind: DeclarationKind::Property,
                        location: p.location.clone(),
                        children: Vec::new(),
                    });
                }
            }
            let const_kind = if is_enum {
                DeclarationKind::EnumCase
            } else {
                DeclarationKind::Constant
            };
            for (_, c) in consts.iter() {
                out.push(DocumentSymbol {
                    name: c.name.clone(),
                    kind: const_kind,
                    location: c.location.clone(),
                    children: Vec::new(),
                });
            }
            out
        };

        for c in defs.slice.classes.iter() {
            out.push(DocumentSymbol {
                name: c.fqcn.clone(),
                kind: DeclarationKind::Class,
                location: c.location.clone(),
                children: class_children(
                    &c.own_methods,
                    Some(&c.own_properties),
                    &c.own_constants,
                    false,
                ),
            });
        }
        for i in defs.slice.interfaces.iter() {
            out.push(DocumentSymbol {
                name: i.fqcn.clone(),
                kind: DeclarationKind::Interface,
                location: i.location.clone(),
                children: class_children(&i.own_methods, None, &i.own_constants, false),
            });
        }
        for t in defs.slice.traits.iter() {
            out.push(DocumentSymbol {
                name: t.fqcn.clone(),
                kind: DeclarationKind::Trait,
                location: t.location.clone(),
                children: class_children(
                    &t.own_methods,
                    Some(&t.own_properties),
                    &t.own_constants,
                    false,
                ),
            });
        }
        for e in defs.slice.enums.iter() {
            let mut children = class_children(&e.own_methods, None, &e.own_constants, true);
            for (_, case) in e.cases.iter() {
                children.push(DocumentSymbol {
                    name: case.name.clone(),
                    kind: DeclarationKind::EnumCase,
                    location: case.location.clone(),
                    children: Vec::new(),
                });
            }
            out.push(DocumentSymbol {
                name: e.fqcn.clone(),
                kind: DeclarationKind::Enum,
                location: e.location.clone(),
                children,
            });
        }
        for f in defs.slice.functions.iter() {
            out.push(DocumentSymbol {
                name: f.fqn.clone(),
                kind: DeclarationKind::Function,
                location: f.location.clone(),
                children: Vec::new(),
            });
        }
        for (name, _) in defs.slice.constants.iter() {
            out.push(DocumentSymbol {
                name: name.clone(),
                kind: DeclarationKind::Constant,
                location: None,
                children: Vec::new(),
            });
        }
        out
    }

    /// Choose the candidate-admission gate for `symbol`.
    ///
    /// For any known non-constructor/non-`__invoke` method, the member name
    /// alone is the sound gate. Every posting-producing reference spells that
    /// token: `$obj->m()`, `Owner::m()`, inherited `Sub::m()`,
    /// `self::`/`static::`/`parent::m()`, callable strings/arrays, and trait
    /// aliases (`orig as alias` records `orig`, alias calls record `alias`
    /// plus the origin key). Dropping the owner short name matters on common
    /// owner names (`User`, `Model`, `Widget`): otherwise a cold reference
    /// query analyzes files that only type-hint the owner and cannot contain a
    /// reference to the queried method. Dynamic member names (`$obj->$m()`)
    /// produce no posting, so nothing is lost there.
    ///
    /// For `__construct` with a known owner, the identifier needle is the
    /// owner's short name (`new Cls(` sites never spell the member name and
    /// the bare word `__construct` would admit every file *declaring* a
    /// constructor), complemented by the raw call tokens `->__construct` /
    /// `::__construct`: an explicit re-init `$obj->__construct()` is a real
    /// recorded reference whose file may never name the class.
    ///
    /// `__invoke` keeps the general owner/name gate because `$obj()` call sites
    /// do not spell `__invoke`.
    ///
    /// Everything else uses the general OR needles
    /// ([`reference_gate_needles`]).
    fn reference_gate(&self, symbol: &crate::Name) -> ReferenceGate {
        if let crate::Name::Method { class, name } = symbol {
            if name.as_ref() == "__construct" && !class.is_empty() {
                return ReferenceGate {
                    idents: reference_gate_needles(symbol),
                    raw: vec!["->__construct".to_string(), "::__construct".to_string()],
                };
            }
            if name.as_ref() != "__construct"
                && name.as_ref() != "__invoke"
                && !class.is_empty()
            {
                let db = self.snapshot_db();
                let here = crate::db::Fqcn::from_str(&db, class.as_ref());
                let is_static = crate::db::find_method_in_chain(&db, here, name)
                    .map(|(_, m)| m.is_static)
                    .unwrap_or(false);
                if is_static || crate::db::class_exists(&db, class.as_ref()) {
                    return ReferenceGate {
                        idents: vec![name.to_string()],
                        raw: Vec::new(),
                    };
                }
            }
        }
        ReferenceGate {
            idents: reference_gate_needles(symbol),
            raw: Vec::new(),
        }
    }
}

/// A transitive subtype hit with its declaration name span, as returned by
/// [`AnalysisSession::indexed_subtype_classes`].
#[derive(Debug, Clone)]
pub struct SubtypeClassSite {
    /// Display-form FQCN (no leading `\`).
    pub fqcn: Arc<str>,
    pub kind: crate::db::ClassLikeKind,
    pub is_abstract: bool,
    pub file: Arc<str>,
    /// The declared name's own token (1-based line, 0-based char columns).
    pub range: crate::Range,
}

/// Build a [`crate::Range`] on one line from mir's native coordinates
/// (1-based line, 0-based columns).
fn span_range(line: u32, col_start: u32, col_end: u32) -> crate::Range {
    crate::Range {
        start: crate::Position {
            line,
            column: col_start,
        },
        end: crate::Position {
            line,
            column: col_end,
        },
    }
}

/// Char column of the first word-boundary occurrence of `needle` in `line`
/// at or after char column `min_col`. Columns are code points, matching the
/// collector's `Location` convention.
fn identifier_char_col(
    line: &str,
    needle: &str,
    min_col: usize,
    case_insensitive: bool,
) -> Option<u32> {
    if needle.is_empty() {
        return None;
    }
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let chars: Vec<char> = line.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    let n = needle_chars.len();
    if chars.len() < n {
        return None;
    }
    for start in min_col..=chars.len().saturating_sub(n) {
        let matches = chars[start..start + n]
            .iter()
            .zip(needle_chars.iter())
            .all(|(a, b)| {
                if case_insensitive {
                    a.eq_ignore_ascii_case(b)
                } else {
                    a == b
                }
            });
        if !matches {
            continue;
        }
        let before_ok = start == 0 || !is_ident(chars[start - 1]);
        let after = start + n;
        let after_ok = after >= chars.len() || !is_ident(chars[after]);
        if before_ok && after_ok {
            return Some(start as u32);
        }
    }
    None
}

/// Whether `hay` mentions `needle` as a whole identifier (ASCII word
/// boundaries; conservative near multibyte text). ASCII-case-insensitive:
/// PHP class, function, and method names are case-insensitive, so `new
/// COLOR()` must count as mentioning `Color`; for the case-sensitive kinds
/// (constants, properties) folding only widens the candidate superset.
///
/// Test-only semantic oracle: the production gates (references freshness,
/// subtype-BFS defs commit) answer this predicate through the persistent
/// `ClassMentionIndex`; the parity test below pins the scanner to these
/// exact boundary and case semantics.
#[cfg(test)]
fn mentions_identifier(hay: &str, needle: &str) -> bool {
    let hay = hay.as_bytes();
    let needle = needle.as_bytes();
    let n = needle.len();
    if n == 0 || hay.len() < n {
        return false;
    }
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let first = needle[0].to_ascii_lowercase();
    for i in 0..=(hay.len() - n) {
        if hay[i].to_ascii_lowercase() != first || !hay[i..i + n].eq_ignore_ascii_case(needle) {
            continue;
        }
        if (i == 0 || !is_ident(hay[i - 1])) && (i + n == hay.len() || !is_ident(hay[i + n])) {
            return true;
        }
    }
    false
}

fn short(fqn: &str) -> &str {
    fqn.rsplit('\\').next().unwrap_or(fqn)
}

/// A candidate-file admission predicate for `indexed_references_to`'s
/// freshness pass, chosen by [`AnalysisSession::reference_gate`]. A file is
/// admitted when its text mentions any of `idents` as a whole identifier
/// (word-bounded, ASCII-case-insensitive) OR contains any of `raw` as a
/// plain substring (ASCII-case-insensitive, no word bounds — used for
/// call-shaped tokens like `->__construct`). A file matching neither can
/// hold no posting for the symbol.
struct ReferenceGate {
    idents: Vec<String>,
    raw: Vec<String>,
}

/// Identifier words whose whole-word presence in a file's text is necessary
/// for the file to hold any posting [`AnalysisSession::indexed_references_to`]
/// can return for `symbol`. Member symbols include the owner class's short
/// name alongside the member name: `__construct` postings are recorded at
/// `new Cls(` sites, which never spell the member name.
fn reference_gate_needles(symbol: &crate::Name) -> Vec<String> {
    let mut needles = match symbol {
        crate::Name::Class(f) | crate::Name::Function(f) | crate::Name::GlobalConstant(f) => {
            vec![short(f).to_string()]
        }
        // `__construct` is invoked only as `new Cls(...)`, `parent::__construct()`,
        // or `self::__construct()`/`static::__construct()` from inside a
        // subclass — every real call site textually names the class itself
        // (directly, or via the enclosing subclass's own `extends`/`use`),
        // never the bare word `__construct`. Gating on the class's short name
        // alone is exact (no lost call sites) and, unlike the general member
        // case, dropping the method-name needle here doesn't reintroduce a
        // false negative. This matters: `__construct` is one of the most
        // common tokens in any real codebase, so OR-ing it in as a needle
        // admits nearly every file as a "must re-analyze" candidate on a
        // cold query, defeating the gate's entire purpose for constructors.
        crate::Name::Method { class, name } if name.as_ref() == "__construct" => {
            if class.is_empty() {
                // No class to scope to (owner unknown) — fall back to gating
                // on the bare name, same as the general member case below.
                vec![name.to_string()]
            } else {
                vec![short(class).to_string()]
            }
        }
        crate::Name::Method { class, name }
        | crate::Name::Property { class, name }
        | crate::Name::ClassConstant { class, name } => {
            let mut v = vec![name.to_string()];
            if !class.is_empty() {
                v.push(short(class).to_string());
            }
            v
        }
    };
    // An empty needle can never match; dropping it keeps the "empty needle
    // set disables the gate" contract at the call site conservative.
    needles.retain(|n| !n.is_empty());
    needles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentions_identifier_is_case_insensitive_and_word_bounded() {
        assert!(mentions_identifier("$this->save();", "save"));
        assert!(mentions_identifier("new COLOR()", "Color"));
        assert!(mentions_identifier("use App\\Color as Paint;", "color"));
        assert!(!mentions_identifier("$this->saveAll();", "save"));
        assert!(!mentions_identifier("return $unsaved;", "save"));
        assert!(!mentions_identifier("no occurrence", "save"));
        assert!(!mentions_identifier("anything", ""));
        // Multibyte neighbors are conservatively treated as boundaries, and
        // substring scans must not split codepoints.
        assert!(!mentions_identifier("function xÉclairFoo() {}", "Éclair"));
        assert!(mentions_identifier("implements Éclair {}", "Éclair"));
    }

    #[test]
    fn mention_scanner_membership_equals_per_needle_scans() {
        // The mention index is the sole implementation of the gates' textual
        // predicate, so scanner membership must equal the reference
        // per-needle predicate for every (hay, needle) pair — same boundary
        // and case semantics.
        use crate::db::MentionScanner;
        use std::sync::Arc;
        let universe = ["Color", "save", "ColorPicker", "Éclair", "C1", "_Wrap"];
        let names: Vec<(mir_types::Name, bool)> = universe
            .iter()
            .map(|s| (mir_types::Name::new(s).ascii_lowercase(), false))
            .collect();
        let scanner = Arc::new(MentionScanner::build(1, names).unwrap());
        let hays = [
            "$this->save();",
            "new COLOR()",
            "use App\\Color as Paint;",
            "$this->saveAll();",
            "return $unsaved;",
            "new ColorPicker(); Color::save();",
            "function xÉclairFoo() {}",
            "implements Éclair {}",
            "colorsave savecolor color_save",
            "class C1 extends _Wrap {}",
            "",
        ];
        for hay in hays {
            let scanned = scanner.scan(hay);
            for needle in universe {
                let expected = mentions_identifier(hay, needle);
                let name = mir_types::Name::new(needle).ascii_lowercase();
                assert_eq!(
                    scanned.binary_search(&name).is_ok(),
                    expected,
                    "needle {needle:?} on {hay:?}"
                );
            }
        }
    }

    #[test]
    fn gate_needles_cover_member_and_owner_class() {
        // A regular member (non-constructor) gates on both the member name
        // and the owner's short name — a call site may name only one.
        let n = reference_gate_needles(&crate::Name::method("App\\Job", "run"));
        assert!(n.contains(&"run".to_string()) && n.contains(&"Job".to_string()));
        let n = reference_gate_needles(&crate::Name::class("App\\Ui\\Color"));
        assert_eq!(n, vec!["Color".to_string()]);
        // Unknown-owner member symbols still gate on the member name alone.
        let n = reference_gate_needles(&crate::Name::method("", "run"));
        assert_eq!(n, vec!["run".to_string()]);
    }

    #[test]
    fn gate_needles_for_constructor_scope_to_owner_class_only() {
        // `__construct` is only ever spelled at `new Cls(`/`parent::__construct()`
        // sites, which always name the class — the bare method-name needle is
        // dropped so a cold constructor query doesn't admit nearly every file
        // in the workspace (every class defines *some* `__construct`).
        let n = reference_gate_needles(&crate::Name::method("App\\Job", "__construct"));
        assert_eq!(n, vec!["Job".to_string()]);
        // Unknown owner: nothing to scope to, fall back to the bare name.
        let n = reference_gate_needles(&crate::Name::method("", "__construct"));
        assert_eq!(n, vec!["__construct".to_string()]);
    }

    fn session_with(files: &[(&str, &str)]) -> crate::AnalysisSession {
        let session = crate::AnalysisSession::new(crate::PhpVersion::LATEST);
        for (path, text) in files {
            session.set_file_text(Arc::from(*path), Arc::from(*text));
        }
        session
    }

    #[test]
    fn gate_static_method_is_member_name_only() {
        // Regardless of subtypes: the member token alone is the sound gate
        // (an instance receiver `$obj::m()` never names the owner), and it
        // is also the selective part — the owner short name would only
        // widen the admitted set.
        let session = session_with(&[
            (
                "owner.php",
                "<?php\nclass Owner { public static function m(): void {} }\n",
            ),
            ("sub.php", "<?php\nclass Sub extends Owner {}\n"),
        ]);
        let gate = session.reference_gate(&crate::Name::method("Owner", "m"));
        assert_eq!(gate.idents, vec!["m".to_string()]);
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_instance_method_is_member_name_only() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function m(): void {} }\n",
        )]);
        let gate = session.reference_gate(&crate::Name::method("Owner", "m"));
        assert_eq!(gate.idents, vec!["m".to_string()]);
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_invoke_keeps_owner_needle() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function __invoke(): void {} }\n",
        )]);
        let gate = session.reference_gate(&crate::Name::method("Owner", "__invoke"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Owner", "__invoke"))
        );
        assert!(gate.raw.is_empty());
    }

    #[test]
    fn gate_constructor_adds_raw_call_tokens() {
        // Owner short name for `new Cls(` sites, plus the raw call tokens
        // for explicit re-init (`$obj->__construct()`) whose file may never
        // name the class. The bare identifier `__construct` must NOT be a
        // needle — it would admit every file declaring a constructor.
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public function __construct() {} }\n",
        )]);
        let gate = session.reference_gate(&crate::Name::method("Owner", "__construct"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Owner", "__construct"))
        );
        assert_eq!(
            gate.raw,
            vec!["->__construct".to_string(), "::__construct".to_string()]
        );
    }

    #[test]
    fn gate_unresolvable_owner_falls_back_to_general_needles() {
        let session = session_with(&[(
            "owner.php",
            "<?php\nclass Owner { public static function m(): void {} }\n",
        )]);
        let gate = session.reference_gate(&crate::Name::method("Nonexistent\\Missing", "m"));
        assert_eq!(
            gate.idents,
            reference_gate_needles(&crate::Name::method("Nonexistent\\Missing", "m"))
        );
        assert!(gate.raw.is_empty());
    }
}
