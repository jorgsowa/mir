//! Session-based analysis API for incremental, per-file analysis.
//!
//! [`AnalysisSession`] owns the salsa database and per-session caches. It is
//! the single writer (mutations take `&mut self`); readers on other threads
//! use [`AnalysisSnapshot`]s from [`AnalysisSession::snapshot`].
//!
//! See [`crate::file_analyzer::FileAnalyzer`] for the per-file analysis
//! entry point that operates against a session.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzer_db::AnalyzerDb;
use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{MirDatabase, MirDbStorage, RefLoc};
use crate::php_version::PhpVersion;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
pub struct AnalysisSession {
    /// Database management (salsa, file registry, stub tracking).
    pub(crate) db: AnalyzerDb,
    pub(crate) cache: Option<Arc<AnalysisCache>>,
    /// PSR-4 / Composer autoload map. Retained alongside `resolver` so the
    /// `psr4()` accessor can still return a typed `Psr4Map` for callers that
    /// need Composer-specific data (project_files / vendor_files / etc.).
    pub(crate) psr4: Option<Arc<Psr4Map>>,
    /// Generic class resolver used for on-demand lazy loading. When `psr4`
    /// is set via [`Self::with_psr4`], this is populated with the same map
    /// re-typed as `dyn ClassResolver`. Consumers can also supply their own
    /// resolver via [`Self::with_class_resolver`] without going through
    /// Composer.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
    pub(crate) php_version: PhpVersion,
    pub(crate) user_stub_files: Vec<PathBuf>,
    pub(crate) user_stub_dirs: Vec<PathBuf>,
    /// Tracks symbols that were previously defined in a file but have since
    /// been removed (deleted or renamed). When `ingest_file` detects that
    /// a symbol disappears, it records it here so `dependency_graph()` can
    /// still produce edges to files that reference the now-gone symbol.
    ///
    /// Keyed by the file that used to define the symbols. Symbols are removed
    /// from the set when re-added to the same file on a subsequent ingest.
    /// The set may contain symbols with no current referencers; those are
    /// harmless — the `symbol_referencers_of` lookup returns empty.
    stale_defined_symbols: HashMap<String, HashSet<Arc<str>>>,
    /// Symbols defined by each file as of its last `ingest_file`. The
    /// authoritative "old" set for the rename/deletion diff, independent of
    /// whether the salsa `SourceFile` input was already updated to the new text
    /// by a host driving the db directly (the LSP convergence path). Without
    /// this, re-deriving "old" symbols from the (possibly pre-updated) input
    /// would miss deletions and break cross-file dependency invalidation.
    last_ingested_symbols: HashMap<String, HashSet<Arc<str>>>,
    /// Structural outgoing dependency targets by file, as of the last
    /// ingestion that updated this session's declaration state. Lets
    /// `ingest_file` tell whether the dependency graph's declaration-shaped
    /// edges actually changed, so body-only edits need not invalidate the
    /// cached graph.
    last_structural_targets: HashMap<String, HashSet<String>>,
    /// Negative cache: FQCNs that `load_class` already failed on.
    /// The value is the resolver-mapped path (when known) so eviction on
    /// `set_file_text` / `ingest_file` is a path equality check rather than
    /// re-running the resolver per entry. `None` means the resolver itself
    /// couldn't map the FQCN; those entries survive file edits (no source
    /// change makes a never-resolvable name resolvable).
    /// Bounded to `UNRESOLVABLE_CACHE_CAP`; clears on overflow.
    unresolvable_fqcns: HashMap<Arc<str>, Option<Arc<str>>>,
    /// Vendor `autoload.files` entries not yet indexed. `Some(paths)` means
    /// pending; `None` means the load has already run (idempotent). Populated
    /// by [`Self::with_psr4`]; drained by [`Self::ensure_vendor_eager_functions`],
    /// which is called automatically from [`Self::prepare_ast_for_analysis`].
    pub(crate) pending_eager_function_files: Option<Vec<PathBuf>>,
    /// Warm-up skip set: files whose [`Self::prepare_ast_for_analysis`] has
    /// already run against their current text. Value is `(text, generation)` —
    /// the entry is live while the file's input text is pointer-equal to `text`
    /// (a text edit self-invalidates) and `generation` matches
    /// [`Self::prepare_generation`]. Lets the per-request Phase-1 warm-up in
    /// `indexed_references_to` / `reanalyze_dependents` skip the serial
    /// parse + AST walk for files already faulted in.
    prepared_files: PreparedFilesCache,
    /// Parsed suppression directives for unchanged files. Batch analysis
    /// revisits every analyzed file to emit `UnusedSuppress`, so retaining
    /// maps across same-session runs avoids rescanning the full workspace.
    pub(crate) suppression_maps: SuppressionMapCache,
    /// Bumped whenever previously loaded declarations may have been removed
    /// (`invalidate_file`, symbol deletions on `ingest_file`, or a host calling
    /// [`Self::bump_prepare_generation`]) — a prepared file might then need its
    /// warm-up re-run to lazy-load a replacement (e.g. a vendor class shadowed
    /// by a since-deleted project class).
    prepare_generation: u64,
    /// Index freshness marks and query memos, shared with every
    /// [`AnalysisSnapshot`].
    pub(crate) index: Arc<IndexState>,
}

/// Which reference postings [`AnalysisSession::indexed_references_to`]
/// should read back from the maintained reference index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceIncludes {
    /// Plain usage/declaration references only (`cls:` / `fn:` / member
    /// postings), excluding `use` import items.
    Plain,
    /// Only `use` import items (`use Foo\Bar;`, `use function ...;`,
    /// `use const ...;`), excluding plain usage/declaration references.
    UseImports,
    /// Plain references plus `use` import items.
    PlainAndUseImports,
}

/// file → `(text, prepare generation)`. See `AnalysisSession::prepared_files`.
type PreparedFilesCache = HashMap<Arc<str>, (Arc<str>, u64)>;

/// Parsed inline suppressions keyed by file path and source text.
type SuppressionMapCache = HashMap<Arc<str>, (Arc<str>, Arc<crate::suppression::SuppressionMap>)>;

/// One file's replayable body-analysis output from the previous batch run.
pub(crate) struct BatchReplayFile {
    pub(crate) issues: Arc<[mir_issues::Issue]>,
    pub(crate) ref_locs: Arc<[crate::cache::CachedRefLoc]>,
    pub(crate) symbols: Arc<[crate::symbol::ResolvedSymbol]>,
}

/// Fingerprint for a session-local whole-batch replay.
///
/// The replay path is intentionally narrow: exact path→content-hash equality,
/// same PHP version, and same symbol-collection mode. This keeps it suitable
/// for watch-mode warm repeats without widening it into a general incremental
/// cache.
#[derive(PartialEq, Eq)]
pub(crate) struct BatchReplayKey {
    php_version: u8,
    skip_symbols: bool,
    content_hashes: HashMap<Arc<str>, String>,
}

pub(crate) struct BatchReplayState {
    key: BatchReplayKey,
    pub(crate) files: HashMap<Arc<str>, BatchReplayFile>,
}

/// Cap on the negative-resolution cache. Sized to accommodate a large
/// workspace's worth of genuinely-missing references without unbounded
/// growth. On overflow the cache is cleared; the cost is a few extra
/// resolver calls until it re-fills.
const UNRESOLVABLE_CACHE_CAP: usize = 10_000;

/// RAII scope from [`AnalysisSession::defer_revision_bumps`]. Derefs to the
/// session, so the scope body mutates it through the guard.
pub(crate) struct DeferredRevisionBumps<'a> {
    session: &'a mut AnalysisSession,
}

impl std::ops::Deref for DeferredRevisionBumps<'_> {
    type Target = AnalysisSession;
    fn deref(&self) -> &AnalysisSession {
        self.session
    }
}

impl std::ops::DerefMut for DeferredRevisionBumps<'_> {
    fn deref_mut(&mut self) -> &mut AnalysisSession {
        self.session
    }
}

impl Drop for DeferredRevisionBumps<'_> {
    fn drop(&mut self) {
        self.session.db.salsa.resume_revision_bumps();
    }
}

impl AnalysisSession {
    /// Create a session targeting the given PHP language version.
    pub fn new(php_version: PhpVersion) -> Self {
        let mut db = AnalyzerDb::new();
        db.salsa.set_php_version(Arc::from(php_version.to_string()));
        db.salsa
            .set_source_provider(Arc::new(crate::FsSourceProvider));
        Self {
            db,
            cache: None,
            psr4: None,
            resolver: None,
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
            stale_defined_symbols: HashMap::default(),
            last_ingested_symbols: HashMap::default(),
            last_structural_targets: HashMap::default(),
            unresolvable_fqcns: HashMap::default(),
            pending_eager_function_files: Some(Vec::new()),
            prepared_files: HashMap::default(),
            suppression_maps: HashMap::default(),
            prepare_generation: 0,
            index: Arc::default(),
        }
    }

    /// A `Send + Clone` read handle on the current revision. See
    /// [`AnalysisSnapshot`] for the lifetime rules.
    pub fn snapshot(&self) -> AnalysisSnapshot {
        AnalysisSnapshot {
            db: self.db.snapshot_db(),
            index: Arc::clone(&self.index),
            cache: self.cache.clone(),
            php_version: self.php_version,
            index_generation: self.index_generation(),
            retire_epoch: self.index.retire_epoch(),
        }
    }

    /// A [`Self::snapshot`] scoped to this borrow of the session; see
    /// [`DbView`].
    pub(crate) fn db_view(&self) -> DbView<'_> {
        DbView::new(self, self.snapshot())
    }

    /// Run a snapshot query on the owner's thread. The owner can't write
    /// while borrowed, so nothing can cancel the query: a `Cancelled` is
    /// re-raised, never retried.
    pub(crate) fn query_snapshot<T>(
        &self,
        query: impl FnOnce(&AnalysisSnapshot) -> Result<T, salsa::Cancelled>,
    ) -> T {
        match query(&self.db_view()) {
            Ok(out) => out,
            Err(cancelled) => snapshot::unwind(cancelled),
        }
    }

    /// Hits served from [`Self::indexed_references_to`]'s memoization cache.
    /// Diagnostic only — lets tests assert a warm repeat skipped the
    /// freshness scan entirely.
    pub fn ref_query_cache_hits(&self) -> u64 {
        self.index.ref_queries.hits()
    }

    /// Total cached reference-location count across the references memo.
    /// Diagnostic only — lets tests/hosts assert the cache stays under its
    /// byte-proportional cap rather than trusting entry count alone.
    pub fn ref_query_cache_locations(&self) -> usize {
        self.index.ref_queries.items()
    }

    /// Hits served from [`Self::indexed_subtype_classes`]'s memoization
    /// cache. Diagnostic only.
    pub fn subtype_query_cache_hits(&self) -> u64 {
        self.index.subtype_queries.hits()
    }

    /// Total cached site count across the subtype memo. Diagnostic only.
    pub fn subtype_query_cache_sites(&self) -> usize {
        self.index.subtype_queries.items()
    }

    /// Times the reference index has been locked on this session's db.
    pub fn ref_index_lock_count(&self) -> u64 {
        self.db.salsa.ref_index_lock_count()
    }

    pub(crate) fn transient_batch_replay(
        &self,
        php_version: u8,
        skip_symbols: bool,
        content_hashes: &HashMap<Arc<str>, String>,
    ) -> Option<Arc<BatchReplayState>> {
        let state = self.index.transient_batch_replay()?;
        (state.key.php_version == php_version
            && state.key.skip_symbols == skip_symbols
            && state.key.content_hashes == *content_hashes)
            .then_some(state)
    }

    pub(crate) fn store_transient_batch_replay(
        &self,
        php_version: u8,
        skip_symbols: bool,
        content_hashes: &HashMap<Arc<str>, String>,
        files: HashMap<Arc<str>, BatchReplayFile>,
    ) {
        self.index.store_transient_batch_replay(BatchReplayState {
            key: BatchReplayKey {
                php_version,
                skip_symbols,
                content_hashes: content_hashes.clone(),
            },
            files,
        });
    }

    /// Defer workspace revision bumps until the returned scope closes; see
    /// [`MirDbStorage::defer_revision_bumps`].
    pub(crate) fn defer_revision_bumps(&mut self) -> DeferredRevisionBumps<'_> {
        self.db.salsa.defer_revision_bumps();
        DeferredRevisionBumps { session: self }
    }

    /// Coverage/size counters for the class-mention gate index (host
    /// metrics and memory-bound checks).
    pub fn class_mention_stats(&self) -> crate::db::ClassMentionStats {
        self.db.salsa.class_mention_stats()
    }

    /// See [`AnalysisSnapshot::files_mentioning_class`].
    pub fn files_mentioning_class(&self, files: &[Arc<str>], class_name: &str) -> Vec<Arc<str>> {
        self.files_mentioning_any(files, &[class_name])
    }

    /// See [`AnalysisSnapshot::files_mentioning_any`].
    pub fn files_mentioning_any(&self, files: &[Arc<str>], needles: &[&str]) -> Vec<Arc<str>> {
        self.query_snapshot(|snap| snap.files_mentioning_any(files, needles))
    }

    /// Persist the attached [`AnalysisCache`] to disk. No-op without an
    /// attached cache or when nothing changed since the last flush.
    /// Reference postings committed by session sweeps and on-demand query
    /// freshness passes reach disk only here — a host should call this after
    /// its warm sweep completes and on shutdown so the next launch's
    /// [`Self::warm_start_files`] finds them.
    pub fn flush_analysis_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.flush();
        }
    }

    /// Whether the workspace symbol index singleton is populated (seeded by
    /// [`Self::warm_start_files`] or built by `index_batch`) — symbol lookups
    /// answer from the O(1) map instead of the tracked O(all-files) walk.
    pub fn workspace_symbol_index_ready(&self) -> bool {
        self.db.salsa.workspace_symbol_index_singleton().is_some()
    }

    /// Executions of the tracked O(all-files) `workspace_symbol_index` walk
    /// (diagnostic; a warm-started session should keep this at zero).
    pub fn workspace_index_walks(&self) -> u64 {
        self.db.salsa.workspace_index_walks()
    }

    /// Swap in a custom [`crate::SourceProvider`]. LSPs install a VFS-backed
    /// provider here so the analyzer reads from unsaved editor buffers
    /// instead of disk.
    pub fn with_source_provider(mut self, provider: Arc<dyn crate::SourceProvider>) -> Self {
        self.db.salsa.set_source_provider(provider);
        self
    }

    /// Attach a pre-built [`AnalysisCache`] (the body-analysis issue cache) and
    /// open a sibling definition [`StubSlice`] cache under the same root, so
    /// callers using this builder get the same speedup as `with_cache_dir`.
    ///
    /// Call **before** any file is ingested. A debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::definitions::StubSlice
    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache must be called before any file is ingested"
        );
        self.db.attach_cache_dir(cache.cache_dir());
        self.cache = Some(cache);
        self
    }

    /// Convenience: open a disk-backed cache at `cache_dir` and attach it.
    ///
    /// Attaches both the body-analysis issue cache ([`AnalysisCache`]) and the
    /// definition [`StubSlice`] cache to the shared database. Call **before**
    /// any file is ingested. A debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::definitions::StubSlice
    pub fn with_cache_dir(mut self, cache_dir: &std::path::Path) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache_dir must be called before any file is ingested"
        );
        self.db.attach_cache_dir(cache_dir);
        // Fold the user-stub fingerprint into the cache epoch. `with_user_stubs`
        // must run before this for it to be picked up (it does in `build_session`);
        // sessions without user stubs get 0, which is correct.
        let user_stub_fp =
            crate::stubs::user_stub_fingerprint(&self.user_stub_files, &self.user_stub_dirs);
        self.cache = Some(Arc::new(AnalysisCache::open(
            cache_dir,
            self.php_version.cache_byte(),
            user_stub_fp,
        )));
        self
    }

    /// Attach a Composer autoload map (PSR-4, PSR-0, classmap, files).
    /// Sets the same map as the active [`crate::ClassResolver`] so
    /// [`Self::load_class`] works out of the box.
    pub fn with_psr4(mut self, map: Arc<Psr4Map>) -> Self {
        let user_resolver: Arc<dyn crate::ClassResolver> = map.clone();
        // Wrap with stub awareness so `find_class_like` / `resolve_fqcn_to_path`
        // can map built-in PHP class FQCNs (`ArrayObject`, `Exception`, …)
        // to their stub virtual paths.
        let resolver: Arc<dyn crate::ClassResolver> = Arc::new(crate::ChainedClassResolver::new(
            user_resolver,
            Arc::new(crate::StubClassResolver),
        ));
        self.psr4 = Some(map.clone());
        self.resolver = Some(resolver.clone());
        // Mirror into MirDbStorage so salsa-tracked resolver queries
        // (`db::resolve_fqcn_to_path`) see the same resolver and are
        // invalidated on swap.
        self.db.salsa.set_resolver(Some(resolver));
        // Register vendor autoload.files for lazy loading. They define global
        // functions and constants that the class resolver cannot discover.
        // `ensure_vendor_eager_functions` will index them on first analysis call.
        self.pending_eager_function_files = Some(map.vendor_eager_files());
        self
    }

    /// Attach a generic class resolver for projects that don't use Composer
    /// (WordPress, Drupal, custom autoloaders, workspace-walk indexes).
    /// Replaces any previously-set Composer-backed resolver. Automatically
    /// wrapped with stub awareness so PHP built-ins remain resolvable.
    pub fn with_class_resolver(mut self, resolver: Arc<dyn crate::ClassResolver>) -> Self {
        let wrapped: Arc<dyn crate::ClassResolver> = Arc::new(crate::ChainedClassResolver::new(
            resolver,
            Arc::new(crate::StubClassResolver),
        ));
        self.db.salsa.set_resolver(Some(wrapped.clone()));
        self.resolver = Some(wrapped);
        self
    }

    pub fn with_user_stubs(mut self, files: Vec<PathBuf>, dirs: Vec<PathBuf>) -> Self {
        self.user_stub_files = files;
        self.user_stub_dirs = dirs;
        self
    }

    pub fn php_version(&self) -> PhpVersion {
        self.php_version
    }

    pub fn cache(&self) -> Option<&AnalysisCache> {
        self.cache.as_deref()
    }

    pub fn psr4(&self) -> Option<&Psr4Map> {
        self.psr4.as_deref()
    }
}

mod incremental;
mod index_state;
mod ingest;
mod loading;
mod queries;
mod snapshot;
mod stubs;

use index_state::IndexState;
pub use queries::SubtypeClassSite;
pub use snapshot::AnalysisSnapshot;
use snapshot::DbView;

/// Compute the full set of files `file` depends on by projecting structural
/// and body-reference symbols through the workspace symbol index. Self-edges
/// are excluded. Used to persist the disk cache's reverse-dep graph.
fn file_outgoing_dependencies(
    db: &dyn MirDatabase,
    file: &str,
    include_body_ref_edges: bool,
) -> HashSet<String> {
    let mut targets: HashSet<String> = HashSet::default();

    if let Some(sf) = db.lookup_source_file(file) {
        for symbol in crate::db::file_structural_symbols(db, sf).iter() {
            let lookup = crate::defining_file_lookup_key(symbol);
            if let Some(defining_file) = db.symbol_defining_file(lookup) {
                if defining_file.as_ref() != file {
                    targets.insert(defining_file.as_ref().to_string());
                }
            }
        }
    }

    if !include_body_ref_edges {
        return targets;
    }

    // Bare-FQN references recorded during body analysis (new \Foo(),
    // \Foo::method(), \foo()) that do not appear in use-import statements.
    for symbol_key in db.file_referenced_symbols(file) {
        let lookup = crate::defining_file_lookup_key(&symbol_key);
        if let Some(defining_file) = db.symbol_defining_file(lookup) {
            if defining_file.as_ref() != file {
                targets.insert(defining_file.as_ref().to_string());
            }
        }
    }

    targets
}

/// AST visitor that collects class FQCN references for PSR-4 preloading.
/// Captures identifiers from `new X`, static calls / property / constant
/// access, type hints, `instanceof`, and `@param`/`@return`/`@var`/`@extends`/
/// `@implements` docblock annotations. Does *not* normalize via PSR-4 /
/// imports — callers run the raw string through `resolve_name`.
fn collect_class_refs_from_ast(program: &php_ast::owned::Program) -> Vec<String> {
    use php_ast::ast::BinaryOp;
    use php_ast::owned::visitor::{
        walk_owned_class_member, walk_owned_expr, walk_owned_program, walk_owned_stmt, OwnedVisitor,
    };
    use php_ast::owned::{ClassMemberKind, ExprKind};
    use std::ops::ControlFlow;

    fn owned_name_str(name: &php_ast::owned::Name) -> String {
        let joined: String = name
            .parts
            .iter()
            .map(|p| p.as_ref())
            .collect::<Vec<&str>>()
            .join("\\");
        if name.kind == php_ast::ast::NameKind::FullyQualified {
            format!("\\{joined}")
        } else {
            joined
        }
    }

    /// Recursively collect all `TNamedObject` FQCNs from a mir type, including
    /// those nested inside generic type parameters (e.g. `Collection<Item>`).
    fn collect_from_type(ty: &mir_types::Type, out: &mut std::collections::HashSet<String>) {
        for atomic in ty.types.iter() {
            if let mir_types::Atomic::TNamedObject { fqcn, type_params } = atomic {
                out.insert(fqcn.as_ref().to_string());
                for tp in type_params.iter() {
                    collect_from_type(tp, out);
                }
            }
        }
    }

    /// Parse a docblock and collect class names from `@param`, `@return`,
    /// `@var`, `@extends`, and `@implements` annotations.
    fn collect_from_docblock(text: &str, out: &mut std::collections::HashSet<String>) {
        let parsed = crate::parser::DocblockParser::parse(text);
        for (_, ty) in &parsed.params {
            collect_from_type(ty, out);
        }
        if let Some(ret) = &parsed.return_type {
            collect_from_type(ret, out);
        }
        if let Some(var) = &parsed.var_type {
            collect_from_type(var, out);
        }
        for ext in &parsed.extends {
            collect_from_type(ext, out);
        }
        for impl_ty in &parsed.implements {
            collect_from_type(impl_ty, out);
        }
    }

    struct V {
        names: std::collections::HashSet<String>,
    }
    impl OwnedVisitor for V {
        fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> ControlFlow<()> {
            if let Some(doc) = stmt.leading_doc_comment() {
                collect_from_docblock(&doc.text, &mut self.names);
            }
            walk_owned_stmt(self, stmt)
        }

        fn visit_class_member(&mut self, member: &php_ast::owned::ClassMember) -> ControlFlow<()> {
            match &member.kind {
                ClassMemberKind::Method(m) => {
                    if let Some(doc) = &m.doc_comment {
                        collect_from_docblock(&doc.text, &mut self.names);
                    }
                }
                ClassMemberKind::Property(p) => {
                    if let Some(doc) = &p.doc_comment {
                        collect_from_docblock(&doc.text, &mut self.names);
                    }
                }
                _ => {}
            }
            walk_owned_class_member(self, member)
        }

        fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> ControlFlow<()> {
            match &expr.kind {
                ExprKind::New(n) => {
                    if let ExprKind::Identifier(name) = &n.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::StaticMethodCall(c) => {
                    if let ExprKind::Identifier(name) = &c.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::StaticPropertyAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::ClassConstAccess(a) => {
                    if let ExprKind::Identifier(name) = &a.class.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
                    if let ExprKind::Identifier(name) = &b.right.kind {
                        self.names.insert(name.as_ref().to_string());
                    }
                }
                _ => {}
            }
            walk_owned_expr(self, expr)
        }

        // Walker routes every class/type-position Name here: type hints, catch types, extends/implements, trait use, attributes.
        fn visit_name(&mut self, name: &php_ast::owned::Name) -> ControlFlow<()> {
            let s = owned_name_str(name);
            if !s.is_empty() {
                self.names.insert(s);
            }
            ControlFlow::Continue(())
        }
    }
    let mut v = V {
        names: std::collections::HashSet::default(),
    };
    let _ = walk_owned_program(&mut v, program);
    v.names.into_iter().collect()
}
