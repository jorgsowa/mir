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
    /// PSR-4 / Composer autoload map. Retained alongside `resolver` so the
    /// `psr4()` accessor can still return a typed `Psr4Map` for callers that
    /// need Composer-specific data (project_files / vendor_files / etc.).
    psr4: Option<Arc<Psr4Map>>,
    /// Generic class resolver used for on-demand lazy loading. When `psr4`
    /// is set via [`Self::with_psr4`], this is populated with the same map
    /// re-typed as `dyn ClassResolver`. Consumers can also supply their own
    /// resolver via [`Self::with_class_resolver`] without going through
    /// Composer.
    resolver: Option<Arc<dyn crate::ClassResolver>>,
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
            resolver: None,
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

    /// Attach a Composer autoload map (PSR-4, PSR-0, classmap, files).
    /// Sets the same map as the active [`crate::ClassResolver`] so
    /// [`Self::lazy_load_class`] works out of the box.
    pub fn with_psr4(mut self, map: Arc<Psr4Map>) -> Self {
        let resolver: Arc<dyn crate::ClassResolver> = map.clone();
        self.psr4 = Some(map);
        self.resolver = Some(resolver);
        self
    }

    /// Attach a generic class resolver for projects that don't use Composer
    /// (WordPress, Drupal, custom autoloaders, workspace-walk indexes).
    /// Replaces any previously-set Composer-backed resolver.
    pub fn with_class_resolver(mut self, resolver: Arc<dyn crate::ClassResolver>) -> Self {
        self.resolver = Some(resolver);
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
            guard.remove_file_definitions(file.as_ref());
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
            guard.remove_file_definitions(file);
            guard.remove_source_file(file);
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
        guard.source_file_count()
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

    /// Class-level issues (inheritance violations, abstract-method gaps, override
    /// incompatibilities) for the given set of files.
    ///
    /// These checks are cross-file by nature and are not emitted by
    /// [`crate::FileAnalyzer::analyze`]. Call this after ingesting or
    /// re-analyzing a file and its dependents to get the full diagnostic picture.
    ///
    /// Circular-inheritance checks always run against the full workspace graph
    /// regardless of the `files` filter — a cycle is a workspace-wide problem.
    pub fn class_issues_for(&self, files: &[Arc<str>]) -> Vec<crate::Issue> {
        let db = self.snapshot_db();
        let file_set: HashSet<Arc<str>> = files.iter().cloned().collect();
        let file_data: Vec<(Arc<str>, Arc<str>)> = files
            .iter()
            .filter_map(|f| Some((f.clone(), self.source_of(f)?)))
            .collect();
        crate::class::ClassAnalyzer::with_files(&db, file_set, &file_data).analyze_all()
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

    /// Try to resolve `fqcn` via PSR-4 and ingest the mapped file, returning
    /// a detailed outcome distinguishing "already there" from "freshly loaded".
    pub fn lazy_load_class_with_outcome(&self, fqcn: &str) -> crate::LazyLoadOutcome {
        if self.contains_class(fqcn) {
            return crate::LazyLoadOutcome::AlreadyLoaded;
        }
        if self.lazy_load_class(fqcn) {
            crate::LazyLoadOutcome::Loaded
        } else {
            crate::LazyLoadOutcome::NotResolvable
        }
    }

    /// Try to resolve `fqcn` via the configured [`crate::ClassResolver`] and
    /// ingest the mapped file.
    ///
    /// This is the LSP-friendly lazy-load entry point: the analyzer never
    /// touches `vendor/` on its own, but consumers can ask it to resolve
    /// individual symbols on demand. Designed to be called when a diagnostic
    /// would otherwise report `UndefinedClass`.
    ///
    /// Returns `true` if either the class is already known or a matching
    /// file was found and successfully ingested. Returns `false` if:
    /// - No resolver is configured (neither `with_psr4` nor `with_class_resolver` called),
    /// - The resolver can't map `fqcn` to a file,
    /// - The file can't be read, or
    /// - The file parsed but did not define `fqcn`.
    pub fn lazy_load_class(&self, fqcn: &str) -> bool {
        if self.contains_class(fqcn) {
            return true;
        }
        let Some(resolver) = &self.resolver else {
            return false;
        };
        let Some(path) = resolver.resolve(fqcn) else {
            return false;
        };
        let Ok(src) = std::fs::read_to_string(&path) else {
            return false;
        };
        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        self.ingest_file(file, Arc::from(src));
        self.contains_class(fqcn)
    }

    /// Lazy-load every class transitively reachable from `fqcn` via parent /
    /// interface / trait edges. Useful when the consumer needs not just the
    /// requested class but enough of its inheritance chain to type-check
    /// member access.
    ///
    /// Walks at most `max_depth` levels (default in batch analysis is 10).
    /// Returns the number of classes successfully loaded (not counting
    /// `fqcn` itself if it was already present).
    pub fn lazy_load_class_transitive(&self, fqcn: &str, max_depth: usize) -> usize {
        if self.resolver.is_none() {
            return 0;
        }
        let mut loaded = 0;
        let mut frontier: Vec<String> = vec![fqcn.to_string()];
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

        for _ in 0..max_depth {
            if frontier.is_empty() {
                break;
            }
            let mut next: Vec<String> = Vec::new();
            for name in frontier.drain(..) {
                if !visited.insert(name.clone()) {
                    continue;
                }
                let was_present = self.contains_class(&name);
                let resolved = self.lazy_load_class(&name);
                if resolved && !was_present {
                    loaded += 1;
                    // Walk the new class's parent / interfaces / traits.
                    let db = self.snapshot_db();
                    if let Some(node) = db.lookup_class_node(&name) {
                        if let Some(parent) = node.parent(&db) {
                            next.push(parent.to_string());
                        }
                        for iface in node.interfaces(&db).iter() {
                            next.push(iface.to_string());
                        }
                        for tr in node.traits(&db).iter() {
                            next.push(tr.to_string());
                        }
                        for ext in node.extends(&db).iter() {
                            next.push(ext.to_string());
                        }
                    }
                }
            }
            frontier = next;
        }
        loaded
    }

    /// Retrieve the source text the session has registered for `file`, if
    /// any. Returns `None` when the file has never been ingested. Used by
    /// the parallel re-analysis path to re-feed dependents to Pass 2 without
    /// the caller having to track sources independently.
    pub fn source_of(&self, file: &str) -> Option<Arc<str>> {
        let guard = self.shared_db.salsa.lock();
        let sf = guard.lookup_source_file(file)?;
        Some(sf.text(&*guard))
    }

    /// Re-analyze every transitive dependent of `file` in parallel.
    ///
    /// When the user saves a file that other files depend on (e.g. editing
    /// a base class, an interface, or a trait), those dependents may have
    /// new diagnostics. This method computes them in parallel using rayon
    /// and returns the per-file analysis results so the LSP server can
    /// publish updated diagnostics in one batch.
    ///
    /// Source text for dependents is retrieved from the session's salsa
    /// inputs (set by previous `ingest_file` calls) — the caller doesn't
    /// need to track or re-read files. Files for which the session has no
    /// source are silently skipped (returns the analyzable subset).
    ///
    /// Does not run inference sweeps. For full-fidelity cross-file inferred
    /// return types, follow up with [`Self::run_inference_sweep`] over the
    /// affected file set.
    pub fn analyze_dependents_of(&self, file: &str) -> Vec<(Arc<str>, crate::FileAnalysis)> {
        use rayon::prelude::*;

        // Phase 1: compute dependents + gather their sources outside the
        // analysis loop so each worker has everything it needs.
        let dependents = self.dependency_graph().transitive_dependents(file);
        if dependents.is_empty() {
            return Vec::new();
        }
        let with_source: Vec<(Arc<str>, Arc<str>)> = dependents
            .into_iter()
            .filter_map(|path| {
                let arc_path: Arc<str> = Arc::from(path.as_str());
                let src = self.source_of(&path)?;
                Some((arc_path, src))
            })
            .collect();
        if with_source.is_empty() {
            return Vec::new();
        }

        // Phase 2: parallel parse + analyze. Each rayon worker gets its own
        // database snapshot via FileAnalyzer; writes are isolated to the
        // session's canonical db (none happen here since we only run Pass 2).
        with_source
            .into_par_iter()
            .map(|(file, source)| {
                let arena = crate::arena::create_parse_arena(source.len());
                let parsed = php_rs_parser::parse(&arena, source.as_ref());
                let analyzer = crate::FileAnalyzer::new(self);
                let analysis = analyzer.analyze(
                    file.clone(),
                    source.as_ref(),
                    &parsed.program,
                    &parsed.source_map,
                );
                (file, analysis)
            })
            .collect()
    }

    /// FQCNs that `file` imports via `use` statements but that aren't yet
    /// loaded in the session.
    ///
    /// Designed as the input to background prefetching: after the LSP server
    /// ingests an open buffer, it can call this and lazy-load the returned
    /// FQCNs on a worker thread so the user's first Cmd+Click into vendor
    /// code doesn't pay the file-read+parse cost.
    ///
    /// Returns an empty Vec if the file hasn't been ingested or has no
    /// unresolved imports.
    pub fn pending_lazy_loads(&self, file: &str) -> Vec<Arc<str>> {
        let db = self.snapshot_db();
        let imports = db.file_imports(file);
        if imports.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for fqcn in imports.values() {
            // Cheap check: skip imports already in the codebase.
            if db.lookup_class_node(fqcn).is_some_and(|n| n.active(&db)) {
                continue;
            }
            // Only worth queueing if the resolver could in principle find it.
            if let Some(resolver) = &self.resolver {
                if resolver.resolve(fqcn).is_some() {
                    out.push(Arc::from(fqcn.as_str()));
                }
            }
        }
        out
    }

    /// Convenience: synchronously lazy-load every import of `file` that
    /// isn't already in the codebase. Returns the number successfully loaded.
    ///
    /// For non-blocking prefetch, call this from a worker thread:
    ///
    /// ```ignore
    /// let s = session.clone();  // AnalysisSession is wrapped in Arc by callers
    /// std::thread::spawn(move || {
    ///     s.prefetch_imports(&file_path);
    /// });
    /// ```
    ///
    /// Internally walks the inheritance chain of each loaded class to a
    /// shallow depth so member access on imported types type-checks without
    /// the user paying the cost on their first navigation.
    pub fn prefetch_imports(&self, file: &str) -> usize {
        let pending = self.pending_lazy_loads(file);
        let mut loaded = 0;
        for fqcn in pending {
            // Use the transitive walker with a small depth so we pick up
            // parent classes / interfaces needed for member resolution, but
            // don't recursively pull in the entire vendor tree.
            loaded += self.lazy_load_class_transitive(&fqcn, 2);
        }
        loaded
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

    /// Cross-file inference sweep. For each `(file, source)` pair, calls the
    /// Salsa-tracked `infer_file_return_types` query in parallel, then commits
    /// the collected inferred return types to INPUT fields.
    ///
    /// Files must already be ingested via [`Self::ingest_file`] before calling
    /// this method. Subsequent [`FileAnalyzer::analyze`] calls read the committed
    /// INPUT fields via O(1) lookups with no lock contention.
    pub fn run_inference_sweep(&self, files: &[(Arc<str>, Arc<str>)]) {
        use rayon::prelude::*;
        let db_priming = self.snapshot_db();
        let inferred_results: Vec<crate::db::InferredFileTypes> = files
            .par_iter()
            .map_with(db_priming, |db, (path, _src)| {
                if let Some(sf) = db.lookup_source_file(path) {
                    crate::db::infer_file_return_types(db, sf)
                } else {
                    crate::db::InferredFileTypes::empty()
                }
            })
            .collect();
        let mut functions = Vec::new();
        let mut methods = Vec::new();
        for result in inferred_results {
            for (fqn, ty) in result.functions.iter() {
                functions.push((fqn.clone(), (**ty).clone()));
            }
            for ((fqcn, name), ty) in result.methods.iter() {
                methods.push((fqcn.clone(), name.clone(), (**ty).clone()));
            }
        }
        let mut guard = self.shared_db.salsa.lock();
        guard.commit_inferred_return_types(functions, methods);
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
        let all_files: Vec<String> = guard
            .source_file_paths()
            .iter()
            .map(|f| f.as_ref().to_string())
            .collect();
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

    let extract_named_objects = |union: &mir_types::Union| {
        union
            .types
            .iter()
            .filter_map(|atomic| match atomic {
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(fqcn.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
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

        // Add types from properties
        for prop in db.class_own_properties(fqcn.as_ref()).iter() {
            if let Some(ty) = prop.ty(db) {
                for named in extract_named_objects(&ty) {
                    add_target(named.as_ref());
                }
            }
        }

        // Add types from methods
        for method in db.class_own_methods(fqcn.as_ref()).iter() {
            // Parameter types
            for param in method.params(db).iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_target(named.as_ref());
                    }
                }
            }
            // Return type
            if let Some(rt) = method.return_type(db) {
                for named in extract_named_objects(rt.as_ref()) {
                    add_target(named.as_ref());
                }
            }
        }
    }

    // Add types from global functions
    for fqn in db.active_function_node_fqns() {
        let Some(node) = db.lookup_function_node(fqn.as_ref()) else {
            continue;
        };
        if let Some(file_of_fn) = db.symbol_defining_file(fqn.as_ref()) {
            if file_of_fn.as_ref() != file {
                continue;
            }
        } else {
            continue;
        }

        // Parameter types
        for param in node.params(db).iter() {
            if let Some(ty) = &param.ty {
                for named in extract_named_objects(ty.as_ref()) {
                    add_target(named.as_ref());
                }
            }
        }
        // Return type
        if let Some(rt) = node.return_type(db) {
            for named in extract_named_objects(rt.as_ref()) {
                add_target(named.as_ref());
            }
        }
    }

    // Also track bare-FQN references recorded during Pass 2 (new \Foo(), \Foo::method(),
    // \foo()) that do not appear in use-import statements.
    for (symbol_key, _, _, _) in db.extract_file_reference_locations(file) {
        let lookup: &str = match symbol_key.split_once("::") {
            Some((class, _)) => class,
            None => &symbol_key,
        };
        add_target(lookup);
    }

    targets
}
