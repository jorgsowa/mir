//! Session-based analysis API for incremental, per-file analysis.
//!
//! [`AnalysisSession`] owns the salsa database and per-session caches for a
//! long-running analysis context shared across many per-file analyses. Reads
//! clone the database under a brief lock, then run lock-free; writes hold the
//! lock briefly to mutate canonical state. `MirDbStorage::clone()` is cheap
//! (Arc-wrapped registries), so this pattern gives parallel readers without
//! blocking on concurrent writes for longer than the clone itself.
//!
//! See [`crate::file_analyzer::FileAnalyzer`] for the per-file analysis
//! entry point that operates against a session.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::analyzer_db::AnalyzerDb;
use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{MirDatabase, MirDbStorage, RefLoc};
use crate::php_version::PhpVersion;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
///
/// Cheap to clone the inner db for parallel reads; writes funnel through
/// [`Self::ingest_file`], [`Self::invalidate_file`], and the crate-internal
/// [`Self::with_db_mut`].
#[derive(Clone)]
pub struct AnalysisSession {
    /// Shared database management (salsa, file registry, stub tracking).
    pub(crate) db: Arc<AnalyzerDb>,
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
    stale_defined_symbols: Arc<RwLock<HashMap<String, HashSet<Arc<str>>>>>,
    /// Symbols defined by each file as of its last `ingest_file`. The
    /// authoritative "old" set for the rename/deletion diff, independent of
    /// whether the salsa `SourceFile` input was already updated to the new text
    /// by a host driving the db directly (the LSP convergence path). Without
    /// this, re-deriving "old" symbols from the (possibly pre-updated) input
    /// would miss deletions and break cross-file dependency invalidation.
    last_ingested_symbols: Arc<RwLock<HashMap<String, HashSet<Arc<str>>>>>,
    /// Negative cache: FQCNs that `load_class` already failed on.
    /// The value is the resolver-mapped path (when known) so eviction on
    /// `set_file_text` / `ingest_file` is a path equality check rather than
    /// re-running the resolver per entry. `None` means the resolver itself
    /// couldn't map the FQCN; those entries survive file edits (no source
    /// change makes a never-resolvable name resolvable).
    /// Bounded to `UNRESOLVABLE_CACHE_CAP`; clears on overflow.
    unresolvable_fqcns: UnresolvableCache,
    /// Pluggable source-text provider for lazy-load. Defaults to filesystem
    /// reads ([`crate::FsSourceProvider`]); LSPs swap in a VFS-backed
    /// implementation so unsaved buffers override on-disk content.
    source_provider: Arc<dyn crate::SourceProvider>,
    /// Vendor `autoload.files` entries not yet indexed. `Some(paths)` means
    /// pending; `None` means the load has already run (idempotent). Populated
    /// by [`Self::with_psr4`]; drained by [`Self::ensure_vendor_eager_functions`],
    /// which is called automatically from [`Self::prepare_ast_for_analysis`].
    ///
    /// The mutex is held for the full duration of the load so concurrent callers
    /// block until indexing is complete rather than proceeding with a stale
    /// workspace snapshot.
    pub(crate) pending_eager_function_files: Arc<parking_lot::Mutex<Option<Vec<PathBuf>>>>,
    /// Warm-up skip set: files whose [`Self::prepare_ast_for_analysis`] has
    /// already run against their current text. Value is `(text, generation)` —
    /// the entry is live while the file's input text is pointer-equal to `text`
    /// (a text edit self-invalidates) and `generation` matches
    /// [`Self::prepare_generation`]. Lets the per-request Phase-1 warm-up in
    /// `references_to_in_files` / `reanalyze_dependents` skip the serial
    /// parse + AST walk for files already faulted in.
    prepared_files: PreparedFilesCache,
    /// Bumped whenever previously loaded declarations may have been removed
    /// (`invalidate_file`, symbol deletions on `ingest_file`, or a host calling
    /// [`Self::bump_prepare_generation`]) — a prepared file might then need its
    /// warm-up re-run to lazy-load a replacement (e.g. a vendor class shadowed
    /// by a since-deleted project class).
    prepare_generation: Arc<std::sync::atomic::AtomicU64>,
    /// file → [`RefCommit`] its reference locations were last committed
    /// from. Exact while the text is pointer-equal and the commit either
    /// fully resolved every name it referenced or was stamped at the current
    /// [`Self::index_generation`] — a later symbol add elsewhere can resolve
    /// a reference this file's analysis left unresolved, even though this
    /// file's own text never changed. Files absent here have never been
    /// committed.
    ref_committed: CommittedRefs,
    /// file → source text its subtype-index class edges were last committed
    /// from. Same freshness contract as `ref_committed`, but definitions
    /// depend only on the file's own text, so a pointer-equal entry is
    /// always exact (no cross-file drift).
    defs_committed: CommittedTexts,
}

/// FQCN → optional resolver-mapped path. See the field doc on
/// `AnalysisSession::unresolvable_fqcns`.
type UnresolvableCache = Arc<RwLock<HashMap<Arc<str>, Option<Arc<str>>>>>;

/// Warm-up skip set keyed by file path. See the field doc on
/// `AnalysisSession::prepared_files`.
type PreparedFilesCache = Arc<RwLock<HashMap<Arc<str>, (Arc<str>, u64)>>>;

/// file → text a per-file index commit was computed from. See the field docs
/// on `AnalysisSession::ref_committed` / `defs_committed`.
type CommittedTexts = Arc<RwLock<HashMap<Arc<str>, Arc<str>>>>;

/// A staged [`AnalysisCache`] write for one file's postings, prepared in the
/// parallel analysis phase and applied during the serial index commit. See
/// `AnalysisSession::stage_ref_cache_put`.
pub(crate) struct RefCachePut {
    content_hash: String,
    surface_hash: String,
    ref_locs: Arc<[crate::cache::CachedRefLoc]>,
}

/// One file's reference-posting commit. See `AnalysisSession::ref_committed`.
pub(crate) struct RefCommit {
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
    /// switches remain the reanalyze_dependents flow's job, as before.
    resolved: bool,
}

/// file → [`RefCommit`] map shared across session clones.
type CommittedRefs = Arc<RwLock<HashMap<Arc<str>, RefCommit>>>;

/// Cap on the negative-resolution cache. Sized to accommodate a large
/// workspace's worth of genuinely-missing references without unbounded
/// growth. On overflow the cache is cleared; the cost is a few extra
/// resolver calls until it re-fills.
const UNRESOLVABLE_CACHE_CAP: usize = 10_000;

impl AnalysisSession {
    /// Create a session targeting the given PHP language version.
    pub fn new(php_version: PhpVersion) -> Self {
        let db = Arc::new(AnalyzerDb::new());
        db.salsa
            .write()
            .set_php_version(Arc::from(php_version.to_string().as_str()));
        Self {
            db,
            cache: None,
            psr4: None,
            resolver: None,
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
            stale_defined_symbols: Arc::new(RwLock::new(HashMap::default())),
            last_ingested_symbols: Arc::new(RwLock::new(HashMap::default())),
            unresolvable_fqcns: Arc::new(RwLock::new(HashMap::default())),
            source_provider: Arc::new(crate::FsSourceProvider),
            pending_eager_function_files: Arc::new(parking_lot::Mutex::new(Some(Vec::new()))),
            prepared_files: Arc::new(RwLock::new(HashMap::default())),
            prepare_generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            ref_committed: Arc::new(RwLock::new(HashMap::default())),
            defs_committed: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    /// Times the reference index has been locked on this session's db.
    pub fn ref_index_lock_count(&self) -> u64 {
        self.db.salsa.read().ref_index_lock_count()
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
        };
        self.ref_committed.write().insert(file.clone(), commit);
    }

    pub(crate) fn forget_ref_committed(&self, file: &str) {
        self.ref_committed.write().remove(file);
    }

    /// Stage a disk-cache write for `file`'s postings, computed in the
    /// parallel analysis phase (needs a live db snapshot for the memoized
    /// parse). `None` when no cache is attached or the stored entry already
    /// matches this content — batch-written entries are never clobbered.
    /// The caller applies the result via [`Self::apply_ref_cache_put`] in
    /// its serial commit, alongside the in-memory index commit.
    pub(crate) fn stage_ref_cache_put(
        &self,
        db: &dyn crate::db::MirDatabase,
        sf: crate::db::SourceFile,
        file: &str,
        text: &Arc<str>,
        out: &Arc<crate::db::AnalyzeOutput>,
    ) -> Option<RefCachePut> {
        let cache = self.cache.as_deref()?;
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

    pub(crate) fn apply_ref_cache_put(
        &self,
        file: &str,
        out: &Arc<crate::db::AnalyzeOutput>,
        put: RefCachePut,
    ) {
        if let Some(cache) = self.cache.as_deref() {
            cache.put(
                file,
                put.content_hash,
                put.surface_hash,
                out.issues.clone(),
                put.ref_locs,
            );
        }
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

    /// Every file with a reference commit on record, regardless of
    /// staleness. Files absent here have no reference postings at all.
    pub(crate) fn ref_committed_keys(&self) -> Vec<Arc<str>> {
        self.ref_committed.read().keys().cloned().collect()
    }

    /// Swap in a custom [`crate::SourceProvider`]. LSPs install a VFS-backed
    /// provider here so the analyzer reads from unsaved editor buffers
    /// instead of disk.
    pub fn with_source_provider(mut self, provider: Arc<dyn crate::SourceProvider>) -> Self {
        self.source_provider = provider;
        self
    }

    /// Attach a pre-built [`AnalysisCache`] (the body-analysis issue cache) and
    /// open a sibling definition [`StubSlice`] cache under the same root, so
    /// callers using this builder get the same speedup as `with_cache_dir`.
    ///
    /// Rebuilds the shared database to attach the definition cache — call
    /// **before** any file is ingested. A debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::definitions::StubSlice
    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache must be called before any file is ingested"
        );
        let dir = cache.cache_dir().to_path_buf();
        self.db = Arc::new(AnalyzerDb::new().with_cache_dir(&dir));
        self.db
            .salsa
            .write()
            .set_php_version(Arc::from(self.php_version.to_string().as_str()));
        self.cache = Some(cache);
        self
    }

    /// Convenience: open a disk-backed cache at `cache_dir` and attach it.
    ///
    /// Attaches both the body-analysis issue cache ([`AnalysisCache`]) and the
    /// definition [`StubSlice`] cache to the shared database. Builds a fresh
    /// [`AnalyzerDb`] internally — call **before** any file is ingested. A
    /// debug assertion catches misuse.
    ///
    /// [`StubSlice`]: mir_codebase::definitions::StubSlice
    pub fn with_cache_dir(mut self, cache_dir: &std::path::Path) -> Self {
        debug_assert_eq!(
            self.db.source_file_count(),
            0,
            "AnalysisSession::with_cache_dir must be called before any file is ingested"
        );
        self.db = Arc::new(AnalyzerDb::new().with_cache_dir(cache_dir));
        self.db
            .salsa
            .write()
            .set_php_version(Arc::from(self.php_version.to_string().as_str()));
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
        self.db.salsa.write().set_resolver(Some(resolver));
        // Register vendor autoload.files for lazy loading. They define global
        // functions and constants that the class resolver cannot discover.
        // `ensure_vendor_eager_functions` will index them on first analysis call.
        *self.pending_eager_function_files.lock() = Some(map.vendor_eager_files());
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
        self.db.salsa.write().set_resolver(Some(wrapped.clone()));
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
mod ingest;
mod loading;
mod queries;
mod stubs;

pub use queries::SubtypeClassSite;

/// Compute the full set of files `file` depends on: structural edges from
/// the memoized [`crate::db::file_structural_deps`] tracked query, plus
/// bare-FQN references recorded during body analysis (which live in the
/// reference index and are not visible to salsa). Self-edges are excluded.
/// Used to persist the disk cache's reverse-dep graph.
fn file_outgoing_dependencies(
    db: &dyn MirDatabase,
    file: &str,
    include_body_ref_edges: bool,
) -> HashSet<String> {
    let mut targets: HashSet<String> = HashSet::default();

    if let Some(sf) = db.lookup_source_file(file) {
        for target in crate::db::file_structural_deps(db, sf).iter() {
            targets.insert(target.as_ref().to_string());
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
