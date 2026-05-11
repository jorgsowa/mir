//! Session-based analysis API for incremental, per-file analysis.
//!
//! [`AnalysisSession`] owns the salsa database and per-session caches for a
//! long-running analysis context shared across many per-file analyses. Reads
//! clone the database under a brief lock, then run lock-free; writes hold the
//! lock briefly to mutate canonical state. `MirDb::clone()` is cheap
//! (Arc-wrapped registries), so this pattern gives parallel readers without
//! blocking on concurrent writes for longer than the clone itself.
//!
//! See [`crate::file_analyzer::FileAnalyzer`] for the per-file Pass 2 entry
//! point that operates against a session.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::cache::AnalysisCache;
use crate::composer::Psr4Map;
use crate::db::{MirDatabase, MirDb};
use crate::php_version::PhpVersion;
use crate::shared_db::SharedDb;

/// Long-lived analysis context. Owns the salsa database and tracks which
/// stubs have been loaded.
///
/// Cheap to clone the inner db for parallel reads; writes funnel through
/// [`Self::ingest_file`], [`Self::invalidate_file`], and the crate-internal
/// [`Self::with_db_mut`].
pub struct AnalysisSession {
    /// Shared database management (salsa, file registry, stub tracking).
    /// Extracted to allow code sharing with ProjectAnalyzer.
    shared_db: Arc<SharedDb>,
    cache: Option<Arc<AnalysisCache>>,
    psr4: Option<Arc<Psr4Map>>,
    php_version: PhpVersion,
    user_stub_files: Vec<PathBuf>,
    user_stub_dirs: Vec<PathBuf>,
}

impl AnalysisSession {
    /// Create a session targeting the given PHP language version.
    pub fn new(php_version: PhpVersion) -> Self {
        Self {
            shared_db: Arc::new(SharedDb::new()),
            cache: None,
            psr4: None,
            php_version,
            user_stub_files: Vec::new(),
            user_stub_dirs: Vec::new(),
        }
    }

    pub fn with_cache(mut self, cache: Arc<AnalysisCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Convenience: open a disk-backed cache at `cache_dir` and attach it.
    /// Avoids forcing callers to wrap [`AnalysisCache`] in `Arc` themselves.
    pub fn with_cache_dir(self, cache_dir: &std::path::Path) -> Self {
        self.with_cache(Arc::new(AnalysisCache::open(cache_dir)))
    }

    pub fn with_psr4(mut self, map: Arc<Psr4Map>) -> Self {
        self.psr4 = Some(map);
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

    /// Load every PHP built-in stub plus any configured user stubs.
    ///
    /// **Deprecated**: prefer [`Self::ensure_all_stubs_loaded`] (explicit
    /// "comprehensive") or [`Self::ensure_essential_stubs_loaded`] (fast
    /// cold-start with auto-discovery on demand).
    #[doc(hidden)]
    pub fn ensure_stubs_loaded(&self) {
        self.ensure_all_stubs_loaded();
    }

    /// Load only the curated set of essential stubs (Core, standard, SPL,
    /// date) plus any configured user stubs. About 25 of 120 stub files;
    /// covers types and functions used by virtually all PHP code.
    ///
    /// Other extension stubs (Reflection, gd, openssl, …) can be brought in
    /// on demand via [`Self::ensure_stubs_for_symbol`] when user code
    /// references them. Idempotent — already-loaded stubs are skipped.
    pub fn ensure_essential_stubs_loaded(&self) {
        self.shared_db
            .ingest_stub_paths(crate::stubs::ESSENTIAL_STUB_PATHS, self.php_version);
        self.ensure_user_stubs_loaded();
    }

    /// Load every embedded PHP stub plus any configured user stubs.
    /// Use for batch tools (CLI, full project analysis) where comprehensive
    /// symbol coverage matters more than cold-start latency.
    pub fn ensure_all_stubs_loaded(&self) {
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
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
                self.shared_db.ingest_stub_paths(&[path], self.php_version);
                true
            }
            None => false,
        }
    }

    /// Number of distinct embedded stubs currently ingested into the session.
    /// Useful for diagnostics and bench reporting.
    pub fn loaded_stub_count(&self) -> usize {
        self.shared_db.loaded_stubs.lock().len()
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
    /// batch tool called [`Self::ensure_all_stubs_loaded`]), the source scan
    /// is skipped entirely.
    pub fn ensure_stubs_for_source(&self, source: &str) {
        // Cheap check first: skip the scan entirely when we already know we
        // have everything. Avoids a ~50-500µs source walk on every analyze
        // call in batch / warm-session scenarios.
        {
            let loaded = self.shared_db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths(source);
        if paths.is_empty() {
            return;
        }
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
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
    pub fn ensure_stubs_for_ast(&self, program: &php_ast::ast::Program<'_, '_>) {
        {
            let loaded = self.shared_db.loaded_stubs.lock();
            if loaded.len() >= crate::stubs::stub_files().len() {
                return;
            }
        }
        let paths = crate::stubs::collect_referenced_builtin_paths_from_ast(program);
        if paths.is_empty() {
            return;
        }
        self.shared_db.ingest_stub_paths(&paths, self.php_version);
    }

    fn ensure_user_stubs_loaded(&self) {
        self.shared_db
            .ingest_user_stubs(&self.user_stub_files, &self.user_stub_dirs);
    }

    /// Cheap clone of the salsa db for a read-only query. The lock is held
    /// only for the duration of the clone, so concurrent readers never
    /// serialize on each other or on writes for longer than the clone itself.
    ///
    /// **Internal API — exposes Salsa types.** Subject to change without
    /// notice. Public consumers should use the typed query methods
    /// ([`Self::definition_of`], [`Self::hover`], etc.) instead.
    #[doc(hidden)]
    pub fn snapshot_db(&self) -> MirDb {
        self.shared_db.snapshot_db()
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

    /// Pass 1 ingestion. Updates the file's source text in the salsa db,
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
        self.ensure_stubs_loaded();
        {
            let mut guard = self.shared_db.salsa.lock();
            let (ref mut db, _) = *guard;
            db.remove_file_definitions(file.as_ref());
        }
        let _file_defs = self
            .shared_db
            .collect_and_ingest_file(file.clone(), source.as_ref());
        self.update_reverse_deps_for(&file);
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
            let mut guard = self.shared_db.salsa.lock();
            let (ref mut db, ref mut files) = *guard;
            db.remove_file_definitions(file);
            files.remove(file);
        }
        if let Some(cache) = &self.cache {
            cache.update_reverse_deps_for_file(file, &HashSet::new());
            cache.evict_with_dependents(&[file.to_string()]);
        }
    }

    /// Number of files currently tracked in this session's salsa input set.
    /// Stable across reads; useful for diagnostics and memory bounds checks.
    pub fn tracked_file_count(&self) -> usize {
        let guard = self.shared_db.salsa.lock();
        guard.1.len()
    }

    // -----------------------------------------------------------------------
    // Read-only codebase queries
    //
    // All take a brief lock to clone the db, then run the lookup against the
    // owned snapshot — concurrent edits proceed without blocking.
    // -----------------------------------------------------------------------

    /// Resolve a top-level symbol (class or function) to its declaration
    /// location. Powers go-to-definition.
    ///
    /// Returns:
    /// - `Ok(Location)` — symbol found with a source location
    /// - `Err(NotFound)` — no such symbol in the codebase
    /// - `Err(NoSourceLocation)` — symbol exists but has no recorded span
    ///   (e.g. some stub-only declarations)
    pub fn definition_of(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<mir_codebase::storage::Location, crate::SymbolLookupError> {
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Class(fqcn) => {
                let node = db
                    .lookup_class_node(fqcn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                node.location(&db)
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Function(fqn) => {
                let node = db
                    .lookup_function_node(fqn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                node.location(&db)
                    .ok_or(crate::SymbolLookupError::NoSourceLocation)
            }
            crate::Symbol::Method { class, name }
            | crate::Symbol::Property { class, name }
            | crate::Symbol::ClassConstant { class, name } => {
                crate::db::member_location_via_db(&db, class, name)
                    .ok_or(crate::SymbolLookupError::NotFound)
            }
            crate::Symbol::GlobalConstant(_) => {
                // Global constants don't currently store location info
                Err(crate::SymbolLookupError::NoSourceLocation)
            }
        }
    }

    /// Hover information for a symbol: type, docstring, and definition location.
    ///
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor
    /// position, then build a [`crate::Symbol`] from its `kind`. This method
    /// assembles the displayable hover data.
    ///
    /// Returns `Err(NotFound)` if the symbol doesn't exist. May still return
    /// `Ok` with `docstring: None` or `definition: None` if those specific
    /// pieces aren't available.
    pub fn hover(
        &self,
        symbol: &crate::Symbol,
    ) -> Result<crate::HoverInfo, crate::SymbolLookupError> {
        use mir_types::{Atomic, Union};
        let db = self.snapshot_db();
        match symbol {
            crate::Symbol::Function(fqn) => {
                let node = db
                    .lookup_function_node(fqn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = node
                    .return_type(&db)
                    .map(|t| (*t).clone())
                    .unwrap_or_else(Union::mixed);
                let docstring = node.docstring(&db).map(|s| s.to_string());
                let definition = node.location(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition,
                })
            }
            crate::Symbol::Method { class, name } => {
                let node = db
                    .lookup_method_node(class.as_ref(), name.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = node
                    .return_type(&db)
                    .map(|t| (*t).clone())
                    .unwrap_or_else(Union::mixed);
                let docstring = node.docstring(&db).map(|s| s.to_string());
                let definition = node.location(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring,
                    definition,
                })
            }
            crate::Symbol::Class(fqcn) => {
                let node = db
                    .lookup_class_node(fqcn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = Union::single(Atomic::TNamedObject {
                    fqcn: fqcn.clone(),
                    type_params: Vec::new(),
                });
                let definition = node.location(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition,
                })
            }
            crate::Symbol::Property { class, name } => {
                let node = db
                    .lookup_property_node(class.as_ref(), name.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = node.ty(&db).unwrap_or_else(Union::mixed);
                let definition = node.location(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition,
                })
            }
            crate::Symbol::ClassConstant { class, name } => {
                let node = db
                    .lookup_class_constant_node(class.as_ref(), name.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = node.ty(&db);
                let definition = node.location(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition,
                })
            }
            crate::Symbol::GlobalConstant(fqn) => {
                let node = db
                    .lookup_global_constant_node(fqn.as_ref())
                    .filter(|n| n.active(&db))
                    .ok_or(crate::SymbolLookupError::NotFound)?;
                let ty = node.ty(&db);
                Ok(crate::HoverInfo {
                    ty,
                    docstring: None,
                    definition: None,
                })
            }
        }
    }

    /// Every recorded reference to `symbol` with its source location as a Range.
    /// Use [`crate::FileAnalysis::symbol_at`] to find the symbol at a cursor,
    /// build a [`crate::Symbol`] from it, and pass it here.
    pub fn references_to(&self, symbol: &crate::Symbol) -> Vec<(Arc<str>, crate::Range)> {
        let db = self.snapshot_db();
        let key = symbol.codebase_key();
        db.reference_locations(&key)
            .into_iter()
            .map(|(file, line, col_start, col_end)| {
                let range = crate::Range {
                    start: crate::Position {
                        line,
                        column: col_start as u32,
                    },
                    end: crate::Position {
                        line,
                        column: col_end as u32,
                    },
                };
                (file, range)
            })
            .collect()
    }

    /// All declarations defined in `file` as a **hierarchical tree**.
    ///
    /// Classes/interfaces/traits/enums are returned with their methods,
    /// properties, and constants nested in `children`. Top-level functions
    /// and constants are returned with empty `children`.
    pub fn document_symbols(&self, file: &str) -> Vec<crate::symbol::DocumentSymbol> {
        use crate::symbol::{DocumentSymbol, DocumentSymbolKind};

        let db = self.snapshot_db();
        let mut out = Vec::new();
        for symbol in db.symbols_defined_in_file(file) {
            // Try class side first — covers Class / Interface / Trait / Enum.
            if let Some(class_node) = db.lookup_class_node(symbol.as_ref()) {
                if !class_node.active(&db) {
                    continue;
                }
                let (kind, is_enum) = crate::db::class_kind_via_db(&db, symbol.as_ref())
                    .map(|k| {
                        let kind = if k.is_interface {
                            DocumentSymbolKind::Interface
                        } else if k.is_trait {
                            DocumentSymbolKind::Trait
                        } else if k.is_enum {
                            DocumentSymbolKind::Enum
                        } else {
                            DocumentSymbolKind::Class
                        };
                        (kind, k.is_enum)
                    })
                    .unwrap_or((DocumentSymbolKind::Class, false));

                // Build children: methods, properties, and class constants.
                let mut children: Vec<DocumentSymbol> = Vec::new();
                for m in db.class_own_methods(symbol.as_ref()) {
                    if !m.active(&db) {
                        continue;
                    }
                    children.push(DocumentSymbol {
                        name: m.name(&db),
                        kind: DocumentSymbolKind::Method,
                        location: m.location(&db),
                        children: Vec::new(),
                    });
                }
                for p in db.class_own_properties(symbol.as_ref()) {
                    if !p.active(&db) {
                        continue;
                    }
                    children.push(DocumentSymbol {
                        name: p.name(&db),
                        kind: DocumentSymbolKind::Property,
                        location: p.location(&db),
                        children: Vec::new(),
                    });
                }
                for c in db.class_own_constants(symbol.as_ref()) {
                    if !c.active(&db) {
                        continue;
                    }
                    let const_kind = if is_enum {
                        DocumentSymbolKind::EnumCase
                    } else {
                        DocumentSymbolKind::Constant
                    };
                    children.push(DocumentSymbol {
                        name: c.name(&db),
                        kind: const_kind,
                        location: c.location(&db),
                        children: Vec::new(),
                    });
                }

                out.push(DocumentSymbol {
                    name: symbol.clone(),
                    kind,
                    location: class_node.location(&db),
                    children,
                });
                continue;
            }
            if let Some(fn_node) = db.lookup_function_node(symbol.as_ref()) {
                if !fn_node.active(&db) {
                    continue;
                }
                out.push(DocumentSymbol {
                    name: symbol.clone(),
                    kind: DocumentSymbolKind::Function,
                    location: fn_node.location(&db),
                    children: Vec::new(),
                });
                continue;
            }
            // Constants and other top-level declarations: emit with no
            // location info; consumers can still surface them in an outline.
            out.push(DocumentSymbol {
                name: symbol,
                kind: DocumentSymbolKind::Constant,
                location: None,
                children: Vec::new(),
            });
        }
        out
    }

    /// Returns `true` if a function with `fqn` is registered and active in
    /// the codebase. Case-insensitive lookup with optional leading backslash.
    pub fn contains_function(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        db.lookup_function_node(fqn).is_some_and(|n| n.active(&db))
    }

    /// Returns `true` if a class / interface / trait / enum with `fqcn` is
    /// registered and active in the codebase.
    pub fn contains_class(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        db.lookup_class_node(fqcn).is_some_and(|n| n.active(&db))
    }

    /// Returns `true` if `class` has a method named `name` registered. Method
    /// names are matched case-insensitively (PHP method dispatch semantics).
    pub fn contains_method(&self, class: &str, name: &str) -> bool {
        let db = self.snapshot_db();
        let name_lower = name.to_ascii_lowercase();
        db.lookup_method_node(class, &name_lower)
            .is_some_and(|n| n.active(&db))
    }

    /// All class / interface / trait / enum FQCNs currently known to the
    /// session, each paired with the file that defines them when available.
    ///
    /// Use this to build workspace-wide views (outline, fuzzy search, etc.).
    /// Consumers implement their own search/match logic on top — the analyzer
    /// only exposes the iterator.
    pub fn all_classes(&self) -> Vec<(Arc<str>, Option<mir_codebase::storage::Location>)> {
        let db = self.snapshot_db();
        db.active_class_node_fqcns()
            .into_iter()
            .filter_map(|fqcn| {
                let node = db.lookup_class_node(fqcn.as_ref())?;
                if !node.active(&db) {
                    return None;
                }
                Some((fqcn, node.location(&db)))
            })
            .collect()
    }

    /// All global function FQNs currently known to the session, each paired
    /// with their declaration location when available.
    pub fn all_functions(&self) -> Vec<(Arc<str>, Option<mir_codebase::storage::Location>)> {
        let db = self.snapshot_db();
        db.active_function_node_fqns()
            .into_iter()
            .filter_map(|fqn| {
                let node = db.lookup_function_node(fqn.as_ref())?;
                if !node.active(&db) {
                    return None;
                }
                Some((fqn, node.location(&db)))
            })
            .collect()
    }

    /// Compute `file`'s outgoing dependency edges and update the cache's
    /// reverse-dep graph in place. No-op if no cache is configured.
    fn update_reverse_deps_for(&self, file: &str) {
        let Some(cache) = self.cache.as_deref() else {
            return;
        };
        let db = self.snapshot_db();
        let targets = file_outgoing_dependencies(&db, file);
        cache.update_reverse_deps_for_file(file, &targets);
    }

    /// Cross-file inference sweep. For each `(file, source)` pair, runs the
    /// Pass 2 inference-only mode on a cloned db (parallel via rayon), then
    /// commits the collected inferred return types to the canonical db.
    ///
    /// Call this on idle / save / explicit user request, **not** on every
    /// keystroke — [`crate::FileAnalyzer::analyze`] deliberately skips
    /// inference sweep on the hot path. Files whose source contains parse
    /// errors are silently skipped.
    pub fn run_inference_sweep(&self, files: &[(Arc<str>, Arc<str>)]) {
        self.ensure_stubs_loaded();

        // The priming db lives only inside `gather_inferred_types`. After it
        // returns, all rayon-clone references to the salsa storage are dropped
        // — required so that the subsequent `commit_inferred_return_types`
        // call (which calls salsa's `cancel_others`) doesn't deadlock waiting
        // for outstanding db references.
        let (functions, methods) =
            gather_inferred_types(self.snapshot_db(), files, self.php_version);

        let mut guard = self.shared_db.salsa.lock();
        guard.0.commit_inferred_return_types(functions, methods);
    }

    /// File dependency graph: which files depend on which other files.
    /// Used for incremental invalidation in LSP servers and build systems.
    ///
    /// Dependencies are computed from:
    /// - Direct imports (use statements)
    /// - Class inheritance (parent classes, interfaces, traits)
    pub fn dependency_graph(&self) -> crate::DependencyGraph {
        let db = self.snapshot_db();

        // Get all files from the session's salsa database
        let guard = self.shared_db.salsa.lock();
        let all_files: Vec<String> = guard.1.keys().map(|f| f.as_ref().to_string()).collect();
        drop(guard);

        // Build forward dependency graph: file → [files it depends on]
        let mut dependencies: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for file in &all_files {
            let deps = file_outgoing_dependencies(&db, file);
            dependencies.insert(file.clone(), deps.into_iter().collect());
        }

        // Build reverse dependency graph: file → [files that depend on it]
        let mut dependents: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (file, deps) in &dependencies {
            for dep in deps {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(file.clone());
            }
        }

        // Sort for determinism
        for deps in dependents.values_mut() {
            deps.sort();
        }

        crate::DependencyGraph {
            dependencies,
            dependents,
        }
    }
}

/// Drive Pass 2 inference-only mode in parallel across `files`, accumulating
/// inferred function and method return types. The `db_priming` MirDb is
/// consumed (cloned per spawned task and dropped on return), so the caller's
/// canonical db can subsequently take exclusive access without deadlock.
///
/// Crate-internal so [`crate::project::ProjectAnalyzer`] can use the same
/// deadlock-safe helper for its lazy-load reanalysis sweep.
#[allow(clippy::type_complexity)]
pub(crate) fn gather_inferred_types(
    db_priming: MirDb,
    files: &[(Arc<str>, Arc<str>)],
    php_version: PhpVersion,
) -> (
    Vec<(Arc<str>, mir_types::Union)>,
    Vec<(Arc<str>, Arc<str>, mir_types::Union)>,
) {
    use crate::pass2::Pass2Driver;
    use mir_types::Union;

    type Functions = Vec<(Arc<str>, Union)>;
    type Methods = Vec<(Arc<str>, Arc<str>, Union)>;
    let functions: Arc<Mutex<Functions>> = Arc::new(Mutex::new(Vec::new()));
    let methods: Arc<Mutex<Methods>> = Arc::new(Mutex::new(Vec::new()));

    rayon::in_place_scope(|s| {
        for (file, source) in files {
            let db = db_priming.clone();
            let functions = Arc::clone(&functions);
            let methods = Arc::clone(&methods);
            let file = file.clone();
            let source = source.clone();

            s.spawn(move |_| {
                let arena = crate::arena::create_parse_arena(source.len());
                let parsed = php_rs_parser::parse(&arena, source.as_ref());
                if !parsed.errors.is_empty() {
                    return;
                }
                let driver = Pass2Driver::new_inference_only(&db as &dyn MirDatabase, php_version);
                driver.analyze_bodies(&parsed.program, file, source.as_ref(), &parsed.source_map);
                let inferred = driver.take_inferred_types();
                {
                    let mut f = functions.lock();
                    f.extend(inferred.functions);
                }
                {
                    let mut m = methods.lock();
                    m.extend(inferred.methods);
                }
            });
        }
    });

    let functions = Arc::try_unwrap(functions)
        .map(|m| m.into_inner())
        .unwrap_or_else(|arc| arc.lock().clone());
    let methods = Arc::try_unwrap(methods)
        .map(|m| m.into_inner())
        .unwrap_or_else(|arc| arc.lock().clone());

    (functions, methods)
}

/// Compute the set of files `file` depends on: defining files of its imports,
/// plus parent / interfaces / traits' defining files for any classes declared
/// in `file`. Self-edges are excluded.
fn file_outgoing_dependencies(db: &dyn MirDatabase, file: &str) -> HashSet<String> {
    let mut targets: HashSet<String> = HashSet::new();

    let mut add_target = |symbol: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            let def = defining_file.as_ref().to_string();
            if def != file {
                targets.insert(def);
            }
        }
    };

    let imports = db.file_imports(file);
    for fqcn in imports.values() {
        add_target(fqcn);
    }

    for fqcn in db.symbols_defined_in_file(file) {
        let Some(node) = db.lookup_class_node(fqcn.as_ref()) else {
            continue;
        };
        if let Some(parent) = node.parent(db) {
            add_target(parent.as_ref());
        }
        for iface in node.interfaces(db).iter() {
            add_target(iface.as_ref());
        }
        for tr in node.traits(db).iter() {
            add_target(tr.as_ref());
        }
    }

    targets
}
