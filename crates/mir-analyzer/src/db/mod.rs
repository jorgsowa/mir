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

    /// Record a reference-location entry.
    fn record_reference_location(&self, loc: RefLoc);

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

    /// Return the Salsa SourceFile handle registered for `path`, if any.
    fn lookup_source_file(&self, path: &str) -> Option<SourceFile>;
}

// Re-export all public items from sub-modules to preserve the flat db::* namespace.
pub use self::ancestors::*;
#[allow(unused_imports)]
pub use self::mirdb::{ClassNodeFields, MirDb};
pub use self::nodes::*;
pub use self::queries::*;
pub use self::reference_locations::*;

// Sub-modules
mod ancestors;
mod mirdb;
mod nodes;
mod queries;
mod reference_locations;

#[cfg(test)]
pub mod tests;
