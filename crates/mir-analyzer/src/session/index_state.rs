//! Session state shared by the owner and every [`super::AnalysisSnapshot`]:
//! freshness marks for the off-salsa reference/subtype indexes, the query
//! memos layered on those indexes, and the caches their commits invalidate.
//!
//! None of this can live in salsa. The indexes it describes are inverted
//! (symbol → files), which salsa can't maintain without an O(workspace)
//! dependency per query, and they are committed lazily *from* read paths —
//! a salsa input write there would cancel the reader doing it.

use rustc_hash::FxHashMap as HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard, RwLock};

use salsa::Cancelled;

use crate::cache::AnalysisCache;
use crate::db::{MirDatabase, MirDbStorage};

use super::BatchReplayState;

pub(crate) struct IndexState {
    /// file → [`RefCommit`] its reference locations were last committed
    /// from. Exact while the text is pointer-equal and the commit either
    /// fully resolved every name it referenced or was stamped at the current
    /// index generation — a later symbol add elsewhere can resolve a
    /// reference this file's analysis left unresolved, even though this
    /// file's own text never changed. Files absent here have never been
    /// committed.
    ref_committed: RwLock<HashMap<Arc<str>, RefCommit>>,
    /// file → source text its subtype-index class edges were last committed
    /// from. Definitions depend only on the file's own text, so a
    /// pointer-equal entry is always exact (no cross-file drift).
    defs_committed: RwLock<HashMap<Arc<str>, Arc<str>>>,
    /// Memoized `indexed_references_to` results. Without this, a repeat
    /// query against an unchanged candidate set re-pays the O(candidates)
    /// freshness scan on every call — measured at tens of MB / seconds of
    /// churn for a widely-referenced symbol queried repeatedly (a host
    /// recomputing code-lens counts per request).
    pub(crate) ref_queries: QueryMemo<RefQueryCacheKey, (Arc<str>, crate::Range)>,
    /// Memoized `indexed_subtype_classes` results; same rationale — the
    /// defs-commit freshness pass is O(candidates) regardless of outcome.
    pub(crate) subtype_queries: QueryMemo<SubtypeQueryCacheKey, super::SubtypeClassSite>,
    /// Derived dependency graph, rebuilt lazily and dropped whenever
    /// committed references, structural edges, source-file membership, or
    /// stale-symbol tracking changes.
    dependency_graph: RwLock<DependencyGraphCache>,
    /// One-run replay cache for `analyze_paths` when the next batch run sees
    /// the same file set with identical bytes. Any mutation outside
    /// `analyze_paths` clears it.
    transient_batch_replay: RwLock<Option<Arc<BatchReplayState>>>,
    /// Serializes every snapshot-side index commit against
    /// [`Self::retire_references`] / [`Self::retire_file`], so a commit's
    /// check → postings → mark sequence never interleaves with a retire.
    retirements: Mutex<Retirements>,
    /// Mirror of `Retirements::seq` readable without the lock. Part of the
    /// query-memo generation: a retire clears postings without a salsa write.
    retire_seq: AtomicU64,
}

#[derive(Default)]
struct DependencyGraphCache {
    /// Bumped by every invalidation, so a build that raced one is not stored.
    epoch: u64,
    graph: Option<(DependencyGraphStamp, crate::DependencyGraph)>,
}

/// The state a dependency graph was built from, captured before the build.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct DependencyGraphStamp {
    epoch: u64,
    /// Workspace generation: moves on every file add, remove or adoption.
    generation: u64,
    /// Files loaded on demand, which can newly resolve a referenced symbol.
    on_demand_files: usize,
}

/// Owner-side retirements of per-file index state, ordered by a sequence
/// number a snapshot captures as its [`RetireEpoch`].
#[derive(Default)]
struct Retirements {
    seq: u64,
    /// file → `seq` of its latest retirement; one entry per path ever
    /// retired, so bounded by the workspace.
    retired_at: HashMap<Arc<str>, u64>,
}

/// The retirement sequence a snapshot's view reflects. Commits for a file
/// retired after it are refused: the snapshot analyzed text the owner has
/// since replaced or removed.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RetireEpoch(u64);

/// The owner state a snapshot's analyses reflect, stamped on its commits.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewStamp {
    /// Workspace generation, captured before the analysis snapshot.
    pub(crate) generation: u64,
    pub(crate) retire_epoch: RetireEpoch,
}

/// Held while committing one file; see [`IndexState::begin_commit`].
pub(crate) struct CommitGuard<'a>(MutexGuard<'a, Retirements>);

impl CommitGuard<'_> {
    /// `Err` when `file` was retired after `epoch`: the view analyzed text
    /// the owner has since replaced or removed, and an owner write is under
    /// way, so the caller retries like any other cancellation.
    pub(crate) fn admit(&self, file: &str, epoch: RetireEpoch) -> Result<(), Cancelled> {
        match self.0.retired_at.get(file) {
            Some(&at) if at > epoch.0 => Err(Cancelled::PendingWrite),
            _ => Ok(()),
        }
    }
}

impl Default for IndexState {
    fn default() -> Self {
        Self {
            ref_committed: RwLock::default(),
            defs_committed: RwLock::default(),
            ref_queries: QueryMemo::new(REF_QUERY_CACHE_LOCATION_CAP),
            subtype_queries: QueryMemo::new(SUBTYPE_QUERY_CACHE_SITE_CAP),
            dependency_graph: RwLock::default(),
            transient_batch_replay: RwLock::default(),
            retirements: Mutex::default(),
            retire_seq: AtomicU64::new(0),
        }
    }
}

/// One file's reference-posting commit. See `IndexState::ref_committed`.
struct RefCommit {
    /// Source text the postings were computed from (pointer identity; a
    /// text write self-invalidates).
    text: Arc<str>,
    /// Weak handle on the analyze memo — pointer-identical output means
    /// identical postings, so sweeps can skip the index rewrite. The upgrade
    /// guards against ABA on evicted memos.
    out: std::sync::Weak<crate::db::AnalyzeOutput>,
    /// Workspace generation whose resolution environment the postings
    /// reflect, captured *before* the analysis snapshot.
    generation: u64,
    /// The analysis resolved every workspace-level name it referenced, so no
    /// later symbol add can change the postings and the commit survives
    /// generation bumps. FQCN shadowing and unqualified-call fallback
    /// switches remain the reanalyze_dependents flow's job.
    resolved: bool,
    /// Whether this commit came from a live `analyze_file` pass (`out:
    /// Some(..)`) rather than a disk-cache replay (`out: None`, from
    /// `warm_start_files`). A live analysis's postings are guaranteed
    /// consistent with a fresh textual scan of the same text — a replayed
    /// commit's are only as trustworthy as the cache entry, so it always
    /// re-verifies via full re-analysis instead of the cheaper gate.
    live_analyzed: bool,
}

impl IndexState {
    /// The current [`RetireEpoch`]. Captured by the owner when it hands out a
    /// snapshot, while no retire can race it.
    pub(crate) fn retire_epoch(&self) -> RetireEpoch {
        RetireEpoch(self.retirements.lock().seq)
    }

    /// Lock out retirements for the duration of one commit. Callers check
    /// [`CommitGuard::admit`] first, then write postings before marks.
    pub(crate) fn begin_commit(&self) -> CommitGuard<'_> {
        CommitGuard(self.retirements.lock())
    }

    /// Drop `file`'s reference postings and their freshness mark as one step
    /// and refuse later commits from snapshots that predate it. The mark goes
    /// first: a concurrent reader may see postings without a mark (re-verified
    /// through the gate) but never a mark over cleared postings.
    pub(crate) fn retire_references(&self, db: &MirDbStorage, file: &str) {
        let mut retirements = self.retirements.lock();
        self.record_retirement(&mut retirements, file);
        self.forget_ref_committed(file);
        db.clear_file_references(file);
    }

    /// [`Self::retire_references`] plus the file's subtype-index class edges.
    pub(crate) fn retire_file(&self, db: &MirDbStorage, file: &str) {
        let mut retirements = self.retirements.lock();
        self.record_retirement(&mut retirements, file);
        self.forget_ref_committed(file);
        db.clear_file_references(file);
        self.forget_defs_committed(file);
        db.clear_file_class_edges(file);
    }

    /// Bumps the seq before the caller clears anything, so a query that
    /// reads the cleared postings sees its generation move.
    fn record_retirement(&self, retirements: &mut Retirements, file: &str) {
        retirements.seq += 1;
        let seq = retirements.seq;
        retirements.retired_at.insert(Arc::from(file), seq);
        self.retire_seq.store(seq, Ordering::SeqCst);
    }

    pub(crate) fn retire_seq(&self) -> u64 {
        self.retire_seq.load(Ordering::SeqCst)
    }

    /// Whether `file`'s reference postings are exact for `current_text` at
    /// `current_gen`: text pointer-equal, and the commit either resolved
    /// every name (immune to workspace growth) or was stamped at that
    /// generation — catches a file analyzed before a class it references
    /// was registered elsewhere, which would otherwise look fresh forever.
    pub(crate) fn is_ref_committed(
        &self,
        file: &str,
        current_text: &Arc<str>,
        current_gen: u64,
    ) -> bool {
        self.ref_committed.read().get(file).is_some_and(|c| {
            Arc::ptr_eq(&c.text, current_text) && (c.resolved || c.generation == current_gen)
        })
    }

    /// Whether `file` has a *live-analyzed* reference commit recorded against
    /// exactly `current_text` — i.e. it's only stale by generation, not
    /// because its text changed or because it was seeded by an unverified
    /// disk-cache replay. Such a commit is still eligible for the
    /// mention/needle gate: the current text is exactly what a live analysis
    /// already scanned, so a needle miss is as conclusive as for a
    /// never-committed file.
    pub(crate) fn ref_commit_stale_by_generation_only(
        &self,
        file: &str,
        current_text: &Arc<str>,
    ) -> bool {
        self.ref_committed
            .read()
            .get(file)
            .is_some_and(|c| c.live_analyzed && Arc::ptr_eq(&c.text, current_text))
    }

    /// Whether `file`'s stored postings came from exactly this
    /// (text, output) pair — generation aside. Pointer-identical output
    /// means identical postings (salsa backdates equal results to the same
    /// Arc), so callers skip the index rewrite and only re-stamp the mark.
    pub(crate) fn ref_commit_is_current(
        &self,
        file: &str,
        current_text: &Arc<str>,
        out: &Arc<crate::db::AnalyzeOutput>,
    ) -> bool {
        self.ref_committed.read().get(file).is_some_and(|c| {
            Arc::ptr_eq(&c.text, current_text)
                && c.out.upgrade().is_some_and(|prev| Arc::ptr_eq(&prev, out))
        })
    }

    /// Record a commit computed against the workspace state at `generation`
    /// — captured by the caller *before* its analysis snapshot, so a file
    /// add racing the analysis leaves the commit stale (re-verified on the
    /// next query) rather than wrongly fresh. `resolved` must come from the
    /// producing analysis' own issue set
    /// ([`crate::db::issues_have_unresolved_names`]); pass `false` when
    /// unknown — the gen-guarded safe direction.
    pub(crate) fn mark_ref_committed(
        &self,
        file: &Arc<str>,
        text: &Arc<str>,
        out: Option<&Arc<crate::db::AnalyzeOutput>>,
        generation: u64,
        resolved: bool,
    ) {
        let commit = RefCommit {
            text: text.clone(),
            out: out.map(Arc::downgrade).unwrap_or_default(),
            generation,
            resolved,
            live_analyzed: out.is_some(),
        };
        self.ref_committed.write().insert(file.clone(), commit);
    }

    pub(crate) fn forget_ref_committed(&self, file: &str) {
        self.ref_committed.write().remove(file);
    }

    /// Every file with a reference commit on record, regardless of
    /// staleness. Files absent here have no reference postings at all.
    pub(crate) fn ref_committed_keys(&self) -> Vec<Arc<str>> {
        self.ref_committed.read().keys().cloned().collect()
    }

    /// Whether `file`'s subtype-index class edges were committed from exactly
    /// `current_text`.
    pub(crate) fn is_defs_committed(&self, file: &str, current_text: &Arc<str>) -> bool {
        self.defs_committed
            .read()
            .get(file)
            .is_some_and(|t| Arc::ptr_eq(t, current_text))
    }

    pub(crate) fn mark_defs_committed(&self, file: &Arc<str>, text: &Arc<str>) {
        self.defs_committed
            .write()
            .insert(file.clone(), text.clone());
    }

    pub(crate) fn forget_defs_committed(&self, file: &str) {
        self.defs_committed.write().remove(file);
    }

    /// Every file with a defs commit on record, regardless of staleness.
    pub(crate) fn defs_committed_keys(&self) -> Vec<Arc<str>> {
        self.defs_committed.read().keys().cloned().collect()
    }

    /// Capture before reading anything a dependency graph is built from.
    pub(crate) fn dependency_graph_stamp(&self, db: &MirDbStorage) -> DependencyGraphStamp {
        DependencyGraphStamp {
            epoch: self.dependency_graph.read().epoch,
            generation: db.workspace_revision_value(),
            on_demand_files: db.on_demand_file_count(),
        }
    }

    pub(crate) fn cached_dependency_graph(
        &self,
        stamp: DependencyGraphStamp,
    ) -> Option<crate::DependencyGraph> {
        match &self.dependency_graph.read().graph {
            Some((built_at, graph)) if *built_at == stamp => Some(graph.clone()),
            _ => None,
        }
    }

    /// Cache `graph`, built from the state at `stamp`, unless an
    /// invalidation landed since.
    pub(crate) fn store_dependency_graph(
        &self,
        stamp: DependencyGraphStamp,
        graph: crate::DependencyGraph,
    ) {
        let mut cache = self.dependency_graph.write();
        if cache.epoch == stamp.epoch {
            cache.graph = Some((stamp, graph));
        }
    }

    pub(crate) fn clear_dependency_graph_cache(&self) {
        let mut cache = self.dependency_graph.write();
        cache.epoch += 1;
        cache.graph = None;
    }

    pub(crate) fn transient_batch_replay(&self) -> Option<Arc<BatchReplayState>> {
        self.transient_batch_replay.read().clone()
    }

    pub(crate) fn store_transient_batch_replay(&self, state: BatchReplayState) {
        *self.transient_batch_replay.write() = Some(Arc::new(state));
    }

    pub(crate) fn clear_transient_batch_replay(&self) {
        *self.transient_batch_replay.write() = None;
    }

    /// Replace `file`'s postings with a complete reference set produced
    /// outside the memoized `analyze_file` query (the open-file analysis
    /// path) by the view at `stamp`. No memo to record, so the empty weak
    /// handle makes the next sweep recommit once — the safe direction.
    pub(crate) fn commit_file_refs(
        &self,
        db: &MirDbStorage,
        file: &Arc<str>,
        text: Option<Arc<str>>,
        locs: Vec<crate::db::RefLoc>,
        resolved: bool,
        stamp: ViewStamp,
    ) -> Result<(), Cancelled> {
        let commit = self.begin_commit();
        commit.admit(file, stamp.retire_epoch)?;
        self.clear_transient_batch_replay();
        let file_no = db.locked_ref_index().intern_path(file);
        db.set_file_reference_locations(file_no, locs);
        self.clear_dependency_graph_cache();
        if let Some(text) = text {
            self.mark_ref_committed(file, &text, None, stamp.generation, resolved);
        }
        Ok(())
    }

    /// Analyze `path` via the memoized `analyze_file` query and stage
    /// everything [`Self::commit_analyzed`] writes for it. Pure: safe on a
    /// parallel pass snapshot.
    pub(crate) fn stage_analyzed(
        &self,
        db: &MirDbStorage,
        cache: Option<&AnalysisCache>,
        mention_scanner: Option<&crate::db::class_mention_index::MentionScanner>,
        path: &Arc<str>,
    ) -> Option<AnalyzedFile> {
        let sf = db.lookup_source_file(path.as_ref())?;
        let text = sf.text(db as &dyn MirDatabase).clone();
        let out = crate::db::analyze_file(db as &dyn MirDatabase, sf).clone();
        let defs = crate::db::collect_file_definitions(db as &dyn MirDatabase, sf);
        let entries = crate::db::subtype_index::entries_from_slice(&defs.slice);
        // Stage the disk-cache write only when the commit will rewrite
        // postings — a no-op re-sweep adds no hashing or parse-walk cost.
        let cache_put = if self.ref_commit_is_current(path.as_ref(), &text, &out) {
            None
        } else {
            cache.and_then(|cache| stage_ref_cache_put(cache, db, sf, path.as_ref(), &text, &out))
        };
        // Mention scan piggybacks on the analysis pass, skipped when the file
        // already holds a current scan.
        let mentions = mention_scanner.and_then(|s| {
            (!db.class_mentions_current(path.as_ref(), &text, s.epoch())).then(|| s.scan(&text))
        });
        Some(AnalyzedFile {
            file: path.clone(),
            text,
            out,
            entries,
            cache_put,
            mentions,
        })
    }

    /// Serial commit of staged analyses into both inverted indexes and their
    /// freshness marks. Each output is the file's complete reference set, so
    /// postings are replaced, not appended; unchanged files (same text, same
    /// memo) skip the rewrite and only re-stamp the mark.
    ///
    /// `Err` when a file was retired after `stamp`; every other file is still
    /// committed.
    pub(crate) fn commit_analyzed(
        &self,
        db: &MirDbStorage,
        cache: Option<&AnalysisCache>,
        mention_scanner: Option<&crate::db::class_mention_index::MentionScanner>,
        analyzed: &mut [AnalyzedFile],
        stamp: ViewStamp,
    ) -> Result<(), Cancelled> {
        let mut dependency_graph_changed = false;
        let mut refused = Ok(());
        for a in analyzed.iter_mut() {
            let commit = self.begin_commit();
            if let Err(cancelled) = commit.admit(&a.file, stamp.retire_epoch) {
                refused = Err(cancelled);
                continue;
            }
            if !self.ref_commit_is_current(a.file.as_ref(), &a.text, &a.out) {
                let file_no = db.locked_ref_index().intern_path(&a.file);
                db.set_file_reference_locations(file_no, a.out.ref_locs.to_vec());
                dependency_graph_changed = true;
            }
            if let (Some(s), Some(m)) = (mention_scanner, a.mentions.take()) {
                db.set_file_class_mentions(&a.file, &a.text, s.epoch(), m);
            }
            if let (Some(cache), Some(put)) = (cache, a.cache_put.take()) {
                cache.put(
                    a.file.as_ref(),
                    put.content_hash,
                    put.surface_hash,
                    a.out.issues.clone(),
                    put.ref_locs,
                );
            }
            self.mark_ref_committed(
                &a.file,
                &a.text,
                Some(&a.out),
                stamp.generation,
                !a.out.has_unresolved_names(),
            );
            if !self.is_defs_committed(a.file.as_ref(), &a.text) {
                let file_no = db.locked_ref_index().intern_path(&a.file);
                db.set_file_class_edges(file_no, a.entries.clone());
                self.mark_defs_committed(&a.file, &a.text);
                dependency_graph_changed = true;
            }
            drop(commit);
        }
        if dependency_graph_changed {
            self.clear_dependency_graph_cache();
        }
        refused
    }

    /// Commit class edges a view at `epoch` computed from `text`; `Err`
    /// when the file was retired since.
    pub(crate) fn commit_class_edges(
        &self,
        db: &MirDbStorage,
        file: &Arc<str>,
        text: &Arc<str>,
        entries: Vec<crate::db::SubtypeEntry>,
        epoch: RetireEpoch,
    ) -> Result<(), Cancelled> {
        let commit = self.begin_commit();
        commit.admit(file, epoch)?;
        let file_no = db.locked_ref_index().intern_path(file);
        db.set_file_class_edges(file_no, entries);
        self.mark_defs_committed(file, text);
        Ok(())
    }
}

/// One file's analysis output staged by [`IndexState::stage_analyzed`] for
/// [`IndexState::commit_analyzed`]. `text` is the exact Arc analyzed: the
/// freshness marks record it, so a text write racing the pass leaves the
/// file dirty rather than wrongly fresh.
pub(crate) struct AnalyzedFile {
    pub(crate) file: Arc<str>,
    text: Arc<str>,
    pub(crate) out: Arc<crate::db::AnalyzeOutput>,
    entries: Vec<crate::db::SubtypeEntry>,
    cache_put: Option<RefCachePut>,
    mentions: Option<Box<[mir_types::Name]>>,
}

/// A staged [`AnalysisCache`] write for one file's postings, prepared in the
/// parallel analysis phase and applied during the serial commit.
struct RefCachePut {
    content_hash: String,
    surface_hash: String,
    ref_locs: Arc<[crate::cache::CachedRefLoc]>,
}

/// `None` when the stored entry already matches this content — batch-written
/// entries are never clobbered.
fn stage_ref_cache_put(
    cache: &AnalysisCache,
    db: &dyn MirDatabase,
    sf: crate::db::SourceFile,
    file: &str,
    text: &Arc<str>,
    out: &Arc<crate::db::AnalyzeOutput>,
) -> Option<RefCachePut> {
    let content_hash = crate::cache::hash_content(text);
    if cache.is_valid(file, &content_hash) {
        return None;
    }
    let parsed = crate::db::parse_file(db, sf);
    let surface_hash = crate::cache::surface_fingerprint(text, &parsed.0.program);
    let ref_locs: Arc<[crate::cache::CachedRefLoc]> = out
        .ref_locs
        .iter()
        .map(|r| (Arc::clone(&r.symbol_key), r.line, r.col_start, r.col_end))
        .collect();
    Some(RefCachePut {
        content_hash,
        surface_hash,
        ref_locs,
    })
}

/// `(text revision, subtype-edge epoch, retirement seq)`; every component is
/// monotonic.
pub(crate) type QueryGeneration = (salsa::Revision, u64, u64);

/// A memo of per-query result lists keyed at a query generation — see
/// [`super::AnalysisSnapshot::query_cache_generation`]. Bounded by the total
/// number of cached *items* across entries, not entry count: one entry's
/// `Vec` scales with how often its symbol occurs (a handful for a typical
/// method, thousands for a hot one), so an entry cap gives no byte ceiling.
/// Cleared wholesale on overflow.
pub(crate) struct QueryMemo<K, T> {
    map: RwLock<RevisionedMap<K, Arc<Vec<T>>>>,
    /// Sum of `.len()` across every entry, kept in lockstep with `map`.
    items: AtomicUsize,
    /// Hits served. Diagnostic only — lets tests assert a warm repeat
    /// skipped the freshness scan entirely.
    hits: AtomicU64,
    cap: usize,
}

impl<K: std::hash::Hash + Eq, T: Clone> QueryMemo<K, T> {
    fn new(cap: usize) -> Self {
        Self {
            map: RwLock::default(),
            items: AtomicUsize::new(0),
            hits: AtomicU64::new(0),
            cap,
        }
    }

    pub(crate) fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    pub(crate) fn items(&self) -> usize {
        self.items.load(Ordering::Relaxed)
    }

    pub(crate) fn get(&self, key: &K) -> Option<Vec<T>> {
        let map = self.map.read();
        let hit = map.get(key)?;
        self.hits.fetch_add(1, Ordering::Relaxed);
        Some((**hit).clone())
    }

    /// Cache `result` under `key`, computed at `generation`. The caller must
    /// only insert when the generation didn't move while computing — such a
    /// key can never be looked up again.
    pub(crate) fn insert(&self, generation: QueryGeneration, key: K, result: &[T]) {
        let mut map = self.map.write();
        if !map.advance_to(generation, || self.items.store(0, Ordering::Relaxed)) {
            return;
        }
        let new_len = result.len();
        let prior = self.items.fetch_add(new_len, Ordering::Relaxed);
        if prior + new_len > self.cap {
            map.map.clear();
            self.items.store(new_len, Ordering::Relaxed);
        }
        map.map.insert(key, Arc::new(result.to_vec()));
    }
}

/// A memo map whose keys embed the generation they were computed at. Once
/// the generation moves, old keys can never be looked up again, so the first
/// insert at a newer generation drops them wholesale — without this, dead
/// keys (heap `String`s) accumulate until the overflow clear.
struct RevisionedMap<K, V> {
    generation: Option<QueryGeneration>,
    map: HashMap<K, V>,
}

impl<K, V> Default for RevisionedMap<K, V> {
    fn default() -> Self {
        Self {
            generation: None,
            map: HashMap::default(),
        }
    }
}

impl<K: std::hash::Hash + Eq, V> RevisionedMap<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Prepare for an insert keyed at `generation`. Rolls the map forward
    /// (dropping the dead generation) when `generation` is newer, running
    /// `on_clear` so the caller can zero its lockstep size counter. Returns
    /// `false` when `generation` is older — the caller should skip caching.
    /// Every component is monotonic, so lexicographic order is sound.
    fn advance_to(&mut self, generation: QueryGeneration, on_clear: impl FnOnce()) -> bool {
        match self.generation {
            Some(g) if g == generation => true,
            Some(g) if g > generation => false,
            _ => {
                self.map.clear();
                self.generation = Some(generation);
                on_clear();
                true
            }
        }
    }
}

/// Cache key for `indexed_references_to`'s memoization.
///
/// `generation`'s revision half is salsa's text revision, not the index
/// generation: a body-only edit never bumps the workspace generation but can
/// still move, add, or remove a reference location. Its epoch half covers a
/// member query's hierarchy fan-out, which reads the subtype index — another
/// query's defs commit can grow it without moving the text revision.
/// `files_hash` guards against a caller narrowing/widening the candidate
/// scope between calls at the same revision.
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct RefQueryCacheKey {
    pub(crate) symbol: String,
    pub(crate) include_declaration: bool,
    pub(crate) includes: super::ReferenceIncludes,
    pub(crate) generation: QueryGeneration,
    pub(crate) files_hash: u64,
}

/// ~24 bytes per `(Arc<str>, Range)` location plus a refcount bump on an
/// already-allocated path, so a few MB regardless of query skew.
const REF_QUERY_CACHE_LOCATION_CAP: usize = 200_000;

/// Cache key for `indexed_subtype_classes`'s memoization. Same contract as
/// [`RefQueryCacheKey`]. `class_fqn` is lowercased and leading-`\`-stripped
/// (PHP class names are case-insensitive) so differently-spelled callers
/// share an entry.
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct SubtypeQueryCacheKey {
    pub(crate) class_fqn: String,
    pub(crate) include_trait_users: bool,
    pub(crate) generation: QueryGeneration,
    pub(crate) files_hash: u64,
}

/// A `SubtypeClassSite` is ~80 bytes, so a few MB.
const SUBTYPE_QUERY_CACHE_SITE_CAP: usize = 50_000;

/// Stable content hash of a candidate-file list, order-sensitive. A
/// coincidental collision would only cause a wrong cache HIT, so this hashes
/// full content, not just length/pointers.
pub(crate) fn hash_files(files: &[Arc<str>]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    files.len().hash(&mut hasher);
    for f in files {
        f.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_graph() -> crate::DependencyGraph {
        crate::DependencyGraph::from_compact_parts(
            Vec::new(),
            HashMap::default(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn a_dependency_graph_built_across_an_invalidation_is_not_cached() {
        let state = IndexState::default();
        let db = MirDbStorage::default();

        let stamp = state.dependency_graph_stamp(&db);
        state.store_dependency_graph(stamp, empty_graph());
        assert!(state.cached_dependency_graph(stamp).is_some());

        let stamp = state.dependency_graph_stamp(&db);
        state.clear_dependency_graph_cache();
        state.store_dependency_graph(stamp, empty_graph());
        let current = state.dependency_graph_stamp(&db);
        assert!(state.cached_dependency_graph(current).is_none());
    }
}
