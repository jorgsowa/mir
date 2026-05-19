use std::collections::HashMap;
use std::sync::Arc;

use mir_types::Union;

// MirDatabase trait

/// Salsa database trait for mir incremental analysis.
#[salsa::db]
pub trait MirDatabase: salsa::Database {
    /// The PHP version configured for this analysis run.
    fn php_version_str(&self) -> Arc<str>;

    /// Look up the [`ClassNode`] handle registered for `fqcn`, if any.
    ///
    /// This is an untracked read — the DashMap holds Salsa input *handles*
    /// (cheap IDs), not data.  Changes to a class's *fields* (parent,
    /// interfaces, active state) are tracked through the `ClassNode` input
    /// itself, so downstream queries are still correctly invalidated.
    fn lookup_class_node(&self, fqcn: &str) -> Option<ClassNode>;

    /// Look up the [`FunctionNode`] handle registered for `fqn`, if any.
    fn lookup_function_node(&self, fqn: &str) -> Option<FunctionNode>;

    /// Look up the [`MethodNode`] for `(fqcn, method_name_lower)`, if any.
    ///
    /// `method_name_lower` must already be lowercased.  This is an untracked
    /// read — changes to a method's fields are tracked through the `MethodNode`
    /// input itself.
    fn lookup_method_node(&self, fqcn: &str, method_name_lower: &str) -> Option<MethodNode>;

    /// Look up the [`PropertyNode`] for `(fqcn, prop_name)`, if any.
    fn lookup_property_node(&self, fqcn: &str, prop_name: &str) -> Option<PropertyNode>;

    /// Look up the [`ClassConstantNode`] for `(fqcn, const_name)`, if any.
    fn lookup_class_constant_node(&self, fqcn: &str, const_name: &str)
        -> Option<ClassConstantNode>;

    /// Look up the [`GlobalConstantNode`] for `fqn`, if any.
    fn lookup_global_constant_node(&self, fqn: &str) -> Option<GlobalConstantNode>;

    /// Return all own-method nodes for `fqcn`.  Empty if no class is
    /// registered.  Untracked iteration of a per-class HashMap.
    fn class_own_methods(&self, fqcn: &str) -> Vec<MethodNode>;

    /// Return all own-property nodes for `fqcn`.  Empty if no class is
    /// registered.  Untracked iteration of a per-class HashMap.
    fn class_own_properties(&self, fqcn: &str) -> Vec<PropertyNode>;

    /// Return all own class-constant nodes for `fqcn`. Empty if no class is
    /// registered. Untracked iteration of a per-class HashMap.
    fn class_own_constants(&self, fqcn: &str) -> Vec<ClassConstantNode>;

    /// Return all class-FQCNs currently registered as active `ClassNode`s,
    /// optionally filtered by kind.  Untracked snapshot — callers should
    /// treat the returned `Vec` as a one-shot view.
    fn active_class_node_fqcns(&self) -> Vec<Arc<str>>;

    /// Return all function-FQNs currently registered as active
    /// `FunctionNode`s.  Untracked snapshot.
    fn active_function_node_fqns(&self) -> Vec<Arc<str>>;

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
pub use self::mirdb::{ClassNodeFields, MirDb};
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
    FqcnIndex, WorkspaceRevision, WorkspaceSymbolIndex,
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
