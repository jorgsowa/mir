use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use mir_codebase::storage::{
    Assertion, ConstantStorage, FnParam, FunctionStorage, MethodStorage, PropertyStorage,
    TemplateParam, Visibility,
};
use mir_codebase::{Codebase, StubSlice};
use mir_issues::Issue;
use mir_types::Union;

// ---------------------------------------------------------------------------
// MirDatabase trait
// ---------------------------------------------------------------------------

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
}

// ---------------------------------------------------------------------------
// SourceFile input (S1)
// ---------------------------------------------------------------------------

/// Source file registered as a Salsa input.
/// Setting `text` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    pub text: Arc<str>,
}

// ---------------------------------------------------------------------------
// FileDefinitions (S1)
// ---------------------------------------------------------------------------

/// Result of the `collect_file_definitions` tracked query.
#[derive(Clone, Debug)]
pub struct FileDefinitions {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

impl PartialEq for FileDefinitions {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.slice, &other.slice) && Arc::ptr_eq(&self.issues, &other.issues)
    }
}

unsafe impl salsa::Update for FileDefinitions {
    unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
        unsafe { *old_pointer = new_value };
        true
    }
}

// ---------------------------------------------------------------------------
// ClassNode input (S2)
// ---------------------------------------------------------------------------

/// Salsa input representing a single class or interface in the inheritance
/// graph.  Fields are kept minimal — only what `class_ancestors` needs.
///
/// Invariant: every FQCN in the codebase that is known to the Salsa DB has
/// exactly one `ClassNode` handle, stored in `MirDb::class_nodes`.  When a
/// class is removed (file deleted or re-indexed), its node is marked
/// `active = false` rather than dropped, so dependent `class_ancestors` queries
/// can still observe the change and re-run.
#[salsa::input]
pub struct ClassNode {
    pub fqcn: Arc<str>,
    /// `false` when the class has been removed from the codebase.  Dependent
    /// queries observe this change and re-run, returning empty ancestors.
    pub active: bool,
    pub is_interface: bool,
    /// `true` for trait nodes.  Traits don't currently participate in the
    /// `class_ancestors` query (matching `Codebase::ensure_finalized` which
    /// returns empty for traits), but registering them as `ClassNode`s lets
    /// callers answer `type_exists`-style questions through the db.
    pub is_trait: bool,
    /// `true` for enum nodes.  See note on `is_trait`.
    pub is_enum: bool,
    /// `true` if the class is declared `abstract`.  Always `false` for
    /// interfaces, traits, and enums.
    pub is_abstract: bool,
    /// Direct parent class (classes only; `None` for interfaces).
    pub parent: Option<Arc<str>>,
    /// Directly implemented interfaces (classes only).
    pub interfaces: Arc<[Arc<str>]>,
    /// Used traits (classes only).  Traits are added to the ancestor list but
    /// their own ancestors are not recursed into, matching PHP semantics.
    pub traits: Arc<[Arc<str>]>,
    /// Directly extended interfaces (interfaces only).
    pub extends: Arc<[Arc<str>]>,
    /// Declared `@template` parameters from the class/interface/trait
    /// docblock.  Empty for classes without templates.
    pub template_params: Arc<[TemplateParam]>,
}

/// Snapshot of a class's discriminator + abstractness, read from a
/// registered active `ClassNode`.
///
/// Returned by [`class_kind_via_db`] when an active node exists for the
/// given FQCN — call sites can use this in place of the corresponding
/// `Codebase` lookups.
#[derive(Debug, Clone, Copy)]
pub struct ClassKind {
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub is_abstract: bool,
}

/// Read class kind/abstractness from an active `ClassNode`, if one is
/// registered for `fqcn`.  Returns `None` for unregistered or inactive
/// nodes, leaving the caller free to fall back to `Codebase` (which still
/// holds bundled-stub types not yet promoted to the db).
pub fn class_kind_via_db(db: &dyn MirDatabase, fqcn: &str) -> Option<ClassKind> {
    let node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;
    Some(ClassKind {
        is_interface: node.is_interface(db),
        is_trait: node.is_trait(db),
        is_enum: node.is_enum(db),
        is_abstract: node.is_abstract(db),
    })
}

/// Whether a class/interface/trait/enum is registered as an active
/// `ClassNode` in the db.  Returns `false` for unregistered or inactive
/// nodes; callers should fall back to `Codebase::type_exists` since
/// bundled-stub types are not yet promoted to the db.
pub fn type_exists_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    db.lookup_class_node(fqcn).is_some_and(|n| n.active(db))
}

/// Return the declared `@template` parameters for `fqcn` from an active
/// `ClassNode`, if one is registered.  Returns `None` for unregistered
/// or inactive nodes; callers should fall back to
/// `Codebase::get_class_template_params`.
pub fn class_template_params_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
) -> Option<Arc<[TemplateParam]>> {
    let node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;
    Some(node.template_params(db))
}

// ---------------------------------------------------------------------------
// FunctionNode input (S5-PR2)
// ---------------------------------------------------------------------------

/// Salsa input representing a single global function.
///
/// `inferred_return_type` is intentionally absent — it lives in
/// `FunctionStorage` until S3 promotes it to a proper tracked query.
///
/// Invariant: every FQN known to the Salsa DB has exactly one `FunctionNode`
/// handle in `MirDb::function_nodes`.  Removed functions are marked
/// `active = false` rather than dropped.
#[salsa::input]
pub struct FunctionNode {
    pub fqn: Arc<str>,
    pub active: bool,
    pub params: Arc<[FnParam]>,
    pub return_type: Option<Union>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
}

// ---------------------------------------------------------------------------
// MethodNode input (S5-PR3)
// ---------------------------------------------------------------------------

/// Salsa input representing a single method or interface/trait method.
///
/// `inferred_return_type` is intentionally absent — it lives in
/// `MethodStorage` until S3 promotes it to a proper tracked query.
///
/// The node is keyed by `(fqcn, method_name_lower)` where `fqcn` is the
/// FQCN of the **owning** class/interface/trait and `method_name_lower` is
/// the PHP-normalised (lowercased) method name.  Nodes for classes that are
/// removed from the codebase are marked `active = false` via
/// `deactivate_class_methods` rather than being dropped.
#[salsa::input]
pub struct MethodNode {
    pub fqcn: Arc<str>,
    pub name: Arc<str>,
    pub active: bool,
    pub params: Arc<[FnParam]>,
    pub return_type: Option<Union>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_constructor: bool,
    pub is_pure: bool,
}

// ---------------------------------------------------------------------------
// PropertyNode input (S5-PR4)
// ---------------------------------------------------------------------------

/// Salsa input representing a single class/trait property.
///
/// `inferred_ty` is intentionally absent — it stays in `PropertyStorage` until
/// a future S3-style tracked query promotes it.
///
/// Keyed by `(owner fqcn, prop_name)` — property names are case-sensitive.
#[salsa::input]
pub struct PropertyNode {
    pub fqcn: Arc<str>,
    pub name: Arc<str>,
    pub active: bool,
    pub ty: Option<Union>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_readonly: bool,
}

// ---------------------------------------------------------------------------
// ClassConstantNode input (S5-PR4)
// ---------------------------------------------------------------------------

/// Salsa input representing a single class/interface/enum constant.
///
/// Keyed by `(owner fqcn, const_name)` — constant names are case-sensitive.
#[salsa::input]
pub struct ClassConstantNode {
    pub fqcn: Arc<str>,
    pub name: Arc<str>,
    pub active: bool,
    pub ty: Union,
    pub visibility: Option<Visibility>,
    pub is_final: bool,
}

// ---------------------------------------------------------------------------
// Ancestors return type (S2)
// ---------------------------------------------------------------------------

/// The computed ancestor list for a class or interface.
///
/// Uses content equality so Salsa's cycle-convergence check can detect
/// fixpoints correctly (two empty lists from different iterations are equal).
#[derive(Clone, Debug, Default)]
pub struct Ancestors(pub Vec<Arc<str>>);

impl PartialEq for Ancestors {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(&other.0)
                .all(|(a, b)| a.as_ref() == b.as_ref())
    }
}

unsafe impl salsa::Update for Ancestors {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

// ---------------------------------------------------------------------------
// class_ancestors tracked query (S2)
// ---------------------------------------------------------------------------

fn ancestors_initial(_db: &dyn MirDatabase, _id: salsa::Id, _node: ClassNode) -> Ancestors {
    Ancestors(vec![])
}

fn ancestors_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &Ancestors,
    _value: Ancestors,
    _node: ClassNode,
) -> Ancestors {
    // PHP class cycles are a compile-time error.  Break immediately with an
    // empty list so the fixpoint converges on the first iteration.
    Ancestors(vec![])
}

/// Salsa tracked query: compute the transitive ancestor list for a class or
/// interface.
///
/// Ancestors are accumulated in the same order as `Codebase::ensure_finalized`:
/// parent → parent's ancestors → implemented interfaces + their ancestors →
/// used traits (class); or: extended interfaces + their ancestors (interface).
///
/// Cycle recovery returns an empty list on the first iteration, which is
/// correct because PHP forbids circular inheritance.
#[salsa::tracked(cycle_fn = ancestors_cycle, cycle_initial = ancestors_initial)]
pub fn class_ancestors(db: &dyn MirDatabase, node: ClassNode) -> Ancestors {
    if !node.active(db) {
        return Ancestors(vec![]);
    }

    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let add = |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut HashSet<String>| {
        if seen.insert(fqcn.to_string()) {
            all.push(fqcn.clone());
        }
    };

    if node.is_interface(db) {
        for e in node.extends(db).iter() {
            add(e, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(e) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
    } else {
        if let Some(ref p) = node.parent(db) {
            add(p, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(p) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for iface in node.interfaces(db).iter() {
            add(iface, &mut all, &mut seen);
            if let Some(iface_node) = db.lookup_class_node(iface) {
                for a in class_ancestors(db, iface_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for t in node.traits(db).iter() {
            add(t, &mut all, &mut seen);
        }
    }

    Ancestors(all)
}

/// Predicate variant of [`Codebase::has_unknown_ancestor`] that prefers the
/// Salsa db when an active `ClassNode` is registered for `fqcn`, falling
/// back to `Codebase` when it isn't.
///
/// When the class is db-registered we walk `class_ancestors` and consider an
/// ancestor "known" if it is either active in the db OR present in
/// `Codebase` (bundled / user stubs aren't promoted to the db yet).
///
/// When the class itself isn't db-registered (or `db` is `None`), we defer
/// entirely to `Codebase::has_unknown_ancestor` — the `class_ancestors`
/// recursion stops at db boundaries and would otherwise miss transitive
/// unknowns reachable only via codebase data.
pub fn has_unknown_ancestor_db_or_codebase(
    db: Option<&dyn MirDatabase>,
    codebase: &Codebase,
    fqcn: &str,
) -> bool {
    if let Some(db) = db {
        if let Some(node) = db.lookup_class_node(fqcn).filter(|n| n.active(db)) {
            for ancestor in class_ancestors(db, node).0.iter() {
                if !type_exists_via_db(db, ancestor) && !codebase.type_exists(ancestor) {
                    return true;
                }
            }
            return false;
        }
    }
    codebase.has_unknown_ancestor(fqcn)
}

// ---------------------------------------------------------------------------
// collect_file_definitions tracked query (S1)
// ---------------------------------------------------------------------------

#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    let path = file.path(db);
    let text = file.text(db);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &text);

    let mut all_issues: Vec<Issue> = parsed
        .errors
        .iter()
        .map(|err| {
            Issue::new(
                mir_issues::IssueKind::ParseError {
                    message: err.to_string(),
                },
                mir_issues::Location {
                    file: path.clone(),
                    line: 1,
                    line_end: 1,
                    col_start: 0,
                    col_end: 0,
                },
            )
        })
        .collect();

    let collector =
        crate::collector::DefinitionCollector::new_for_slice(path, &text, &parsed.source_map);
    let (slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);

    FileDefinitions {
        slice: Arc::new(slice),
        issues: Arc::new(all_issues),
    }
}

// ---------------------------------------------------------------------------
// MirDb concrete database
// ---------------------------------------------------------------------------

/// Concrete in-process Salsa database.
///
/// `Clone` is required for parallel batch analysis: salsa's supported
/// pattern for sharing a db across threads is to give each worker its
/// own clone (each clone gets a fresh `ZalsaLocal`, sharing the
/// underlying memoization storage).  Sharing `&MirDb` across threads is
/// **not** supported because `salsa::Database: Send` (not `Sync`).
#[salsa::db]
#[derive(Default, Clone)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
    /// FQCN → ClassNode handle registry (not tracked by Salsa; see
    /// `lookup_class_node` for the rationale).
    class_nodes: HashMap<Arc<str>, ClassNode>,
    /// FQN → FunctionNode handle registry.
    function_nodes: HashMap<Arc<str>, FunctionNode>,
    /// (owner FQCN) → (method_name_lower → MethodNode) handle registry.
    method_nodes: HashMap<Arc<str>, HashMap<Arc<str>, MethodNode>>,
    /// (owner FQCN) → (prop_name → PropertyNode) handle registry.
    property_nodes: HashMap<Arc<str>, HashMap<Arc<str>, PropertyNode>>,
    /// (owner FQCN) → (const_name → ClassConstantNode) handle registry.
    class_constant_nodes: HashMap<Arc<str>, HashMap<Arc<str>, ClassConstantNode>>,
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        Arc::from("8.2")
    }

    fn lookup_class_node(&self, fqcn: &str) -> Option<ClassNode> {
        self.class_nodes.get(fqcn).copied()
    }

    fn lookup_function_node(&self, fqn: &str) -> Option<FunctionNode> {
        self.function_nodes.get(fqn).copied()
    }

    fn lookup_method_node(&self, fqcn: &str, method_name_lower: &str) -> Option<MethodNode> {
        self.method_nodes
            .get(fqcn)
            .and_then(|m| m.get(method_name_lower).copied())
    }

    fn lookup_property_node(&self, fqcn: &str, prop_name: &str) -> Option<PropertyNode> {
        self.property_nodes
            .get(fqcn)
            .and_then(|m| m.get(prop_name).copied())
    }

    fn lookup_class_constant_node(
        &self,
        fqcn: &str,
        const_name: &str,
    ) -> Option<ClassConstantNode> {
        self.class_constant_nodes
            .get(fqcn)
            .and_then(|m| m.get(const_name).copied())
    }
}

impl MirDb {
    /// Create or update the `ClassNode` for `fqcn`.
    ///
    /// If a handle already exists, its fields are updated in-place so Salsa
    /// can track the change.  A new handle is created only on first registration.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_class_node(
        &mut self,
        fqcn: Arc<str>,
        is_interface: bool,
        is_trait: bool,
        is_enum: bool,
        is_abstract: bool,
        parent: Option<Arc<str>>,
        interfaces: Arc<[Arc<str>]>,
        traits: Arc<[Arc<str>]>,
        extends: Arc<[Arc<str>]>,
        template_params: Arc<[TemplateParam]>,
    ) -> ClassNode {
        use salsa::Setter as _;
        if let Some(&node) = self.class_nodes.get(&fqcn) {
            node.set_active(self).to(true);
            node.set_is_interface(self).to(is_interface);
            node.set_is_trait(self).to(is_trait);
            node.set_is_enum(self).to(is_enum);
            node.set_is_abstract(self).to(is_abstract);
            node.set_parent(self).to(parent);
            node.set_interfaces(self).to(interfaces);
            node.set_traits(self).to(traits);
            node.set_extends(self).to(extends);
            node.set_template_params(self).to(template_params);
            node
        } else {
            let node = ClassNode::new(
                self,
                fqcn.clone(),
                true,
                is_interface,
                is_trait,
                is_enum,
                is_abstract,
                parent,
                interfaces,
                traits,
                extends,
                template_params,
            );
            self.class_nodes.insert(fqcn, node);
            node
        }
    }

    /// Mark the `ClassNode` for `fqcn` as inactive.
    ///
    /// Dependent `class_ancestors` queries will observe the change and re-run,
    /// returning an empty list.
    pub fn deactivate_class_node(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.class_nodes.get(fqcn) {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `FunctionNode` for the given `FunctionStorage`.
    pub fn upsert_function_node(&mut self, storage: &FunctionStorage) -> FunctionNode {
        use salsa::Setter as _;
        let fqn = &storage.fqn;
        if let Some(&node) = self.function_nodes.get(fqn.as_ref()) {
            node.set_active(self).to(true);
            node.set_params(self)
                .to(Arc::from(storage.params.as_slice()));
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_is_pure(self).to(storage.is_pure);
            node
        } else {
            let node = FunctionNode::new(
                self,
                fqn.clone(),
                true,
                Arc::from(storage.params.as_slice()),
                storage.return_type.clone(),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.is_pure,
            );
            self.function_nodes.insert(fqn.clone(), node);
            node
        }
    }

    /// Mark the `FunctionNode` for `fqn` as inactive.
    pub fn deactivate_function_node(&mut self, fqn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.function_nodes.get(fqn) {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `MethodNode` for `(storage.fqcn, storage.name.to_lowercase())`.
    pub fn upsert_method_node(&mut self, storage: &MethodStorage) -> MethodNode {
        use salsa::Setter as _;
        let fqcn = &storage.fqcn;
        let name_lower: Arc<str> = Arc::from(storage.name.to_lowercase().as_str());
        // Copy the existing handle out to release the immutable borrow before
        // calling node.set_*(self), which needs &mut self.
        let existing = self
            .method_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(&name_lower))
            .copied();
        if let Some(node) = existing {
            node.set_active(self).to(true);
            node.set_params(self)
                .to(Arc::from(storage.params.as_slice()));
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_static(self).to(storage.is_static);
            node.set_is_abstract(self).to(storage.is_abstract);
            node.set_is_final(self).to(storage.is_final);
            node.set_is_constructor(self).to(storage.is_constructor);
            node.set_is_pure(self).to(storage.is_pure);
            node
        } else {
            // MethodNode::new takes &mut self; insert after it returns.
            let node = MethodNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                Arc::from(storage.params.as_slice()),
                storage.return_type.clone(),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.visibility,
                storage.is_static,
                storage.is_abstract,
                storage.is_final,
                storage.is_constructor,
                storage.is_pure,
            );
            self.method_nodes
                .entry(fqcn.clone())
                .or_default()
                .insert(name_lower, node);
            node
        }
    }

    /// Mark all `MethodNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_methods(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<MethodNode> = match self.method_nodes.get(fqcn) {
            Some(methods) => methods.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `PropertyNode` for `(storage.fqcn, storage.name)`.
    pub fn upsert_property_node(&mut self, fqcn: &Arc<str>, storage: &PropertyStorage) {
        use salsa::Setter as _;
        let existing = self
            .property_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(storage.name.as_ref()))
            .copied();
        if let Some(node) = existing {
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_static(self).to(storage.is_static);
            node.set_is_readonly(self).to(storage.is_readonly);
        } else {
            let node = PropertyNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.ty.clone(),
                storage.visibility,
                storage.is_static,
                storage.is_readonly,
            );
            self.property_nodes
                .entry(fqcn.clone())
                .or_default()
                .insert(storage.name.clone(), node);
        }
    }

    /// Mark all `PropertyNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_properties(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<PropertyNode> = match self.property_nodes.get(fqcn) {
            Some(props) => props.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `ClassConstantNode` for `(fqcn, storage.name)`.
    pub fn upsert_class_constant_node(&mut self, fqcn: &Arc<str>, storage: &ConstantStorage) {
        use salsa::Setter as _;
        let existing = self
            .class_constant_nodes
            .get(fqcn.as_ref())
            .and_then(|m| m.get(storage.name.as_ref()))
            .copied();
        if let Some(node) = existing {
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_final(self).to(storage.is_final);
        } else {
            let node = ClassConstantNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.ty.clone(),
                storage.visibility,
                storage.is_final,
            );
            self.class_constant_nodes
                .entry(fqcn.clone())
                .or_default()
                .insert(storage.name.clone(), node);
        }
    }

    /// Walk every entry in `codebase` and upsert the corresponding db node.
    ///
    /// Used after bundled / user stubs are loaded into `Codebase` so that
    /// `type_exists_via_db` / `class_kind_via_db` / `class_template_params_via_db`
    /// see them too.  Idempotent — re-running upserts existing nodes in place
    /// without invalidating downstream queries when fields are unchanged.
    pub fn ingest_codebase(&mut self, codebase: &Codebase) {
        for entry in codebase.classes.iter() {
            let cls = entry.value();
            self.upsert_class_node(
                cls.fqcn.clone(),
                false,
                false,
                false,
                cls.is_abstract,
                cls.parent.clone(),
                Arc::from(cls.interfaces.as_slice()),
                Arc::from(cls.traits.as_slice()),
                Arc::from([]),
                Arc::from(cls.template_params.as_slice()),
            );
            for method in cls.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            for prop in cls.own_properties.values() {
                self.upsert_property_node(&cls.fqcn, prop);
            }
            for constant in cls.own_constants.values() {
                self.upsert_class_constant_node(&cls.fqcn, constant);
            }
        }
        for entry in codebase.interfaces.iter() {
            let iface = entry.value();
            self.upsert_class_node(
                iface.fqcn.clone(),
                true,
                false,
                false,
                false,
                None,
                Arc::from([]),
                Arc::from([]),
                Arc::from(iface.extends.as_slice()),
                Arc::from(iface.template_params.as_slice()),
            );
            for method in iface.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            for constant in iface.own_constants.values() {
                self.upsert_class_constant_node(&iface.fqcn, constant);
            }
        }
        for entry in codebase.traits.iter() {
            let tr = entry.value();
            self.upsert_class_node(
                tr.fqcn.clone(),
                false,
                true,
                false,
                false,
                None,
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from(tr.template_params.as_slice()),
            );
            for method in tr.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            for prop in tr.own_properties.values() {
                self.upsert_property_node(&tr.fqcn, prop);
            }
            for constant in tr.own_constants.values() {
                self.upsert_class_constant_node(&tr.fqcn, constant);
            }
        }
        for entry in codebase.enums.iter() {
            let en = entry.value();
            self.upsert_class_node(
                en.fqcn.clone(),
                false,
                false,
                true,
                false,
                None,
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
                Arc::from([]),
            );
            for method in en.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            for constant in en.own_constants.values() {
                self.upsert_class_constant_node(&en.fqcn, constant);
            }
        }
        for entry in codebase.functions.iter() {
            self.upsert_function_node(entry.value());
        }
    }

    /// Mark all `ClassConstantNode`s owned by `fqcn` as inactive.
    pub fn deactivate_class_constants(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        let nodes: Vec<ClassConstantNode> = match self.class_constant_nodes.get(fqcn) {
            Some(consts) => consts.values().copied().collect(),
            None => return,
        };
        for node in nodes {
            node.set_active(self).to(false);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    fn upsert_class(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        extends: Arc<[Arc<str>]>,
        is_interface: bool,
    ) -> ClassNode {
        db.upsert_class_node(
            Arc::from(fqcn),
            is_interface,
            false,
            false,
            false,
            parent,
            Arc::from([]),
            Arc::from([]),
            extends,
            Arc::from([]),
        )
    }

    #[test]
    fn mirdb_constructs() {
        let _db = MirDb::default();
    }

    #[test]
    fn source_file_input_roundtrip() {
        let db = MirDb::default();
        let file = SourceFile::new(&db, Arc::from("/tmp/test.php"), Arc::from("<?php echo 1;"));
        assert_eq!(file.path(&db).as_ref(), "/tmp/test.php");
        assert_eq!(file.text(&db).as_ref(), "<?php echo 1;");
    }

    #[test]
    fn collect_file_definitions_basic() {
        let db = MirDb::default();
        let src = Arc::from("<?php class Foo {}");
        let file = SourceFile::new(&db, Arc::from("/tmp/foo.php"), src);
        let defs = collect_file_definitions(&db, file);
        assert!(defs.issues.is_empty());
        assert_eq!(defs.slice.classes.len(), 1);
        assert_eq!(defs.slice.classes[0].fqcn.as_ref(), "Foo");
    }

    #[test]
    fn collect_file_definitions_memoized() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo.php"),
            Arc::from("<?php class Bar {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        let defs2 = collect_file_definitions(&db, file);
        assert!(
            Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "unchanged file must return the memoized result"
        );
    }

    #[test]
    fn collect_file_definitions_recomputes_on_change() {
        let mut db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo2.php"),
            Arc::from("<?php class Foo {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        file.set_text(&mut db)
            .to(Arc::from("<?php class Foo {} class Bar {}"));
        let defs2 = collect_file_definitions(&db, file);

        assert!(
            !Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "changed file must produce a new result"
        );
        assert_eq!(defs2.slice.classes.len(), 2);
    }

    #[test]
    fn class_ancestors_empty_for_root_class() {
        let mut db = MirDb::default();
        let node = upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "root class has no ancestors");
    }

    #[test]
    fn class_ancestors_single_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        let child = upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        let ancestors = class_ancestors(&db, child);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Base");
    }

    #[test]
    fn class_ancestors_transitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "GrandParent", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Parent",
            Some(Arc::from("GrandParent")),
            Arc::from([]),
            false,
        );
        let child = upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Parent")),
            Arc::from([]),
            false,
        );
        let ancestors = class_ancestors(&db, child);
        assert_eq!(ancestors.0.len(), 2);
        assert_eq!(ancestors.0[0].as_ref(), "Parent");
        assert_eq!(ancestors.0[1].as_ref(), "GrandParent");
    }

    #[test]
    fn class_ancestors_cycle_returns_empty() {
        let mut db = MirDb::default();
        // A extends A — not valid PHP, but we must not panic.
        let node_a = upsert_class(&mut db, "A", Some(Arc::from("A")), Arc::from([]), false);
        let ancestors = class_ancestors(&db, node_a);
        // Cycle recovery: empty list (A's ancestors exclude itself).
        assert!(ancestors.0.is_empty(), "cycle must yield empty ancestors");
    }

    #[test]
    fn class_ancestors_inactive_node_returns_empty() {
        let mut db = MirDb::default();
        let node = upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        db.deactivate_class_node("Foo");
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "inactive node must yield empty");
    }

    #[test]
    fn class_ancestors_recomputes_on_parent_change() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        let child = upsert_class(&mut db, "Child", None, Arc::from([]), false);

        let before = class_ancestors(&db, child);
        assert!(before.0.is_empty());

        // Add Base as parent of Child.
        child.set_parent(&mut db).to(Some(Arc::from("Base")));

        let after = class_ancestors(&db, child);
        assert_eq!(after.0.len(), 1);
        assert_eq!(after.0[0].as_ref(), "Base");
    }

    #[test]
    fn interface_ancestors_via_extends() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Countable", None, Arc::from([]), true);
        let child_iface = upsert_class(
            &mut db,
            "Collection",
            None,
            Arc::from([Arc::from("Countable")]),
            true,
        );
        let ancestors = class_ancestors(&db, child_iface);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Countable");
    }

    #[test]
    fn type_exists_via_db_tracks_active_state() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        assert!(type_exists_via_db(&db, "Foo"));
        assert!(!type_exists_via_db(&db, "Bar"));
        db.deactivate_class_node("Foo");
        assert!(!type_exists_via_db(&db, "Foo"));
    }

    #[test]
    fn clone_preserves_class_node_lookups() {
        // PR10a: each parallel batch worker gets its own MirDb clone.
        // Verify the clone observes the same registered nodes.
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        let cloned = db.clone();
        assert!(
            type_exists_via_db(&cloned, "Foo"),
            "clone must observe nodes registered before clone()"
        );
        assert!(
            !type_exists_via_db(&cloned, "Bar"),
            "clone must not observe nodes that were never registered"
        );
        // Clones must also resolve ancestors through the same shared storage.
        let foo_node = cloned.lookup_class_node("Foo").expect("registered");
        let ancestors = class_ancestors(&cloned, foo_node);
        assert!(ancestors.0.is_empty(), "Foo has no ancestors");
    }

    #[test]
    fn class_template_params_via_db_returns_registered_params() {
        use mir_types::Variance;
        let mut db = MirDb::default();
        let tp = TemplateParam {
            name: Arc::from("T"),
            bound: None,
            defining_entity: Arc::from("Box"),
            variance: Variance::Invariant,
        };
        db.upsert_class_node(
            Arc::from("Box"),
            false,
            false,
            false,
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
            Arc::from([tp.clone()]),
        );
        let got = class_template_params_via_db(&db, "Box").expect("registered");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name.as_ref(), "T");

        assert!(class_template_params_via_db(&db, "Missing").is_none());
        db.deactivate_class_node("Box");
        assert!(class_template_params_via_db(&db, "Box").is_none());
    }
}
