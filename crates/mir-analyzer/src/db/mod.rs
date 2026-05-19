use std::collections::HashMap;
use std::sync::Arc;

use mir_types::Union;

// MirDatabase trait

/// Salsa database trait for mir incremental analysis.
#[salsa::db]
pub trait MirDatabase: salsa::Database {
    /// The PHP version configured for this analysis run.
    fn php_version_str(&self) -> Arc<str>;

    /// Return this file's first declared namespace, if any.
    fn file_namespace(&self, file: &str) -> Option<Arc<str>>;

    /// Return this file's `use` alias map.
    fn file_imports(&self, file: &str) -> HashMap<String, String>;

    /// Return the known type for a PHP global variable.
    fn global_var_type(&self, name: &str) -> Option<Union>;

    /// Return `(file, imports)` snapshots for every known file.
    fn file_import_snapshots(&self) -> Vec<(Arc<str>, HashMap<String, String>)>;

    /// Return the defining file for a symbol, if known.
    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>>;

    /// Return all symbols whose defining file is `file`.
    fn symbols_defined_in_file(&self, file: &str) -> Vec<Arc<str>>;

    /// Return the set of symbol FQNs currently defined in `file`.
    /// O(1) via the forward index; use instead of `symbols_defined_in_file`
    /// when a `HashSet` is more convenient.
    fn file_defined_symbols(&self, file: &str) -> std::collections::HashSet<Arc<str>>;

    /// Return all files that reference `symbol_key`.
    /// O(1) via the `symbol_referencers` reverse index; valid even after
    /// the symbol has been removed from its defining file.
    fn symbol_referencers_of(&self, symbol_key: &str) -> Vec<Arc<str>>;

    /// Record a reference-location entry.
    fn record_reference_location(&self, loc: RefLoc);

    /// Drain pending reference locations staged on this db handle.
    ///
    /// Pass 2 routes each `record_reference_location` call through a
    /// per-clone staging buffer; consumers (rayon orchestrators, the
    /// `analyze_file` tracked query) drain via this trait method so the
    /// underlying `MirDb` doesn't need to be named.
    fn take_pending_ref_locs(&self) -> Vec<RefLoc>;

    /// Replay reference locations for one file from cache.
    fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u16, u16)]);

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

    /// Return the singleton [`ResolverConfig`] input handle, if a resolver
    /// has ever been attached via `MirDb::set_resolver`. Tracked queries
    /// read `cfg.revision(db)` to anchor on the resolver's version so
    /// they're invalidated when the resolver changes.
    fn resolver_config(&self) -> Option<ResolverConfig>;

    /// Return the current class resolver, if any. **Side channel** — this
    /// read is not salsa-tracked. Tracked queries that consult this must
    /// also read `resolver_config().revision(db)` so salsa correctly
    /// invalidates on resolver swap.
    fn current_resolver(&self) -> Option<Arc<dyn crate::ClassResolver>>;

    /// Return the singleton [`InferredReturnTypes`] input, if the
    /// inference sweep has ever committed. Pass-2 readers go through the
    /// `inferred_*_return_type` helpers in `db::inferred_types`.
    fn inferred_return_types(&self) -> Option<InferredReturnTypes>;

    /// Return the singleton [`WorkspaceRevision`] input. Tracked
    /// workspace-enumeration queries (`workspace_classes`,
    /// `workspace_functions`) read its `revision` to anchor on
    /// add/remove invalidations.
    fn workspace_revision(&self) -> Option<WorkspaceRevision>;

    /// Snapshot every registered SourceFile. Side channel — not
    /// salsa-tracked; tracked queries that consult this must also
    /// read `workspace_revision().revision(db)` so file add/remove
    /// correctly invalidates results.
    fn all_source_files(&self) -> Vec<SourceFile>;

    /// Return the subset of registered SourceFile inputs that are user-provided
    /// stubs. Used by `workspace_symbol_index` to give user stubs priority
    /// over native stubs for the same symbol.
    fn user_stub_source_files(&self) -> Vec<SourceFile>;
}

// Re-export all public items from sub-modules to preserve the flat db::* namespace.
pub use self::ancestors::*;
pub use self::find_queries::{
    class_ancestors_by_fqcn, class_in_file, enum_in_file, find_class_constant_in_chain,
    find_class_constant_in_class, find_class_like, find_function, find_global_constant,
    find_method_in_chain, find_method_in_class, find_property_in_chain, find_property_in_class,
    function_in_file, global_constant_in_file, has_method_in_chain, interface_in_file,
    is_method_concretely_implemented_pull, trait_in_file, ClassLike,
};
pub use self::inferred_types::{
    inferred_function_return_type, inferred_method_return_type, FunctionInferredMap,
    InferredReturnTypes, MethodInferredMap,
};
#[allow(unused_imports)]
pub use self::mirdb::MirDb;
pub use self::nodes::*;
pub use self::queries::{
    class_constant_exists_in_chain, class_kind_via_db, class_template_params_via_db,
    collect_file_definitions, collect_file_definitions_uncached, constant_exists_via_db,
    extends_or_implements_via_db, function_exists_via_db, has_unknown_ancestor_via_db,
    infer_file_return_types, inherited_template_bindings_via_db, is_unchecked_exception_via_db,
    member_location_via_db, method_is_concretely_implemented, resolve_name_via_db,
    type_exists_via_db, ClassKind, InferredFileTypes,
};
pub use self::reference_locations::*;
pub use self::resolver::{resolve_fqcn_to_path, source_file_for_fqcn, Fqcn, ResolverConfig};
pub use self::workspace::{
    workspace_classes, workspace_fqcn_index, workspace_functions, workspace_symbol_index,
    FqcnIndex, SymbolLoc, WorkspaceRevision, WorkspaceSymbolIndex,
};

// Sub-modules
mod ancestors;
mod find_queries;
mod inferred_types;
mod mirdb;
mod nodes;
mod queries;
mod reference_locations;
mod resolver;
mod workspace;

#[cfg(test)]
pub mod tests;
