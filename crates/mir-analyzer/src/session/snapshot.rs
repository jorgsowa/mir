use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use rustc_hash::FxHashSet as HashSet;
use salsa::Cancelled;

use super::index_state::{
    hash_files, AnalyzedFile, IndexState, RefQueryCacheKey, SubtypeQueryCacheKey,
};
use super::queries::{identifier_char_col, span_range, ReferenceGate};
use super::SubtypeClassSite;
use crate::cache::AnalysisCache;
use crate::db::{MirDatabase, MirDbStorage};
use crate::php_version::PhpVersion;

/// A read handle on one revision of an [`super::AnalysisSession`]:
/// `Send + Clone`, for running queries on threads other than the owner's.
///
/// No query writes salsa inputs. Class-likes and built-in stubs the symbol
/// index lacks load on demand as pure reads; the owner adopts them at its
/// next [`super::AnalysisSession::prepare_for_query`], which also settles
/// the index before a snapshot is handed out.
///
/// A snapshot blocks the owner's next input write until it is dropped (salsa
/// waits for outstanding handles), so hand snapshots out per request and
/// never park one. When the owner starts a write, in-flight queries unwind
/// and return `Err(Cancelled)`: drop the snapshot and request a fresh one.
///
/// Queries may still commit to the shared reference/subtype indexes (lazy
/// freshness passes); those commits land in the owner's state too.
#[derive(Clone)]
pub struct AnalysisSnapshot {
    pub(super) db: MirDbStorage,
    pub(super) index: Arc<IndexState>,
    pub(super) cache: Option<Arc<AnalysisCache>>,
    pub(super) php_version: PhpVersion,
    /// [`Self::index_generation`] captured when the owner created the
    /// snapshot, while no write could race it.
    pub(super) index_generation: u64,
}

const _: () = {
    const fn assert_send_clone<T: Send + Clone>() {}
    assert_send_clone::<AnalysisSnapshot>();
};

/// An [`AnalysisSnapshot`] that borrows its owner, so the owner's next
/// `&mut self` write can't start while it's alive. A live handle across that
/// write would deadlock the owner in salsa's `cancel_others`.
pub(crate) struct DbView<'a> {
    snapshot: AnalysisSnapshot,
    _owner: std::marker::PhantomData<&'a super::AnalysisSession>,
}

impl<'a> DbView<'a> {
    pub(super) fn new(_owner: &'a super::AnalysisSession, snapshot: AnalysisSnapshot) -> Self {
        Self {
            snapshot,
            _owner: std::marker::PhantomData,
        }
    }
}

impl std::ops::Deref for DbView<'_> {
    type Target = AnalysisSnapshot;
    fn deref(&self) -> &AnalysisSnapshot {
        &self.snapshot
    }
}

impl std::ops::DerefMut for DbView<'_> {
    fn deref_mut(&mut self) -> &mut AnalysisSnapshot {
        &mut self.snapshot
    }
}

/// Analyses staged by [`AnalysisSnapshot::stage_warm`], awaiting commit.
struct WarmPass {
    mention_scanner: Option<Arc<crate::db::class_mention_index::MentionScanner>>,
    analyzed: Vec<AnalyzedFile>,
}

/// A fallback mention scan `(file, text scanned, names found)`, recorded
/// after the pass so later gate checks become set lookups.
type MentionScanRecord = (Arc<str>, Arc<str>, Box<[mir_types::Name]>);

pub(super) fn catch<T>(f: impl FnOnce() -> T) -> Result<T, Cancelled> {
    Cancelled::catch(AssertUnwindSafe(f))
}

impl AnalysisSnapshot {
    /// The salsa database this snapshot reads.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice. Queries on it unwind with [`Cancelled`] rather than returning
    /// it; use [`Self::read`] to catch that.
    #[doc(hidden)]
    pub fn db(&self) -> &MirDbStorage {
        &self.db
    }

    /// Run `f` against the database, catching cancellation.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice.
    #[doc(hidden)]
    pub fn read<R>(&self, f: impl FnOnce(&dyn MirDatabase) -> R) -> Result<R, Cancelled> {
        catch(|| f(&self.db))
    }

    pub(crate) fn index(&self) -> &IndexState {
        &self.index
    }

    pub fn php_version(&self) -> PhpVersion {
        self.php_version
    }

    /// See [`super::AnalysisSession::index_generation`].
    pub fn index_generation(&self) -> u64 {
        self.index_generation
    }

    /// See [`super::AnalysisSession::text_revision`].
    pub fn text_revision(&self) -> salsa::Revision {
        self.db.current_revision()
    }

    /// The combined generation the query memos key on: salsa's text revision
    /// plus the off-salsa subtype-edge epoch. The epoch covers what the
    /// revision can't — subtype edges and anonymous-class `impl:` postings
    /// committed *within* one revision (e.g. a subtype BFS admitting a file
    /// the reference gate skipped), which change a member query's hierarchy
    /// fan-out without any text write.
    pub(crate) fn query_cache_generation(&self) -> (salsa::Revision, u64) {
        (self.db.current_revision(), self.db.subtype_edges_epoch())
    }

    pub fn find_class_like(&self, fqcn: &str) -> Result<Option<crate::db::ClassLike>, Cancelled> {
        self.read(|db| crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, fqcn)))
    }

    pub fn find_function(&self, fqn: &str) -> Result<Option<Arc<crate::FunctionDef>>, Cancelled> {
        self.read(|db| crate::db::find_function(db, crate::db::Fqcn::from_str(db, fqn)))
    }

    /// `method` as resolved along `class`'s ancestor chain (then mixins), as
    /// `(declaring class, method)`.
    #[allow(clippy::type_complexity)]
    pub fn find_method_in_chain(
        &self,
        class: &str,
        method: &str,
    ) -> Result<Option<(Arc<str>, Arc<mir_codebase::definitions::MethodDef>)>, Cancelled> {
        self.read(|db| {
            crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, class), method)
        })
    }

    /// Declaration location of `symbol` among already-loaded state.
    pub fn definition_of_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<Result<mir_types::Location, crate::SymbolLookupError>, Cancelled> {
        catch(|| self.definition_of_unguarded(symbol))
    }

    fn definition_of_unguarded(
        &self,
        symbol: &crate::Name,
    ) -> Result<mir_types::Location, crate::SymbolLookupError> {
        let db = &self.db;
        match symbol {
            crate::Name::Class(fqcn) => {
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                let class = crate::db::find_class_like(db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                class
                    .location()
                    .cloned()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(db, fqn.as_ref());
                let f =
                    crate::db::find_function(db, here).ok_or(crate::SymbolLookupError::NotFound)?;
                f.location
                    .clone()
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Name::Method { class, name }
            | crate::Name::Property { class, name }
            | crate::Name::ClassConstant { class, name } => {
                crate::db::member_location(db, class, name)
                    .ok_or(crate::SymbolLookupError::NotFound)
            }
            crate::Name::GlobalConstant(_) => Err(crate::SymbolLookupError::NoSourceLocation),
        }
    }

    /// Hover payload for `symbol` among already-loaded state.
    pub fn hover_cached(
        &self,
        symbol: &crate::Name,
    ) -> Result<Result<crate::HoverInfo, crate::SymbolLookupError>, Cancelled> {
        catch(|| self.hover_unguarded(symbol))
    }

    fn hover_unguarded(
        &self,
        symbol: &crate::Name,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Type};
        let db = &self.db;
        match symbol {
            crate::Name::Function(fqn) => {
                let here = crate::db::Fqcn::from_str(db, fqn.as_ref());
                let f =
                    crate::db::find_function(db, here).ok_or(crate::SymbolLookupError::NotFound)?;
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
                let here = crate::db::Fqcn::from_str(db, class.as_ref());
                let (_, m) = crate::db::find_method_in_chain(db, here, name)
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
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                let class = crate::db::find_class_like(db, here)
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
                let here = crate::db::Fqcn::from_str(db, class.as_ref());
                let (_, p) = crate::db::find_property_in_chain(db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = p.ty.as_deref().cloned().unwrap_or_else(Type::mixed);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: p.location.clone(),
                })
            }
            crate::Name::ClassConstant { class, name } => {
                let here = crate::db::Fqcn::from_str(db, class.as_ref());
                let (_, c) = crate::db::find_class_constant_in_chain(db, here, name)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: c.ty.clone(),
                    docstring: None,
                    definition: c.location.clone(),
                })
            }
            crate::Name::GlobalConstant(fqn) => {
                let here = crate::db::Fqcn::from_str(db, fqn.as_ref());
                let ty = crate::db::find_global_constant(db, here)
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                Ok(crate::HoverInfo {
                    ty: (*ty).clone(),
                    docstring: None,
                    definition: None,
                })
            }
        }
    }

    /// Which of `files` currently mention `class_name` as a whole identifier
    /// (case-insensitive) — the same persistent, incrementally-maintained
    /// mechanism [`Self::indexed_references_to`]'s own gate uses (see
    /// `db::class_mention_index`), exposed so a host needn't re-implement a
    /// from-scratch text scan for its own narrowing. A file already scanned
    /// against a universe that included `class_name` answers via an O(log n)
    /// lookup; only a never-scanned or since-edited file pays a scan, and
    /// that scan is recorded for every other name in the universe too.
    pub fn files_mentioning_class(
        &self,
        files: &[Arc<str>],
        class_name: &str,
    ) -> Result<Vec<Arc<str>>, Cancelled> {
        self.files_mentioning_any(files, &[class_name])
    }

    /// Which of `files` currently mention ANY of `needles` as a whole
    /// identifier (case-insensitive). Every needle is admitted into the
    /// shared universe verbatim (no short-name stripping) before querying,
    /// so the answer is never conservative on account of an unknown needle.
    pub fn files_mentioning_any(
        &self,
        files: &[Arc<str>],
        needles: &[&str],
    ) -> Result<Vec<Arc<str>>, Cancelled> {
        catch(|| self.mentioning_any_unguarded(files, needles))
    }

    fn mentioning_any_unguarded(&self, files: &[Arc<str>], needles: &[&str]) -> Vec<Arc<str>> {
        use rayon::prelude::*;

        if needles.is_empty() {
            return files.to_vec();
        }
        let db = self.db.clone();
        db.add_literal_mention_names(needles.iter().copied());
        // Admission just grew the universe, so these only fail defensively.
        let queries: Vec<_> = needles
            .iter()
            .filter_map(|n| db.prepare_class_mention_query(n))
            .collect();
        if queries.is_empty() {
            return files.to_vec();
        }
        let Some(scanner) = db.class_mention_scanner() else {
            return files.to_vec();
        };

        files
            .par_iter()
            .map_with(db, |db, f| -> Option<Arc<str>> {
                let sf = db.lookup_source_file(f.as_ref())?;
                let text = sf.text(&*db as &dyn MirDatabase).clone();
                for q in &queries {
                    match db.class_mention_answer(f.as_ref(), q, &text) {
                        Some(true) => return Some(f.clone()),
                        Some(false) => continue,
                        None => {
                            let names = scanner.scan(text.as_ref());
                            let hit = queries
                                .iter()
                                .any(|q2| names.binary_search(&q2.name).is_ok());
                            db.set_file_class_mentions(f, &text, scanner.epoch(), names);
                            return hit.then(|| f.clone());
                        }
                    }
                }
                None
            })
            .flatten()
            .collect()
    }

    /// Inverted-index find-references: posting-list lookup plus an on-demand
    /// freshness/completeness pass over `files` (the host's candidate scope
    /// — passing the whole workspace is fine; see the gate in
    /// `stale_reference_candidates`).
    ///
    /// A candidate whose postings were committed from its current text (Arc
    /// identity) is answered from the index with no salsa work. Stale or
    /// never-committed candidates are analyzed via the memoized
    /// `analyze_file` query and committed, so each file pays that cost once
    /// per text change — after a background warm sweep the steady state is a
    /// pure lookup, O(results) instead of O(candidates). Hosts need no text
    /// prefilter of their own, and must not use one: a host-side filter
    /// cannot know the gate's matching semantics.
    ///
    /// Results are filtered to `files`. With `include_declaration`, the
    /// symbol's declaration name span is appended when it lies inside the
    /// scope.
    ///
    /// Returned ranges use mir's native coordinates: 1-based lines and
    /// 0-based Unicode code-point columns (UTF-32/LSP `positionEncoding`
    /// `"utf-32"`).
    ///
    /// Stale candidates are analyzed against whatever the owner has loaded.
    /// A candidate whose warm-up the owner never ran may reference a class
    /// that isn't loaded yet; its commit then records unresolved names and
    /// is re-verified after the owner's next load bumps the generation.
    ///
    /// Memoized per `(symbol, files, include_declaration, includes,
    /// query generation)` — see `RefQueryCacheKey` — so a repeat query
    /// against unchanged state (a code-lens refresh) is one hashmap lookup
    /// instead of an O(candidates) freshness scan.
    pub fn indexed_references_to(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
    ) -> Result<Vec<(Arc<str>, crate::Range)>, Cancelled> {
        catch(|| {
            let key = self.reference_query_key(symbol, files, include_declaration, includes);
            if let Some(hit) = self.index.ref_queries.get(&key) {
                return hit;
            }
            let stale = self.stale_reference_candidates(symbol, files);
            if !stale.is_empty() {
                self.commit_reference_candidates(&stale);
            }
            let out = self.read_references(symbol, files, include_declaration, includes);
            self.memoize_references(key, &out);
            out
        })
    }

    /// `use` import items (`use Foo\Bar;`, `use function ...;`,
    /// `use const ...;`) for `symbol` — [`Self::indexed_references_to`] with
    /// [`crate::ReferenceIncludes::UseImports`].
    pub fn indexed_use_import_locations(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Result<Vec<(Arc<str>, crate::Range)>, Cancelled> {
        self.indexed_references_to(symbol, files, false, crate::ReferenceIncludes::UseImports)
    }

    pub(super) fn reference_query_key(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
    ) -> RefQueryCacheKey {
        RefQueryCacheKey {
            symbol: symbol.codebase_key(),
            include_declaration,
            includes,
            generation: self.query_cache_generation(),
            files_hash: hash_files(files),
        }
    }

    /// Cache `out` unless the generation moved while computing it (an edit,
    /// or a defs commit growing the subtype index — possibly this query's
    /// own): such a key can never be looked up again, and the next identical
    /// query recomputes once against settled state and caches then.
    pub(super) fn memoize_references(
        &self,
        key: RefQueryCacheKey,
        out: &[(Arc<str>, crate::Range)],
    ) {
        let generation = self.query_cache_generation();
        if generation == key.generation {
            self.index.ref_queries.insert(generation, key, out);
        }
    }

    /// Freshness pass: candidates whose postings are not exact for their
    /// current text. Files not registered as `SourceFile` inputs are
    /// skipped. Never-committed files — no commit mark, hence no postings at
    /// all (every mark drop accompanies a posting clear) — are further gated
    /// on their text mentioning the symbol's name: such a file can neither
    /// hold stale postings nor produce new ones, so a cold query on a common
    /// name skips the bulk of the workspace instead of analyzing it. A
    /// LIVE-analyzed file stale only by generation gets the same gate.
    /// Everything else — a genuinely edited file, or a commit seeded by an
    /// unverified disk-cache replay — re-analyzes unconditionally. Same
    /// discipline as [`Self::commit_defs_for_matching`] on the defs index.
    ///
    /// The whole gate answers from the persistent mention index: every
    /// needle is admitted, so a recorded mention set answers with lookups
    /// and only a never-scanned or since-edited file pays one scan against
    /// the whole universe, recorded for every later consumer. A needle new
    /// to the universe epoch-invalidates older recordings for itself only.
    ///
    /// Intentionally serial: each request already runs on its caller
    /// thread, and putting every concurrent request back onto the shared
    /// rayon pool lets an index batch monopolize the workers.
    pub(super) fn stale_reference_candidates(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
    ) -> Vec<Arc<str>> {
        let db = &self.db;
        let gate = self.reference_gate(symbol);
        let has_needles = !gate.idents.is_empty() || !gate.raw.is_empty();
        let (mention_queries, mention_scanner) = if has_needles {
            db.add_literal_mention_names(gate.idents.iter().map(|s| s.as_str()));
            db.add_raw_mention_needles(gate.raw.iter().map(|s| s.as_str()));
            let queries: Vec<_> = gate
                .idents
                .iter()
                .chain(gate.raw.iter())
                .filter_map(|s| db.prepare_class_mention_query(s))
                .collect();
            (queries, db.class_mention_scanner())
        } else {
            (Vec::new(), None)
        };
        // Admission guarantees a query per needle and a non-empty universe;
        // anything else is defensive — the gate then admits every candidate
        // (analyze rather than skip, the conservative direction).
        let gate_complete = mention_queries.len() == gate.idents.len() + gate.raw.len()
            && mention_scanner.is_some();
        let committed_any: HashSet<Arc<str>> =
            self.index.ref_committed_keys().into_iter().collect();
        let current_gen = self.index_generation;
        let mut stale = Vec::new();
        let mut scanned: Vec<MentionScanRecord> = Vec::new();
        for f in files {
            let Some(sf) = db.lookup_source_file(f.as_ref()) else {
                continue;
            };
            let text = sf.text(db as &dyn MirDatabase);
            if self.index.is_ref_committed(f.as_ref(), text, current_gen) {
                continue;
            }
            if committed_any.contains(f.as_ref())
                && !self
                    .index
                    .ref_commit_stale_by_generation_only(f.as_ref(), text)
            {
                stale.push(f.clone());
                continue;
            }
            if has_needles && gate_complete {
                // Any needle answering `true` admits the file; an
                // unanswerable one forces the single recorded scan, which
                // settles every needle at once.
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
                    Some(false) => continue,
                    None => {
                        let scanner = mention_scanner
                            .as_ref()
                            .expect("gate_complete implies a scanner");
                        let names = scanner.scan(text);
                        let hit = mention_queries
                            .iter()
                            .any(|q| names.binary_search(&q.name).is_ok());
                        scanned.push((f.clone(), text.clone(), names));
                        if hit {
                            stale.push(f.clone());
                        }
                        continue;
                    }
                }
            }
            stale.push(f.clone());
        }
        // Recorded regardless of how the query proceeds: each is a complete,
        // current mention set for its file.
        if let Some(scanner) = &mention_scanner {
            for (file, text, names) in scanned {
                db.set_file_class_mentions(&file, &text, scanner.epoch(), names);
            }
        }
        stale
    }

    /// Analyze `stale` and commit their postings and class edges.
    pub(super) fn commit_reference_candidates(&self, stale: &[Arc<str>]) {
        let _ = self.warm_pass(stale, &crate::IndexCancel::new());
    }

    /// Analyze `files` and commit their reference postings, class edges and
    /// mention scans to the index shared with the owner, so later queries
    /// find them fresh. `Ok(false)` when `cancel` stopped the pass first.
    ///
    /// A commit with unresolved names stays tied to this snapshot's
    /// [`Self::index_generation`], so the next workspace change re-opens it.
    pub fn warm_files(
        &self,
        files: &[Arc<str>],
        cancel: &crate::IndexCancel,
    ) -> Result<bool, Cancelled> {
        Ok(self.warm_pass(files, cancel)?.is_some())
    }

    pub(super) fn warm_pass(
        &self,
        files: &[Arc<str>],
        cancel: &crate::IndexCancel,
    ) -> Result<Option<Vec<AnalyzedFile>>, Cancelled> {
        let Some(mut pass) = self.stage_warm(files, cancel)? else {
            return Ok(None);
        };
        self.commit_warm(&mut pass);
        Ok(Some(pass.analyzed))
    }

    /// Freezes the workspace index on a pass-scoped clone (borrow-only
    /// symbol lookups + pass-shared subtype cache): a snapshot never
    /// lazy-loads, and a concurrent write cancels the pass, so the frozen
    /// view is never stale. Same discipline as the batch body pass.
    fn stage_warm(
        &self,
        files: &[Arc<str>],
        cancel: &crate::IndexCancel,
    ) -> Result<Option<WarmPass>, Cancelled> {
        catch(|| {
            let mut db = self.db.clone();
            db.freeze_workspace_index();
            let mention_scanner = db.class_mention_scanner();
            let cache = self.cache.as_deref();
            let mut analyzed = Vec::with_capacity(files.len());
            for path in files {
                if cancel.is_cancelled() {
                    return None;
                }
                analyzed.extend(self.index.stage_analyzed(
                    &db,
                    cache,
                    mention_scanner.as_deref(),
                    path,
                ));
            }
            Some(WarmPass {
                mention_scanner,
                analyzed,
            })
        })
    }

    /// Marks record the exact text Arc each file was analyzed against, so an
    /// owner text write racing the pass leaves the file stale, never fresh.
    fn commit_warm(&self, pass: &mut WarmPass) {
        if self.index.commit_analyzed(
            &self.db,
            self.cache.as_deref(),
            pass.mention_scanner.as_deref(),
            &mut pass.analyzed,
            self.index_generation,
        ) {
            self.index.clear_dependency_graph_cache();
        }
    }

    /// Posting lookup for `symbol`, filtered to the candidate scope.
    ///
    /// Member symbols resolve against the queried class plus its hierarchy
    /// (mir records member refs under the *declaring* class, so a query on
    /// an interface method must include implementor keys and vice versa).
    /// Name-only fallback postings — receivers whose type couldn't be
    /// resolved — are consulted only when the typed keys produce nothing:
    /// exact results when resolution succeeds, by-name matches when nothing
    /// resolves. `__construct` stays exact: `new Sub()` invokes
    /// `Sub::__construct` even when only a parent declares one, so hierarchy
    /// fan-out would wrongly return subtype instantiation sites for a parent
    /// query.
    pub(super) fn read_references(
        &self,
        symbol: &crate::Name,
        files: &[Arc<str>],
        include_declaration: bool,
        includes: crate::ReferenceIncludes,
    ) -> Vec<(Arc<str>, crate::Range)> {
        let key = symbol.codebase_key();
        let hierarchy: Vec<String> = match symbol {
            crate::Name::Method { class, name } => {
                if class.is_empty() {
                    Vec::new()
                } else if name.as_ref() == "__construct" {
                    vec![class.trim_start_matches('\\').to_string()]
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
        let scope: HashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        let read_symbol_key = |symbol_key: &str| -> Vec<(Arc<str>, crate::Range)> {
            self.db
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
        if !matches!(includes, crate::ReferenceIncludes::UseImports) {
            let member_prefix = match symbol {
                crate::Name::Method { name, .. } => Some(("meth:", name)),
                crate::Name::Property { name, .. } => Some(("prop:", name)),
                crate::Name::ClassConstant { name, .. } => Some(("cnst:", name)),
                _ => None,
            };
            match member_prefix {
                Some((prefix, name)) => {
                    for class in &hierarchy {
                        out.extend(read_composed_key(prefix, class, "::", name));
                    }
                }
                None => out.extend(read_symbol_key(key.as_str())),
            }
        }
        if !matches!(includes, crate::ReferenceIncludes::Plain) {
            out.extend(read_composed_key("use:", key.as_str(), "", ""));
        }
        if matches!(includes, crate::ReferenceIncludes::Plain) && out.is_empty() {
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
            let decls: Vec<(Arc<str>, crate::Range)> = match symbol {
                crate::Name::Method { class, name }
                | crate::Name::Property { class, name }
                | crate::Name::ClassConstant { class, name } => {
                    if class.is_empty() {
                        // Unknown owner: declarations by name, recorded as
                        // `methdecl:`/`propdecl:`/`cnstdecl:` postings during
                        // class/trait/interface/enum analysis.
                        let prefix = match symbol {
                            crate::Name::Method { .. } => "methdecl:",
                            crate::Name::Property { .. } => "propdecl:",
                            _ => "cnstdecl:",
                        };
                        read_composed_key(prefix, name.as_ref(), "", "")
                    } else {
                        self.member_decl_sites(&hierarchy, symbol)
                    }
                }
                _ => self.declaration_name_range(symbol).into_iter().collect(),
            };
            for (file, range) in decls {
                if scope.contains(file.as_ref())
                    && !out.iter().any(|(f, r)| *f == file && *r == range)
                {
                    out.push((file, range));
                }
            }
        }
        out
    }

    /// The queried class plus every class its members' references could be
    /// keyed under: resolved ancestors (a call on a subtype instance records
    /// the declaring ancestor) and transitive subtypes including trait users
    /// (a call on a subtype that overrides records the subtype). Display-form
    /// FQCNs, deduplicated case-insensitively.
    fn member_hierarchy_classes(&self, class_fqn: &str) -> Vec<String> {
        let target = class_fqn.trim_start_matches('\\');
        let mut out = vec![target.to_string()];
        let db = &self.db;
        let here = crate::db::Fqcn::from_str(db, target);
        // Ancestors include the class itself first.
        let ancestors = crate::db::class_ancestors_by_fqcn(db, here)
            .iter()
            .skip(1)
            .map(|a| a.trim_start_matches('\\').to_string())
            .collect::<Vec<_>>();
        out.extend(ancestors);
        out.extend(
            db.subtype_sites_of(target, true)
                .into_iter()
                .map(|s| s.fqcn.trim_start_matches('\\').to_string()),
        );
        let mut seen: HashSet<String> = HashSet::default();
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
        let db = &self.db;
        for class in classes {
            let here = crate::db::Fqcn::from_str(db, class);
            let (loc, needle) = match symbol {
                crate::Name::Method { name, .. } => {
                    let Some(m) = crate::db::find_method_in_class(db, here, name) else {
                        continue;
                    };
                    (m.location.clone(), name.to_string())
                }
                crate::Name::Property { name, .. } => {
                    let Some(p) = crate::db::find_property_in_class(db, here, name) else {
                        continue;
                    };
                    (p.location.clone(), name.to_string())
                }
                crate::Name::ClassConstant { name, .. } => {
                    let Some(c) = crate::db::find_class_constant_in_class(db, here, name) else {
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

    /// The symbol's declaration site among already-loaded state, narrowed
    /// from the collector's whole-declaration span to the declared name's
    /// own token (matching the span shape of recorded references).
    pub(super) fn declaration_name_range(
        &self,
        symbol: &crate::Name,
    ) -> Option<(Arc<str>, crate::Range)> {
        if let crate::Name::GlobalConstant(fqn) = symbol {
            return self.global_constant_decl_range(fqn);
        }
        let loc = self.definition_of_unguarded(symbol).ok()?;
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
        let range = self.refine_location_to_name(&loc, short);
        Some((loc.file.clone(), range))
    }

    /// Narrow a whole-declaration [`mir_types::Location`] to the first
    /// word-boundary occurrence of `needle` inside its line span. Falls back
    /// to the location's own coordinates when the text is unavailable or the
    /// name doesn't appear (e.g. stub-only declarations).
    fn refine_location_to_name(&self, loc: &mir_types::Location, needle: &str) -> crate::Range {
        let fallback = span_range(loc.line, loc.col_start as u32, loc.col_end as u32);
        let Some(text) = self
            .db
            .lookup_source_file(loc.file.as_ref())
            .map(|sf| sf.text(&self.db as &dyn MirDatabase).clone())
        else {
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

    /// Declaration name span for a global constant. Constant slices carry no
    /// stored location, so this finds the declaring file via the workspace
    /// constants index and locates the `const NAME` / `define('NAME'` token
    /// textually.
    fn global_constant_decl_range(&self, fqn: &str) -> Option<(Arc<str>, crate::Range)> {
        let short = crate::db::subtype_index::short_name_of(fqn).to_string();
        let db = &self.db;
        let index = crate::db::workspace_index(db);
        let loc = index.constant_loc(mir_types::Name::from(fqn.trim_start_matches('\\')))?;
        let file = loc.file().path(db).clone();
        let sf = db.lookup_source_file(file.as_ref())?;
        let text = sf.text(db as &dyn MirDatabase);
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
    }

    /// Files declaring transitive subclasses of `class_fqn` anywhere in the
    /// workspace, excluding `class_fqn`'s own declaring file — the caller
    /// adds it. Subclasses are matched by resolved FQCN, so `extends \Ns\Base`
    /// and aliased `use` forms are all found.
    pub fn subtype_files(&self, class_fqn: &str) -> Result<Vec<Arc<str>>, Cancelled> {
        let files = self.db.source_file_paths();
        let mut out: Vec<Arc<str>> = self
            .indexed_subtype_classes(class_fqn, &files, false)?
            .into_iter()
            .map(|s| s.file)
            .collect();
        out.sort();
        out.dedup();
        Ok(out)
    }

    /// Transitive subtypes of `class_fqn` (classes/interfaces/enums whose
    /// resolved ancestor chain reaches it), answered from the maintained
    /// subtype edge index.
    ///
    /// `files` is the host's candidate scope for the on-demand completeness
    /// pass: per BFS round, not-yet-committed files whose text mentions a
    /// frontier short name get their definitions committed, so results are
    /// complete even before a background sweep has covered the workspace.
    /// That short-name gate is only candidate discovery; subtype identity is
    /// resolved from the edge index by exact canonical FQCN. Committed files
    /// answer from the index with no parsing at all.
    ///
    /// `include_trait_users` also counts `use Trait;` composition as a
    /// subtype edge (visibility-scoping semantics); leave it off for
    /// goto-implementation semantics (extends/implements only).
    ///
    /// Memoized per `(class_fqn, include_trait_users, files, query
    /// generation)`, same rationale as [`Self::indexed_references_to`].
    pub fn indexed_subtype_classes(
        &self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Result<Vec<SubtypeClassSite>, Cancelled> {
        catch(|| {
            let key = SubtypeQueryCacheKey {
                class_fqn: class_fqn.trim_start_matches('\\').to_ascii_lowercase(),
                include_trait_users,
                generation: self.query_cache_generation(),
                files_hash: hash_files(files),
            };
            if let Some(hit) = self.index.subtype_queries.get(&key) {
                return hit;
            }
            let out = self.subtype_classes_uncached(class_fqn, files, include_trait_users);
            // See `memoize_references` for why a moved generation skips.
            let generation = self.query_cache_generation();
            if generation == key.generation {
                self.index.subtype_queries.insert(generation, key, &out);
            }
            out
        })
    }

    fn subtype_classes_uncached(
        &self,
        class_fqn: &str,
        files: &[Arc<str>],
        include_trait_users: bool,
    ) -> Vec<SubtypeClassSite> {
        let db = &self.db;
        let mut scanned: HashSet<String> = HashSet::default();
        let mut pending: Vec<String> = vec![class_fqn.trim_start_matches('\\').to_string()];
        let mut sites: Vec<crate::db::SubtypeSite> = Vec::new();
        while !pending.is_empty() {
            // Short names only discover stale/uncommitted files worth
            // collecting; the result itself comes from the edge index.
            let needles: Vec<String> = pending
                .drain(..)
                .filter(|f| scanned.insert(f.clone()))
                .map(|f| crate::db::subtype_index::short_name_of(&f).to_string())
                .collect();
            if !needles.is_empty() {
                self.commit_defs_for_matching(files, &needles);
            }
            sites = db.subtype_sites_of(class_fqn, include_trait_users);
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
        // `new class implements X {}` sites are recorded under the resolved
        // canonical parent FQCN during body analysis.
        let root_lc = class_fqn.trim_start_matches('\\').to_ascii_lowercase();
        let scope: HashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        for (file, line, cs, ce) in db.reference_locations(&format!("impl:{root_lc}")) {
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
    #[allow(clippy::type_complexity)]
    pub fn indexed_method_implementations(
        &self,
        class_fqn: &str,
        method: &str,
        files: &[Arc<str>],
    ) -> Result<Vec<(Arc<str>, Arc<str>, crate::Range)>, Cancelled> {
        let subs = self.indexed_subtype_classes(class_fqn, files, false)?;
        catch(|| {
            let db = &self.db;
            let mut out: Vec<(Arc<str>, Arc<str>, crate::Range)> = Vec::new();
            for sub in &subs {
                let here = crate::db::Fqcn::from_str(db, sub.fqcn.as_ref());
                let Some((_, m)) = crate::db::find_method_in_chain(db, here, method) else {
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
            out.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.start.line.cmp(&b.2.start.line)));
            out.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);
            out
        })
    }

    /// Commit definitions (class edges + freshness) for every file in `files`
    /// that is stale (committed against older text) or that has never been
    /// committed and mentions one of `shorts` as a whole identifier.
    ///
    /// The textual gate answers from the shared per-file mention cache — the
    /// same one [`Self::indexed_references_to`]'s gate populates — so a file
    /// scanned by either consumer answers the other with a set lookup. A file
    /// the cache can't answer for is scanned once against the whole name
    /// universe and recorded.
    fn commit_defs_for_matching(&self, files: &[Arc<str>], shorts: &[String]) {
        use rayon::prelude::*;

        let committed_any: HashSet<Arc<str>> =
            self.index.defs_committed_keys().into_iter().collect();
        // Admit the frontier names before preparing, so every needle gets a
        // real query (a declared class's short name is already in the
        // universe from indexing — admission then changes nothing).
        self.db
            .add_literal_mention_names(shorts.iter().map(|s| s.as_str()));
        let queries: Vec<_> = shorts
            .iter()
            .filter_map(|s| self.db.prepare_class_mention_query(s))
            .collect();
        let mention_scanner = self.db.class_mention_scanner();
        // Admission guarantees a query per needle and a non-empty universe;
        // anything else is defensive — the gate then admits every candidate
        // (recommit rather than skip, the conservative direction).
        let use_mentions = queries.len() == shorts.len() && mention_scanner.is_some();
        type Work = (Arc<str>, Arc<str>, Vec<crate::db::SubtypeEntry>);
        let index = &*self.index;
        let results: Vec<(Option<Work>, Option<MentionScanRecord>)> = files
            .par_iter()
            .map_with(self.db.clone(), |db, path| {
                let Some(sf) = db.lookup_source_file(path.as_ref()) else {
                    return (None, None);
                };
                let text = sf.text(&*db as &dyn MirDatabase).clone();
                if index.is_defs_committed(path.as_ref(), &text) {
                    return (None, None);
                }
                // Never-committed files must mention a frontier name; stale
                // (previously committed) files recommit unconditionally —
                // their classes may have re-parented.
                let mut scan_rec: Option<MentionScanRecord> = None;
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
                            // Uncoverable entry: one scan answers every
                            // query and is recorded below.
                            let scanner = mention_scanner.as_ref().unwrap();
                            let names = scanner.scan(&text);
                            let hit = queries.iter().any(|q| names.binary_search(&q.name).is_ok());
                            scan_rec = Some((path.clone(), text.clone(), names));
                            hit
                        }
                    };
                    if !hit {
                        return (None, scan_rec);
                    }
                }
                let defs = crate::db::collect_file_definitions(&*db as &dyn MirDatabase, sf);
                let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
                (Some((path.clone(), text, entries)), scan_rec)
            })
            .collect();
        // Record the fallback scans regardless of hit/miss: each is a
        // complete, current mention set for its file, so the next round's
        // (and the references gate's) checks become set lookups.
        for (work, scan_rec) in results {
            if let (Some(scanner), Some((file, text, names))) = (&mention_scanner, scan_rec) {
                self.db
                    .set_file_class_mentions(&file, &text, scanner.epoch(), names);
            }
            if let Some((file, text, entries)) = work {
                let file_no = self.db.locked_ref_index().intern_path(&file);
                self.db.set_file_class_edges(file_no, entries);
                self.index.mark_defs_committed(&file, &text);
            }
        }
    }

    /// Class-level issues (inheritance violations, abstract-method gaps,
    /// override incompatibilities) for `files`.
    ///
    /// These checks are cross-file by nature and are not emitted by
    /// [`Self::analyze`]. Circular-inheritance checks always run against the
    /// full workspace graph regardless of the `files` filter — a cycle is a
    /// workspace-wide problem.
    pub fn class_issues(&self, files: &[Arc<str>]) -> Result<Vec<crate::Issue>, Cancelled> {
        catch(|| {
            let db = &self.db;
            let file_set: HashSet<Arc<str>> = files.iter().cloned().collect();
            let file_data: Vec<(Arc<str>, Arc<str>)> = files
                .iter()
                .filter_map(|f| {
                    let sf = db.lookup_source_file(f)?;
                    Some((f.clone(), sf.text(db as &dyn MirDatabase).clone()))
                })
                .collect();
            crate::class::ClassAnalyzer::with_files(db, file_set, &file_data).analyze_all()
        })
    }

    /// Collector-phase issues (e.g. `BackedEnumCaseTypeMismatch`,
    /// `InvalidReadonlyPropertyDeclaration`, `InvalidDocblock`, and raw parse
    /// errors) for `files`.
    ///
    /// These are found while building a file's declaration slice, before body
    /// analysis or cross-file class checks run — neither [`Self::analyze`]
    /// nor [`Self::class_issues`] reports them, so a caller merging just those
    /// two sources silently drops every collector-time diagnostic.
    pub fn collector_issues(&self, files: &[Arc<str>]) -> Result<Vec<crate::Issue>, Cancelled> {
        catch(|| {
            files
                .iter()
                .filter_map(|f| self.db.lookup_source_file(f))
                .flat_map(|sf| {
                    crate::db::collect_file_definitions(&self.db, sf)
                        .issues
                        .as_ref()
                        .clone()
                })
                .collect()
        })
    }

    /// Admission predicate for [`Self::stale_reference_candidates`]; see
    /// [`ReferenceGate`] and `reference_gate_needles`.
    ///
    /// Instance/static methods other than `__construct`/`__invoke` on a
    /// resolvable owner gate on the member name alone: every call site
    /// spells it (`->name(`, `::name(`), while the owner's short name would
    /// admit every file mentioning the class for any reason.
    /// `__construct` gates on the owner's short name plus the raw call tokens
    /// `->__construct` / `::__construct`. `__invoke` keeps the general
    /// owner/name gate because `$obj()` call sites do not spell `__invoke`.
    pub(super) fn reference_gate(&self, symbol: &crate::Name) -> ReferenceGate {
        ReferenceGate::for_symbol(&self.db, symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnalysisSession, IndexCancel, Name, ReferenceIncludes};
    use salsa::Database as _;

    const BASE: &str =
        "<?php\nclass Base { public function run(): void {} public function stop(): void {} }\n";
    const CALLS_RUN: &str = "<?php\nfunction go(Base $b): void { $b->run(); }\n";
    const CALLS_STOP: &str = "<?php\nfunction go(Base $b): void { $b->stop(); }\n";

    /// Pins the warm pass's commit landing after an owner text write has
    /// started but before it applies: the commit must not leave the file
    /// fresh for the incoming text.
    #[test]
    fn warm_commit_racing_a_text_write_leaves_the_file_stale() {
        let mut session = AnalysisSession::new(PhpVersion::LATEST);
        let files: Vec<Arc<str>> = vec![Arc::from("base.php"), Arc::from("caller.php")];
        session.ingest_file(files[0].clone(), Arc::from(BASE));
        session.ingest_file(files[1].clone(), Arc::from(CALLS_RUN));
        session.prepare_for_query(Some(&files[1]));

        let snap = session.snapshot();
        let mut pass = snap
            .stage_warm(&files[1..], &IndexCancel::new())
            .unwrap()
            .unwrap();

        let caller = files[1].clone();
        let writer = std::thread::spawn(move || {
            session.upsert_source_file(caller, Arc::from(CALLS_STOP), salsa::Durability::LOW);
            session
        });
        // The write trips cancellation before it waits for `snap` to drop.
        while catch(|| snap.db.unwind_if_revision_cancelled()).is_ok() {
            std::thread::yield_now();
        }
        snap.commit_warm(&mut pass);
        drop(snap);
        let mut session = writer.join().unwrap();

        let callers_of = |session: &mut AnalysisSession, method: &str| -> Vec<Arc<str>> {
            session
                .indexed_references_to(
                    &Name::method("Base", method),
                    &files,
                    false,
                    ReferenceIncludes::Plain,
                    &|| false,
                )
                .unwrap()
                .into_iter()
                .map(|(file, _)| file)
                .collect()
        };
        assert_eq!(callers_of(&mut session, "stop"), [files[1].clone()]);
        assert!(callers_of(&mut session, "run").is_empty());
    }
}
