use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_types::{Name, Type};

/// `(file_path, alias→FQCN map)` pair returned by
/// [`MirDatabase::file_import_snapshots`]. The map is `Arc`-shared with the
/// file's `StubSlice` so producing a snapshot is O(1) per file.
pub type FileImportSnapshot = (Arc<str>, Arc<FxHashMap<Name, Name>>);

/// Pass-scoped cache for `extends_or_implements`, keyed by the FxHash of the
/// `(child, ancestor)` `&str` pair so lookups need no interning. The stored
/// `(child, ancestor, result)` lets the lookup verify the strings against the
/// key — a hash collision degrades to a recompute, never a wrong subtype
/// answer. Shared across body-pass workers via `Arc` (DashMap is sharded for
/// concurrent access); created and dropped per frozen pass, so it cannot
/// outlive the immutable-graph window that makes it sound.
pub type SubtypeCache = dashmap::DashMap<(u64, u64), (Box<str>, Box<str>, bool)>;

// MirDatabase trait

/// Salsa database trait for mir incremental analysis.
#[salsa::db]
pub trait MirDatabase: salsa::Database {
    /// The PHP version configured for this analysis run.
    fn php_version_str(&self) -> Arc<str>;

    /// Return this file's first declared namespace, if any.
    fn file_namespace(&self, file: &str) -> Option<Arc<str>>;

    /// Return this file's `use` alias map.
    ///
    /// Cheap to call: returns a cloned `Arc` of the underlying map stored
    /// inside the file's `StubSlice`, not a deep clone of the entries. body-analysis
    /// `resolve_name` calls this on every symbol reference.
    fn file_imports(&self, file: &str) -> Arc<FxHashMap<Name, Name>>;

    /// Return this file's `use` alias map restricted to class/interface/trait/enum
    /// imports (`UseKind::Normal`), excluding `use function`/`use const` aliases.
    ///
    /// Class-name resolution (`resolve_name`) must consult this instead of
    /// `file_imports` so a `use function Foo\bar;` can't make an unrelated `bar`
    /// class/type-hint reference resolve to `Foo\bar`.
    fn file_class_imports(&self, file: &str) -> Arc<FxHashMap<Name, Name>>;

    /// Return the known type for a PHP global variable.
    fn global_var_type(&self, name: &str) -> Option<Type>;

    /// Return `(file, imports)` snapshots for every known file.
    fn file_import_snapshots(&self) -> Vec<FileImportSnapshot>;

    /// Return the defining file for a symbol, if known.
    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>>;

    /// Return all files that reference `symbol_key`.
    /// O(1) via the `symbol_referencers` reverse index; valid even after
    /// the symbol has been removed from its defining file.
    fn symbol_referencers_of(&self, symbol_key: &str) -> Vec<Arc<str>>;

    /// Record a reference-location entry.
    fn record_reference_location(&self, loc: RefLoc);

    /// Drain pending reference locations staged on this db handle.
    ///
    /// body analysis routes each `record_reference_location` call through a
    /// per-clone staging buffer; consumers (rayon orchestrators, the
    /// `analyze_file` tracked query) drain via this trait method so the
    /// underlying `MirDbStorage` doesn't need to be named.
    fn take_pending_ref_locs(&self) -> Vec<RefLoc>;

    /// Push a fresh reference-location staging frame. Pure per-scope
    /// analysis entry points bracket their walk with push/pop so refs
    /// recorded by a nested tracked query on the same db handle don't leak
    /// into the caller's staged refs.
    fn push_ref_loc_frame(&self);

    /// Pop the top staging frame, returning the refs recorded since the
    /// matching [`Self::push_ref_loc_frame`]. An unbalanced pop drains the
    /// base frame instead of removing it.
    fn pop_ref_loc_frame(&self) -> Vec<RefLoc>;

    /// Extract reference locations for one file in cache-storage shape.
    fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)>;

    /// Return all reference locations for one public symbol key.
    fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)>;

    /// Whether the public symbol key has at least one recorded reference.
    fn has_reference(&self, symbol: &str) -> bool;

    /// Clear reference locations for a file before re-analysis.
    fn clear_file_references(&self, file: &str);

    /// Return all `(file, symbol_key)` pairs recorded across every file.
    /// Used to build the dependency graph from bare-FQN references.
    fn all_reference_location_pairs(&self) -> Vec<(Arc<str>, Arc<str>)>;

    /// Return all symbol keys referenced by `file`. O(degree) via the
    /// file→symbols forward index; use this instead of scanning
    /// `extract_file_reference_locations` for dependency-graph lookups.
    fn file_referenced_symbols(&self, file: &str) -> Vec<Arc<str>>;

    /// Return the Salsa SourceFile handle registered for `path`, if any.
    fn lookup_source_file(&self, path: &str) -> Option<SourceFile>;

    /// Return the singleton [`AnalyzeFileInput`] input handle, lazily
    /// creating it (seeded from `php_version_str()`) on first use. Tracked
    /// queries read `cfg.php_version(db)` so a version change invalidates
    /// their memos; the handle itself is stable for the session, giving
    /// queries like `analyze_file` a repeatable memo key.
    fn analyze_config(&self) -> AnalyzeFileInput;

    /// Return the singleton [`ResolverConfig`] input handle, if a resolver
    /// has ever been attached via `MirDbStorage::set_resolver`. Tracked queries
    /// read `cfg.revision(db)` to anchor on the resolver's version so
    /// they're invalidated when the resolver changes.
    fn resolver_config(&self) -> Option<ResolverConfig>;

    /// Return the current class resolver, if any. **Side channel** — this
    /// read is not salsa-tracked. Tracked queries that consult this must
    /// also read `resolver_config().revision(db)` so salsa correctly
    /// invalidates on resolver swap.
    fn current_resolver(&self) -> Option<Arc<dyn crate::ClassResolver>>;

    /// Return the singleton [`WorkspaceRevision`] input. Tracked
    /// workspace-enumeration queries (`workspace_classes`,
    /// `workspace_functions`) read its `revision` to anchor on
    /// add/remove invalidations.
    fn workspace_revision(&self) -> Option<WorkspaceRevision>;

    /// Return the pre-built workspace symbol index singleton, if populated.
    /// **Side channel** — not salsa-tracked. Call `singleton.index(db)` on
    /// the returned handle to read the index with an O(1) tracked dep
    /// (`Durability::HIGH`). Falls back to the tracked `workspace_symbol_index`
    /// query when `None`.
    fn workspace_symbol_index_singleton(&self) -> Option<WorkspaceSymbolIndexSingleton>;

    /// Borrow a frozen, immutable snapshot of the workspace symbol index, if
    /// this db clone has one (set via `MirDbStorage::freeze_workspace_index`
    /// on an ephemeral read-only pass). Returns `None` on the canonical db and
    /// every clone that hasn't been frozen — callers fall back to
    /// `workspace_index(db)`.
    ///
    /// Borrow-only (`Option<&_>`): the hot `find_class_like` path reads the
    /// index with zero per-call atomics instead of cloning the singleton's
    /// three `Arc`s on every lookup. See the `frozen_index` field docs.
    fn frozen_workspace_index(&self) -> Option<&WorkspaceSymbolIndex>;

    /// Pass-scoped memoization cache for `extends_or_implements`, present only
    /// on a frozen read-only pass (set alongside `frozen_workspace_index`).
    /// `None` on the canonical / open-file db, where the class graph can mutate
    /// mid-analysis and a cache would go stale. See [`SubtypeCache`].
    fn subtype_cache(&self) -> Option<&SubtypeCache>;

    /// Snapshot every registered SourceFile. Side channel — not
    /// salsa-tracked; tracked queries that consult this must also
    /// read `workspace_revision().revision(db)` so file add/remove
    /// correctly invalidates results.
    fn all_source_files(&self) -> Vec<SourceFile>;

    /// Return the subset of registered SourceFile inputs that are user-provided
    /// stubs. Used by `workspace_symbol_index` to give user stubs priority
    /// over native stubs for the same symbol.
    fn user_stub_source_files(&self) -> Vec<SourceFile>;

    /// Return the disk-backed stub slice cache, if configured. **Side channel**
    /// — not salsa-tracked. Safe to use inside tracked queries because the
    /// cache is content-addressed: same `(path, hash, php_version)` always
    /// yields the same `StubSlice`.
    fn stub_cache(&self) -> Option<Arc<crate::stub_cache::StubSliceCache>>;

    /// Return the in-process parse-result cache. **Side channel** — not
    /// salsa-tracked. Populated by `collect_and_ingest_file` so that
    /// `collect_file_definitions_uncached` avoids re-parsing files that were
    /// already parsed in the same session.
    fn parse_cache(&self) -> Arc<crate::parse_cache::ParseCache>;
}

// Re-export all public items from sub-modules to preserve the flat db::* namespace.
pub use self::ancestors::*;
pub use self::deps::file_structural_deps;
pub use self::find_queries::{
    analyzed_class_defs, analyzed_enum_defs, analyzed_interface_defs, analyzed_trait_defs,
    class_ancestors_by_fqcn, class_in_file, class_like_decl_file, enum_in_file,
    find_class_constant_in_chain, find_class_constant_in_class, find_class_like,
    find_function, find_global_constant, find_inheritdoc_parent, find_method_in_chain,
    find_method_in_class, find_method_respecting_precedence, find_property_in_chain,
    find_property_in_class, function_in_file, global_constant_in_file, has_method_in_chain,
    interface_in_file, is_method_concretely_implemented, property_in_own_composition,
    trait_in_file, ClassLike,
};
pub use self::inferred_types::{
    inferred_function_return_type_demand, inferred_method_return_type_demand,
};
#[allow(unused_imports)]
pub use self::mirdb::MirDbStorage;
pub use self::nodes::*;
pub use self::per_function::{infer_function, FunctionInferenceResult};
pub use self::queries::{
    class_constant_exists_in_chain, class_exists, class_kind, class_template_params,
    collect_file_definitions, collect_file_definitions_uncached, constant_exists,
    effective_class_template_params, extends_or_implements, function_exists, has_unknown_ancestor,
    infer_file_return_types, inherited_template_bindings, is_unchecked_exception, member_location,
    parse_file, resolve_name, ClassKind, InferredFileTypes, TrackedParseResult,
};
pub use self::ref_index::RefIndex;
pub use self::reference_locations::*;
pub use self::resolver::{resolve_fqcn_to_path, source_file_for_fqcn, Fqcn, ResolverConfig};
pub use self::scopes::{
    analyze_file_per_scope, file_scopes, infer_scope, ScopeInferenceResult, ScopeKey,
};
pub use self::subtype_index::{ClassLikeKind, SubtypeEntry, SubtypeIndex, SubtypeSite};
pub use self::workspace::{
    collect_file_declarations, workspace_classes, workspace_functions, workspace_global_vars,
    workspace_index, workspace_symbol_index, FileDeclarations, GlobalVarMap, IndexDeclCounts,
    SymbolLoc, SymbolTier, WorkspaceRevision, WorkspaceSymbolIndex, WorkspaceSymbolIndexSingleton,
};

// Sub-modules
mod ancestors;
mod deps;
mod find_queries;
mod inferred_types;
mod mirdb;
mod nodes;
mod per_function;
mod queries;
pub(crate) mod ref_index;
mod reference_locations;
mod resolver;
mod scopes;
pub(crate) mod subtype_index;
mod workspace;

#[cfg(test)]
pub mod tests;
