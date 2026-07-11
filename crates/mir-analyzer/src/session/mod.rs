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
    /// Whether analysis maintains the legacy imperative reference index
    /// ([`crate::db::RefIndex`]). On by default. Hosts that read references
    /// exclusively through the memoized [`Self::references_to_in_files`] path
    /// opt out via [`Self::without_reference_index`], removing every
    /// `RefIndex` lock from their edit and read paths.
    pub(crate) maintain_ref_index: bool,
}

/// FQCN → optional resolver-mapped path. See the field doc on
/// `AnalysisSession::unresolvable_fqcns`.
type UnresolvableCache = Arc<RwLock<HashMap<Arc<str>, Option<Arc<str>>>>>;

/// Warm-up skip set keyed by file path. See the field doc on
/// `AnalysisSession::prepared_files`.
type PreparedFilesCache = Arc<RwLock<HashMap<Arc<str>, (Arc<str>, u64)>>>;

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
            maintain_ref_index: true,
        }
    }

    /// Stop maintaining the legacy imperative reference index on the
    /// incremental (LSP-style) paths: `ingest_file`, `invalidate_file`,
    /// [`crate::FileAnalyzer`] commits, and the `reanalyze_*` sweeps.
    ///
    /// After this, [`Self::references_to`] / [`Self::reference_locations`]
    /// return empty for files analyzed through those paths and
    /// [`Self::dependency_graph`] loses body-level bare-FQN edges — callers
    /// must use the memoized [`Self::references_to_in_files`] /
    /// [`Self::reanalyze_files_cancellable`] paths instead. In exchange, no
    /// edit or read ever takes the `RefIndex` lock (assert via
    /// [`Self::ref_index_lock_count`]) and the index holds no memory.
    /// The batch entry points (`analyze_paths`) still maintain the index.
    pub fn without_reference_index(mut self) -> Self {
        self.maintain_ref_index = false;
        self
    }

    /// Times the reference index has been locked on this session's db.
    pub fn ref_index_lock_count(&self) -> u64 {
        self.db.salsa.read().ref_index_lock_count()
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
        let lookup: &str = match symbol_key.split_once("::") {
            Some((class, _)) => class,
            None => &symbol_key,
        };
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
