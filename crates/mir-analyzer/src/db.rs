use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use mir_codebase::storage::{
    Assertion, ConstantStorage, FnParam, FunctionStorage, Location, MethodStorage, PropertyStorage,
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

    /// Look up the [`GlobalConstantNode`] for `fqn`, if any.
    fn lookup_global_constant_node(&self, fqn: &str) -> Option<GlobalConstantNode>;

    /// Return all own-method nodes for `fqcn`.  Empty if no class is
    /// registered.  Untracked iteration of a per-class HashMap.
    fn class_own_methods(&self, fqcn: &str) -> Vec<MethodNode>;

    /// Return all own-property nodes for `fqcn`.  Empty if no class is
    /// registered.  Untracked iteration of a per-class HashMap.
    fn class_own_properties(&self, fqcn: &str) -> Vec<PropertyNode>;

    /// Return all class-FQCNs currently registered as active `ClassNode`s,
    /// optionally filtered by kind.  Untracked snapshot — callers should
    /// treat the returned `Vec` as a one-shot view.
    fn active_class_node_fqcns(&self) -> Vec<Arc<str>>;

    /// Return all function-FQNs currently registered as active
    /// `FunctionNode`s.  Untracked snapshot.
    fn active_function_node_fqns(&self) -> Vec<Arc<str>>;
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

/// `(interface_fqcn, type_args)` pairs from `@implements Iface<T1, T2>`
/// docblocks.  Stored on `ClassNode` for classes only.
pub type ImplementsTypeArgs = Arc<[(Arc<str>, Arc<[Union]>)]>;

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
    /// `@psalm-require-extends` / `@phpstan-require-extends` — FQCNs that
    /// using classes must extend.  Populated for trait nodes only; empty for
    /// classes/interfaces/enums.
    pub require_extends: Arc<[Arc<str>]>,
    /// `@psalm-require-implements` / `@phpstan-require-implements` — FQCNs
    /// that using classes must implement.  Populated for trait nodes only;
    /// empty for classes/interfaces/enums.
    pub require_implements: Arc<[Arc<str>]>,
    /// `true` if this is a *backed* enum (declared with a scalar type).
    /// Always `false` for non-enum nodes and pure (unbacked) enums.  Used by
    /// `extends_or_implements_via_db` to answer the implicit `BackedEnum`
    /// interface check.
    pub is_backed_enum: bool,
    /// `@mixin` / `@psalm-mixin` FQCNs declared on the class docblock.
    /// Used by `lookup_method_in_chain` for delegated magic-method lookup,
    /// matching `Codebase::get_method`'s mixin walk.  Empty for interfaces,
    /// traits, and enums (mixin is a class-only docblock concept).
    pub mixins: Arc<[Arc<str>]>,
    /// `@deprecated` message from the class docblock, if any.  Mirrors
    /// `ClassStorage::deprecated`.  Empty / `None` for interfaces, traits,
    /// and enums (S5-PR42 only mirrors the class-level field — those storages
    /// don't carry a deprecated message).
    pub deprecated: Option<Arc<str>>,
    /// For backed-enum nodes: the declared scalar type (`int`/`string`).
    /// Mirrors `EnumStorage::scalar_type`.  `None` for non-enum nodes and
    /// for unbacked (pure) enums.  Used by the `Enum->value` property read
    /// in `expr.rs` to return the backed scalar type instead of `mixed`.
    pub enum_scalar_type: Option<Union>,
    /// `true` if the class is declared `final`.  Always `false` for
    /// interfaces, traits, and enums (PHP enums are implicitly final but the
    /// codebase doesn't currently track that on `EnumStorage`).
    pub is_final: bool,
    /// `true` if the class is declared `readonly`.  Always `false` for
    /// non-class kinds.
    pub is_readonly: bool,
    /// Source location of the class declaration.  Mirrors
    /// `ClassStorage::location` (and `InterfaceStorage::location`,
    /// `TraitStorage::location`, `EnumStorage::location`).  Used by
    /// `ClassAnalyzer` to attribute issues to the right span.
    pub location: Option<Location>,
    /// Type arguments from `@extends Parent<T1, T2>` — populated for
    /// classes only.  Mirrors `ClassStorage::extends_type_args`.
    pub extends_type_args: Arc<[Union]>,
    /// Type arguments from `@implements Iface<T1, T2>` — populated for
    /// classes only.  Mirrors `ClassStorage::implements_type_args`.
    pub implements_type_args: ImplementsTypeArgs,
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
/// nodes.  All bundled and user types are mirrored into `ClassNode` by
/// `MirDb::ingest_codebase`, so a `None` here means the type genuinely
/// doesn't exist (or is inactive after a `deactivate_class_node` pass).
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
/// nodes.  After `MirDb::ingest_codebase` runs (S5-PR8/PR9), this is
/// the authoritative answer — bundled and user types are both mirrored.
pub fn type_exists_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    db.lookup_class_node(fqcn).is_some_and(|n| n.active(db))
}

/// Return the declared `@template` parameters for `fqcn` from an active
/// `ClassNode`, if one is registered.  Returns `None` for unregistered
/// or inactive nodes.  Authoritative after `ingest_codebase`.
pub fn class_template_params_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
) -> Option<Arc<[TemplateParam]>> {
    let node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;
    Some(node.template_params(db))
}

/// Walk the parent chain collecting template bindings from `@extends` type
/// args.  Mirrors `Codebase::get_inherited_template_bindings`.
///
/// For `class UserRepo extends BaseRepo` with `@extends BaseRepo<User>`, this
/// returns `{ T → User }` where `T` is `BaseRepo`'s declared template
/// parameter.  Cycle-safe via a visited set.
pub fn inherited_template_bindings_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
) -> std::collections::HashMap<Arc<str>, Union> {
    let mut bindings: std::collections::HashMap<Arc<str>, Union> = std::collections::HashMap::new();
    let mut visited: std::collections::HashSet<Arc<str>> = std::collections::HashSet::new();
    let mut current: Arc<str> = Arc::from(fqcn);
    loop {
        if !visited.insert(current.clone()) {
            break;
        }
        let node = match db
            .lookup_class_node(current.as_ref())
            .filter(|n| n.active(db))
        {
            Some(n) => n,
            None => break,
        };
        let parent = match node.parent(db) {
            Some(p) => p,
            None => break,
        };
        let extends_type_args = node.extends_type_args(db);
        if !extends_type_args.is_empty() {
            if let Some(parent_tps) = class_template_params_via_db(db, parent.as_ref()) {
                for (tp, ty) in parent_tps.iter().zip(extends_type_args.iter()) {
                    bindings
                        .entry(tp.name.clone())
                        .or_insert_with(|| ty.clone());
                }
            }
        }
        current = parent;
    }
    bindings
}

// ---------------------------------------------------------------------------
// FunctionNode input (S5-PR2)
// ---------------------------------------------------------------------------

/// Salsa input representing a single global function.
///
/// `inferred_return_type` is intentionally absent — by design it lives in
/// `FunctionStorage` (read via `Codebase::functions.get(...).inferred_return_type`).
/// Promoting it to a Salsa tracked field deadlocks against rayon's per-worker
/// db clones in the priming sweep (`Storage::cancel_others` waits for
/// strong-count==1 forever); see ROADMAP "S3 deadlock" for the five
/// resolution candidates and why none is uniformly safe.  Treat this as
/// the chosen design, not migration debt.
///
/// Invariant: every FQN known to the Salsa DB has exactly one `FunctionNode`
/// handle in `MirDb::function_nodes`.  Removed functions are marked
/// `active = false` rather than dropped.
#[salsa::input]
pub struct FunctionNode {
    pub fqn: Arc<str>,
    pub short_name: Arc<str>,
    pub active: bool,
    pub params: Arc<[FnParam]>,
    pub return_type: Option<Union>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
    /// Source location of the declaration.  `None` for functions registered
    /// without a known origin (e.g. some legacy test fixtures).
    pub location: Option<Location>,
}

// ---------------------------------------------------------------------------
// MethodNode input (S5-PR3)
// ---------------------------------------------------------------------------

/// Salsa input representing a single method or interface/trait method.
///
/// `inferred_return_type` is intentionally absent — by design it lives in
/// `MethodStorage` (read via `Codebase::method_inferred_return_type`).
/// Same rayon/Salsa deadlock rationale as `FunctionNode`; see that doc
/// comment + ROADMAP "S3 deadlock".
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
    /// Source location of the declaration.  `None` for synthesized methods
    /// (e.g. enum implicit `cases`/`from`/`tryFrom`).
    pub location: Option<Location>,
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
    pub location: Option<Location>,
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
// GlobalConstantNode input (S5-PR47)
// ---------------------------------------------------------------------------

/// Salsa input representing a global PHP constant (e.g. `PHP_EOL`).
/// Mirrors `Codebase::constants`.
#[salsa::input]
pub struct GlobalConstantNode {
    pub fqn: Arc<str>,
    pub active: bool,
    pub ty: Union,
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
    // Invariant: enums and traits always return empty here.
    // - Enums: matches `Codebase::ensure_finalized`.  Enum membership
    //   questions go through `extends_or_implements_via_db`, which reads
    //   `interfaces` / `is_backed_enum` directly.
    // - Traits: matches `Codebase::ensure_finalized` (which only computes
    //   ancestors for classes/interfaces).  Trait-of-trait walking is
    //   handled by `method_is_concretely_implemented` /
    //   `trait_provides_method` directly via the `traits` field.
    // Do not lift either short-circuit without also auditing every caller
    // of `class_ancestors`.
    if node.is_enum(db) || node.is_trait(db) {
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

/// Predicate variant of [`Codebase::has_unknown_ancestor`] backed by the
/// Salsa db.
///
/// `ingest_codebase` (S5-PR8/PR9 / PR11a) mirrors bundled stubs, user
/// stubs, and PSR-4 lazy-loaded definitions into the db before any
/// Pass 2 driver runs, so a class with no active `ClassNode` is one
/// that genuinely doesn't exist — and an unknown class trivially has
/// no known ancestors.
pub fn has_unknown_ancestor_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let Some(node) = db.lookup_class_node(fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    class_ancestors(db, node)
        .0
        .iter()
        .any(|ancestor| !type_exists_via_db(db, ancestor))
}

/// Returns `true` iff `fqcn` (or any non-interface ancestor) declares a
/// *concrete* (non-abstract) implementation of `method_name`.  Methods
/// declared on interface ancestors are treated as abstract — interfaces don't
/// supply implementations even though their `MethodStorage` is collected with
/// `is_abstract = false`.  Mirrors the implemented-method semantics that
/// [`Codebase::get_method`] hand-rolls via its `ms.is_abstract = true`
/// rewrite for interface ancestors.
///
/// Method names are PHP-case-insensitive; the lookup lower-cases internally.
/// Cycle-safe: relies on `class_ancestors` cycle recovery.
pub fn method_is_concretely_implemented(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> bool {
    let lower = method_name.to_lowercase();
    let Some(self_node) = db.lookup_class_node(fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    // Interfaces don't supply implementations, regardless of how their methods
    // are stored.
    if self_node.is_interface(db) {
        return false;
    }
    // 1. Direct own method.
    if let Some(m) = db.lookup_method_node(fqcn, &lower).filter(|m| m.active(db)) {
        if !m.is_abstract(db) {
            return true;
        }
    }
    // 2. Traits used directly by this class — walk transitively.
    let mut visited_traits: HashSet<String> = HashSet::new();
    for t in self_node.traits(db).iter() {
        if trait_provides_method(db, t.as_ref(), &lower, &mut visited_traits) {
            return true;
        }
    }
    // 3. Ancestor chain (classes only — interfaces skipped, trait nodes here
    //    are owning-class trait references already handled by their own walk).
    for ancestor in class_ancestors(db, self_node).0.iter() {
        let Some(anc_node) = db
            .lookup_class_node(ancestor.as_ref())
            .filter(|n| n.active(db))
        else {
            continue;
        };
        if anc_node.is_interface(db) {
            continue;
        }
        // Ancestor's own method.
        if !anc_node.is_trait(db) {
            if let Some(m) = db
                .lookup_method_node(ancestor.as_ref(), &lower)
                .filter(|m| m.active(db))
            {
                if !m.is_abstract(db) {
                    return true;
                }
            }
        }
        // Ancestor's used traits — walk transitively.  (For trait nodes in
        // the ancestor list, this re-checks their own_methods + sub-traits.)
        if anc_node.is_trait(db) {
            if trait_provides_method(db, ancestor.as_ref(), &lower, &mut visited_traits) {
                return true;
            }
        } else {
            for t in anc_node.traits(db).iter() {
                if trait_provides_method(db, t.as_ref(), &lower, &mut visited_traits) {
                    return true;
                }
            }
        }
    }
    false
}

/// Helper for [`method_is_concretely_implemented`]: walk a trait's own methods
/// and recursively its used traits.  Returns true iff any provides a
/// non-abstract method named `method_lower`.  Cycle-safe via `visited`.
fn trait_provides_method(
    db: &dyn MirDatabase,
    trait_fqcn: &str,
    method_lower: &str,
    visited: &mut HashSet<String>,
) -> bool {
    if !visited.insert(trait_fqcn.to_string()) {
        return false;
    }
    if let Some(m) = db
        .lookup_method_node(trait_fqcn, method_lower)
        .filter(|m| m.active(db))
    {
        if !m.is_abstract(db) {
            return true;
        }
    }
    let Some(node) = db.lookup_class_node(trait_fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    if !node.is_trait(db) {
        return false;
    }
    for t in node.traits(db).iter() {
        if trait_provides_method(db, t.as_ref(), method_lower, visited) {
            return true;
        }
    }
    false
}

/// Returns `true` iff `fqcn` (or any ancestor / used trait, transitively)
/// declares a method named `method_name` (abstract or concrete).  Used by
/// magic-method existence checks (`__call`, `__callStatic`, `__invoke`,
/// `__construct`) and intersection-type method lookups.
///
/// Method names are PHP-case-insensitive; the lookup lower-cases internally.
/// Cycle-safe: relies on `class_ancestors` cycle recovery and a per-call
/// `visited` set across trait-of-trait walks.
/// Walk `fqcn`'s own MethodNode then the class-ancestor chain, returning the
/// first active [`MethodNode`] whose name matches `method_name` (case-
/// insensitive).  Mirrors [`Codebase::get_method`]'s ancestor walk.
///
/// Used when a caller needs the full method node (params, return type,
/// visibility, etc.), not just an existence check.
pub fn lookup_method_in_chain(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> Option<MethodNode> {
    let mut visited_mixins: HashSet<String> = HashSet::new();
    lookup_method_in_chain_inner(db, fqcn, &method_name.to_lowercase(), &mut visited_mixins)
}

fn lookup_method_in_chain_inner(
    db: &dyn MirDatabase,
    fqcn: &str,
    lower: &str,
    visited_mixins: &mut HashSet<String>,
) -> Option<MethodNode> {
    let self_node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;

    // 1. Direct own method.
    if let Some(node) = db.lookup_method_node(fqcn, lower).filter(|n| n.active(db)) {
        return Some(node);
    }
    // 2. Docblock @mixin chains (delegated magic-method lookup) — recurse so
    //    each mixin's own walk includes its own mixins, traits, ancestors.
    //    Cycle-safe via `visited_mixins`.
    for m in self_node.mixins(db).iter() {
        if visited_mixins.insert(m.to_string()) {
            if let Some(node) = lookup_method_in_chain_inner(db, m.as_ref(), lower, visited_mixins)
            {
                return Some(node);
            }
        }
    }
    // 3. Traits used directly — walk transitively (trait-of-traits is *not*
    //    included in `class_ancestors`, by design — see that fn's comments).
    let mut visited_traits: HashSet<String> = HashSet::new();
    for t in self_node.traits(db).iter() {
        if let Some(node) = trait_provides_method_node(db, t.as_ref(), lower, &mut visited_traits) {
            return Some(node);
        }
    }
    // 4. Ancestor chain (parents, interfaces, traits — empty for enums).
    for ancestor in class_ancestors(db, self_node).0.iter() {
        if let Some(node) = db
            .lookup_method_node(ancestor.as_ref(), lower)
            .filter(|n| n.active(db))
        {
            return Some(node);
        }
        if let Some(anc_node) = db
            .lookup_class_node(ancestor.as_ref())
            .filter(|n| n.active(db))
        {
            if anc_node.is_trait(db) {
                if let Some(node) =
                    trait_provides_method_node(db, ancestor.as_ref(), lower, &mut visited_traits)
                {
                    return Some(node);
                }
            } else {
                for t in anc_node.traits(db).iter() {
                    if let Some(node) =
                        trait_provides_method_node(db, t.as_ref(), lower, &mut visited_traits)
                    {
                        return Some(node);
                    }
                }
                for m in anc_node.mixins(db).iter() {
                    if visited_mixins.insert(m.to_string()) {
                        if let Some(node) =
                            lookup_method_in_chain_inner(db, m.as_ref(), lower, visited_mixins)
                        {
                            return Some(node);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Node-returning sibling of [`trait_declares_method`] used by
/// [`lookup_method_in_chain`].  Walks `trait_fqcn`'s own MethodNode then its
/// used traits transitively.  Cycle-safe via `visited`.
fn trait_provides_method_node(
    db: &dyn MirDatabase,
    trait_fqcn: &str,
    method_lower: &str,
    visited: &mut HashSet<String>,
) -> Option<MethodNode> {
    if !visited.insert(trait_fqcn.to_string()) {
        return None;
    }
    if let Some(node) = db
        .lookup_method_node(trait_fqcn, method_lower)
        .filter(|n| n.active(db))
    {
        return Some(node);
    }
    let node = db.lookup_class_node(trait_fqcn).filter(|n| n.active(db))?;
    if !node.is_trait(db) {
        return None;
    }
    for t in node.traits(db).iter() {
        if let Some(found) = trait_provides_method_node(db, t.as_ref(), method_lower, visited) {
            return Some(found);
        }
    }
    None
}

pub fn method_exists_via_db(db: &dyn MirDatabase, fqcn: &str, method_name: &str) -> bool {
    let lower = method_name.to_lowercase();
    let Some(self_node) = db.lookup_class_node(fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    // Direct own method.
    if db
        .lookup_method_node(fqcn, &lower)
        .is_some_and(|m| m.active(db))
    {
        return true;
    }
    // Traits used directly — walk transitively.
    let mut visited_traits: HashSet<String> = HashSet::new();
    for t in self_node.traits(db).iter() {
        if trait_declares_method(db, t.as_ref(), &lower, &mut visited_traits) {
            return true;
        }
    }
    // Ancestor chain (parents, interfaces, traits).
    for ancestor in class_ancestors(db, self_node).0.iter() {
        if db
            .lookup_method_node(ancestor.as_ref(), &lower)
            .is_some_and(|m| m.active(db))
        {
            return true;
        }
        if let Some(anc_node) = db
            .lookup_class_node(ancestor.as_ref())
            .filter(|n| n.active(db))
        {
            if anc_node.is_trait(db) {
                if trait_declares_method(db, ancestor.as_ref(), &lower, &mut visited_traits) {
                    return true;
                }
            } else {
                for t in anc_node.traits(db).iter() {
                    if trait_declares_method(db, t.as_ref(), &lower, &mut visited_traits) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Existence-only sibling of [`trait_provides_method`].  Returns true iff the
/// trait or any sub-trait declares a method named `method_lower` (abstract
/// counts).  Cycle-safe via `visited`.
fn trait_declares_method(
    db: &dyn MirDatabase,
    trait_fqcn: &str,
    method_lower: &str,
    visited: &mut HashSet<String>,
) -> bool {
    if !visited.insert(trait_fqcn.to_string()) {
        return false;
    }
    if db
        .lookup_method_node(trait_fqcn, method_lower)
        .is_some_and(|m| m.active(db))
    {
        return true;
    }
    let Some(node) = db.lookup_class_node(trait_fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    if !node.is_trait(db) {
        return false;
    }
    for t in node.traits(db).iter() {
        if trait_declares_method(db, t.as_ref(), method_lower, visited) {
            return true;
        }
    }
    false
}

/// Walk `fqcn`'s own [`PropertyNode`] then mixins, traits, and ancestors,
/// returning the first active node whose name matches `prop_name`.
/// Mirrors [`Codebase::get_property`]'s walk: own → mixins (recursive) →
/// each ancestor's own + mixins → direct traits' own.  `class_ancestors`
/// already includes parents, interfaces, and direct traits in its returned
/// list, so the ancestor loop covers traits' `own_properties`.
///
/// Property names are case-sensitive in PHP.  Cycle-safe via a per-call
/// `visited_mixins` set; `class_ancestors` itself is cycle-safe.
pub fn lookup_property_in_chain(
    db: &dyn MirDatabase,
    fqcn: &str,
    prop_name: &str,
) -> Option<PropertyNode> {
    let mut visited_mixins: HashSet<String> = HashSet::new();
    lookup_property_in_chain_inner(db, fqcn, prop_name, &mut visited_mixins)
}

fn lookup_property_in_chain_inner(
    db: &dyn MirDatabase,
    fqcn: &str,
    prop_name: &str,
    visited_mixins: &mut HashSet<String>,
) -> Option<PropertyNode> {
    let self_node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;

    // 1. Own property.
    if let Some(node) = db
        .lookup_property_node(fqcn, prop_name)
        .filter(|n| n.active(db))
    {
        return Some(node);
    }
    // 2. Docblock @mixin chains — recurse so each mixin's own walk includes
    //    its own mixins, traits, ancestors.  Cycle-safe via `visited_mixins`.
    for m in self_node.mixins(db).iter() {
        if visited_mixins.insert(m.to_string()) {
            if let Some(node) =
                lookup_property_in_chain_inner(db, m.as_ref(), prop_name, visited_mixins)
            {
                return Some(node);
            }
        }
    }
    // 3. Ancestor chain (parents + interfaces + direct traits).  Each
    //    ancestor may itself have `@mixin` declarations that forward
    //    property access — recurse into those too.
    for ancestor in class_ancestors(db, self_node).0.iter() {
        if let Some(node) = db
            .lookup_property_node(ancestor.as_ref(), prop_name)
            .filter(|n| n.active(db))
        {
            return Some(node);
        }
        if let Some(anc_node) = db
            .lookup_class_node(ancestor.as_ref())
            .filter(|n| n.active(db))
        {
            for m in anc_node.mixins(db).iter() {
                if visited_mixins.insert(m.to_string()) {
                    if let Some(node) =
                        lookup_property_in_chain_inner(db, m.as_ref(), prop_name, visited_mixins)
                    {
                        return Some(node);
                    }
                }
            }
        }
    }
    None
}

/// Returns `true` iff `fqcn` (or any class/interface in its ancestor chain)
/// declares a class constant named `const_name`.  Mirrors
/// [`Codebase::get_class_constant`]'s walk for existence purposes:
/// own → traits → ancestors (incl. interfaces).  `class_ancestors` already
/// includes direct traits and interfaces in its returned list, so a single
/// walk is sufficient.
///
/// Constant names are case-sensitive in PHP.  Cycle-safe via
/// `class_ancestors`'s own cycle recovery.
pub fn class_constant_exists_in_chain(db: &dyn MirDatabase, fqcn: &str, const_name: &str) -> bool {
    if db
        .lookup_class_constant_node(fqcn, const_name)
        .is_some_and(|n| n.active(db))
    {
        return true;
    }
    let Some(class_node) = db.lookup_class_node(fqcn).filter(|n| n.active(db)) else {
        return false;
    };
    for ancestor in class_ancestors(db, class_node).0.iter() {
        if db
            .lookup_class_constant_node(ancestor.as_ref(), const_name)
            .is_some_and(|n| n.active(db))
        {
            return true;
        }
    }
    false
}

/// Predicate variant of [`Codebase::extends_or_implements`] backed by the
/// Salsa db.
///
/// Returns `true` iff `child` is `ancestor`, or `child`'s transitive
/// ancestor list (via [`class_ancestors`]) contains `ancestor`.  For enums
/// the ancestor list is empty by construction (matching
/// `Codebase::ensure_finalized`); membership is answered directly from
/// the enum's directly-declared interfaces and the implicit
/// `UnitEnum` / `BackedEnum` interfaces.
///
/// Unregistered classes return `false` — `ingest_codebase` populates
/// the db before any Pass 2 driver runs, so a class with no active
/// `ClassNode` genuinely doesn't exist.
pub fn extends_or_implements_via_db(db: &dyn MirDatabase, child: &str, ancestor: &str) -> bool {
    if child == ancestor {
        return true;
    }
    let Some(node) = db.lookup_class_node(child).filter(|n| n.active(db)) else {
        return false;
    };
    if node.is_enum(db) {
        // Match `Codebase::extends_or_implements` enum semantics: only
        // directly-declared interfaces participate (no transitive walk),
        // plus the implicit UnitEnum / BackedEnum interfaces.
        if node.interfaces(db).iter().any(|i| i.as_ref() == ancestor) {
            return true;
        }
        if ancestor == "UnitEnum" || ancestor == "\\UnitEnum" {
            return true;
        }
        if (ancestor == "BackedEnum" || ancestor == "\\BackedEnum") && node.is_backed_enum(db) {
            return true;
        }
        return false;
    }
    class_ancestors(db, node)
        .0
        .iter()
        .any(|p| p.as_ref() == ancestor)
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
    /// FQN → GlobalConstantNode handle registry.
    global_constant_nodes: HashMap<Arc<str>, GlobalConstantNode>,
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

    fn lookup_global_constant_node(&self, fqn: &str) -> Option<GlobalConstantNode> {
        self.global_constant_nodes.get(fqn).copied()
    }

    fn class_own_methods(&self, fqcn: &str) -> Vec<MethodNode> {
        self.method_nodes
            .get(fqcn)
            .map(|m| m.values().copied().collect())
            .unwrap_or_default()
    }

    fn class_own_properties(&self, fqcn: &str) -> Vec<PropertyNode> {
        self.property_nodes
            .get(fqcn)
            .map(|m| m.values().copied().collect())
            .unwrap_or_default()
    }

    fn active_class_node_fqcns(&self) -> Vec<Arc<str>> {
        self.class_nodes
            .iter()
            .filter_map(|(fqcn, node)| {
                if node.active(self) {
                    Some(fqcn.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn active_function_node_fqns(&self) -> Vec<Arc<str>> {
        self.function_nodes
            .iter()
            .filter_map(|(fqn, node)| {
                if node.active(self) {
                    Some(fqn.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Field bag for [`MirDb::upsert_class_node`].  Construct with `..Default::default()`
/// to fill in the fields that don't apply to your kind (e.g. interfaces leave
/// `parent`, `traits`, `mixins`, `is_abstract`, etc. at their defaults).
///
/// Per-kind constructors (`for_class` / `for_interface` / `for_trait` /
/// `for_enum`) seed the kind discriminators so the caller only has to populate
/// kind-specific fields.
#[derive(Debug, Clone, Default)]
pub struct ClassNodeFields {
    pub fqcn: Arc<str>,
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub is_abstract: bool,
    pub parent: Option<Arc<str>>,
    pub interfaces: Arc<[Arc<str>]>,
    pub traits: Arc<[Arc<str>]>,
    pub extends: Arc<[Arc<str>]>,
    pub template_params: Arc<[TemplateParam]>,
    pub require_extends: Arc<[Arc<str>]>,
    pub require_implements: Arc<[Arc<str>]>,
    pub is_backed_enum: bool,
    pub mixins: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub enum_scalar_type: Option<Union>,
    pub is_final: bool,
    pub is_readonly: bool,
    pub location: Option<Location>,
    pub extends_type_args: Arc<[Union]>,
    pub implements_type_args: ImplementsTypeArgs,
}

impl ClassNodeFields {
    pub fn for_class(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            ..Self::default()
        }
    }

    pub fn for_interface(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_interface: true,
            ..Self::default()
        }
    }

    pub fn for_trait(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_trait: true,
            ..Self::default()
        }
    }

    pub fn for_enum(fqcn: Arc<str>) -> Self {
        Self {
            fqcn,
            is_enum: true,
            ..Self::default()
        }
    }
}

impl MirDb {
    /// Create or update the `ClassNode` for `fqcn`.
    ///
    /// If a handle already exists, its fields are updated in-place so Salsa
    /// can track the change.  A new handle is created only on first registration.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_class_node(&mut self, fields: ClassNodeFields) -> ClassNode {
        use salsa::Setter as _;
        let ClassNodeFields {
            fqcn,
            is_interface,
            is_trait,
            is_enum,
            is_abstract,
            parent,
            interfaces,
            traits,
            extends,
            template_params,
            require_extends,
            require_implements,
            is_backed_enum,
            mixins,
            deprecated,
            enum_scalar_type,
            is_final,
            is_readonly,
            location,
            extends_type_args,
            implements_type_args,
        } = fields;
        if let Some(&node) = self.class_nodes.get(&fqcn) {
            // Fast-skip: an already-active node whose Salsa-tracked fields
            // match the upsert input.  Bulk re-ingest paths
            // (`ingest_codebase` / `lazy_load_*`) call this for every class
            // on every iteration; without the skip each call fires 13
            // setters, each acquiring the Salsa write lock.  Schema doesn't
            // mutate after Pass 1 (Pass 2 only writes `inferred_return_type`
            // which lives on `Codebase`, not the db), so an active node with
            // matching fields is by construction up to date.
            //
            // Mutation paths (LSP re-analyze) call `deactivate_class_node`
            // first; that flips `active=false`, defeating this guard so the
            // setters run as before.
            if node.active(self)
                && node.is_interface(self) == is_interface
                && node.is_trait(self) == is_trait
                && node.is_enum(self) == is_enum
                && node.is_abstract(self) == is_abstract
                && node.is_backed_enum(self) == is_backed_enum
                && node.parent(self) == parent
                && *node.interfaces(self) == *interfaces
                && *node.traits(self) == *traits
                && *node.extends(self) == *extends
                && *node.template_params(self) == *template_params
                && *node.require_extends(self) == *require_extends
                && *node.require_implements(self) == *require_implements
                && *node.mixins(self) == *mixins
                && node.deprecated(self) == deprecated
                && node.enum_scalar_type(self) == enum_scalar_type
                && node.is_final(self) == is_final
                && node.is_readonly(self) == is_readonly
                && node.location(self) == location
                && *node.extends_type_args(self) == *extends_type_args
                && *node.implements_type_args(self) == *implements_type_args
            {
                return node;
            }
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
            node.set_require_extends(self).to(require_extends);
            node.set_require_implements(self).to(require_implements);
            node.set_is_backed_enum(self).to(is_backed_enum);
            node.set_mixins(self).to(mixins);
            node.set_deprecated(self).to(deprecated);
            node.set_enum_scalar_type(self).to(enum_scalar_type);
            node.set_is_final(self).to(is_final);
            node.set_is_readonly(self).to(is_readonly);
            node.set_location(self).to(location);
            node.set_extends_type_args(self).to(extends_type_args);
            node.set_implements_type_args(self).to(implements_type_args);
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
                require_extends,
                require_implements,
                is_backed_enum,
                mixins,
                deprecated,
                enum_scalar_type,
                is_final,
                is_readonly,
                location,
                extends_type_args,
                implements_type_args,
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
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.short_name(self) == storage.short_name
                && node.is_pure(self) == storage.is_pure
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self) == storage.return_type
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_slice()
                && *node.template_params(self) == *storage.template_params.as_slice()
                && *node.assertions(self) == *storage.assertions.as_slice()
                && *node.throws(self) == *storage.throws.as_slice()
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_short_name(self).to(storage.short_name.clone());
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
            node.set_location(self).to(storage.location.clone());
            node
        } else {
            let node = FunctionNode::new(
                self,
                fqn.clone(),
                storage.short_name.clone(),
                true,
                Arc::from(storage.params.as_slice()),
                storage.return_type.clone(),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.is_pure,
                storage.location.clone(),
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
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_static(self) == storage.is_static
                && node.is_abstract(self) == storage.is_abstract
                && node.is_final(self) == storage.is_final
                && node.is_constructor(self) == storage.is_constructor
                && node.is_pure(self) == storage.is_pure
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self) == storage.return_type
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_slice()
                && *node.template_params(self) == *storage.template_params.as_slice()
                && *node.assertions(self) == *storage.assertions.as_slice()
                && *node.throws(self) == *storage.throws.as_slice()
            {
                return node;
            }
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
            node.set_location(self).to(storage.location.clone());
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
                storage.location.clone(),
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

    /// Deactivate `MethodNode`s for `fqcn` whose lowercased name is not in
    /// `keep_lower`.  Used by `ingest_codebase` to prune stale stub methods
    /// when a user file shadows a bundled-stub class with a different method
    /// set.  Active-only check preserves PR21's fast-skip — already-inactive
    /// nodes don't fire a setter.
    pub fn prune_class_methods(
        &mut self,
        fqcn: &str,
        keep_lower: &std::collections::HashSet<Arc<str>>,
    ) {
        use salsa::Setter as _;
        let candidates: Vec<MethodNode> = self
            .method_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep_lower.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
        }
    }

    /// Deactivate `PropertyNode`s for `fqcn` whose name is not in `keep`.
    pub fn prune_class_properties(
        &mut self,
        fqcn: &str,
        keep: &std::collections::HashSet<Arc<str>>,
    ) {
        use salsa::Setter as _;
        let candidates: Vec<PropertyNode> = self
            .property_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
        }
    }

    /// Deactivate `ClassConstantNode`s for `fqcn` whose name is not in `keep`.
    pub fn prune_class_constants(
        &mut self,
        fqcn: &str,
        keep: &std::collections::HashSet<Arc<str>>,
    ) {
        use salsa::Setter as _;
        let candidates: Vec<ClassConstantNode> = self
            .class_constant_nodes
            .get(fqcn)
            .map(|m| {
                m.iter()
                    .filter(|(k, _)| !keep.contains(k.as_ref()))
                    .map(|(_, n)| *n)
                    .collect()
            })
            .unwrap_or_default();
        for node in candidates {
            if node.active(self) {
                node.set_active(self).to(false);
            }
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
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_static(self) == storage.is_static
                && node.is_readonly(self) == storage.is_readonly
                && node.ty(self) == storage.ty
                && node.location(self) == storage.location
            {
                return;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_static(self).to(storage.is_static);
            node.set_is_readonly(self).to(storage.is_readonly);
            node.set_location(self).to(storage.location.clone());
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
                storage.location.clone(),
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
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_final(self) == storage.is_final
                && node.ty(self) == storage.ty
            {
                return;
            }
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
        use std::collections::HashSet;
        for entry in codebase.classes.iter() {
            let cls = entry.value();
            self.upsert_class_node(ClassNodeFields {
                is_abstract: cls.is_abstract,
                parent: cls.parent.clone(),
                interfaces: Arc::from(cls.interfaces.as_slice()),
                traits: Arc::from(cls.traits.as_slice()),
                template_params: Arc::from(cls.template_params.as_slice()),
                mixins: Arc::from(cls.mixins.as_slice()),
                deprecated: cls.deprecated.clone(),
                is_final: cls.is_final,
                is_readonly: cls.is_readonly,
                location: cls.location.clone(),
                extends_type_args: Arc::from(cls.extends_type_args.as_slice()),
                implements_type_args: Arc::from(
                    cls.implements_type_args
                        .iter()
                        .map(|(iface, args)| (iface.clone(), Arc::from(args.as_slice())))
                        .collect::<Vec<_>>(),
                ),
                ..ClassNodeFields::for_class(cls.fqcn.clone())
            });
            let method_keep: HashSet<Arc<str>> = cls
                .own_methods
                .values()
                .map(|m| Arc::<str>::from(m.name.to_lowercase().as_str()))
                .collect();
            self.prune_class_methods(&cls.fqcn, &method_keep);
            for method in cls.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            let prop_keep: HashSet<Arc<str>> = cls
                .own_properties
                .values()
                .map(|p| p.name.clone())
                .collect();
            self.prune_class_properties(&cls.fqcn, &prop_keep);
            for prop in cls.own_properties.values() {
                self.upsert_property_node(&cls.fqcn, prop);
            }
            let const_keep: HashSet<Arc<str>> =
                cls.own_constants.values().map(|c| c.name.clone()).collect();
            self.prune_class_constants(&cls.fqcn, &const_keep);
            for constant in cls.own_constants.values() {
                self.upsert_class_constant_node(&cls.fqcn, constant);
            }
        }
        for entry in codebase.interfaces.iter() {
            let iface = entry.value();
            self.upsert_class_node(ClassNodeFields {
                extends: Arc::from(iface.extends.as_slice()),
                template_params: Arc::from(iface.template_params.as_slice()),
                location: iface.location.clone(),
                ..ClassNodeFields::for_interface(iface.fqcn.clone())
            });
            let method_keep: HashSet<Arc<str>> = iface
                .own_methods
                .values()
                .map(|m| Arc::<str>::from(m.name.to_lowercase().as_str()))
                .collect();
            self.prune_class_methods(&iface.fqcn, &method_keep);
            for method in iface.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            let const_keep: HashSet<Arc<str>> = iface
                .own_constants
                .values()
                .map(|c| c.name.clone())
                .collect();
            self.prune_class_constants(&iface.fqcn, &const_keep);
            for constant in iface.own_constants.values() {
                self.upsert_class_constant_node(&iface.fqcn, constant);
            }
        }
        for entry in codebase.traits.iter() {
            let tr = entry.value();
            self.upsert_class_node(ClassNodeFields {
                traits: Arc::from(tr.traits.as_slice()),
                template_params: Arc::from(tr.template_params.as_slice()),
                require_extends: Arc::from(tr.require_extends.as_slice()),
                require_implements: Arc::from(tr.require_implements.as_slice()),
                location: tr.location.clone(),
                ..ClassNodeFields::for_trait(tr.fqcn.clone())
            });
            let method_keep: HashSet<Arc<str>> = tr
                .own_methods
                .values()
                .map(|m| Arc::<str>::from(m.name.to_lowercase().as_str()))
                .collect();
            self.prune_class_methods(&tr.fqcn, &method_keep);
            for method in tr.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            let prop_keep: HashSet<Arc<str>> =
                tr.own_properties.values().map(|p| p.name.clone()).collect();
            self.prune_class_properties(&tr.fqcn, &prop_keep);
            for prop in tr.own_properties.values() {
                self.upsert_property_node(&tr.fqcn, prop);
            }
            let const_keep: HashSet<Arc<str>> =
                tr.own_constants.values().map(|c| c.name.clone()).collect();
            self.prune_class_constants(&tr.fqcn, &const_keep);
            for constant in tr.own_constants.values() {
                self.upsert_class_constant_node(&tr.fqcn, constant);
            }
        }
        for entry in codebase.enums.iter() {
            let en = entry.value();
            self.upsert_class_node(ClassNodeFields {
                interfaces: Arc::from(en.interfaces.as_slice()),
                is_backed_enum: en.scalar_type.is_some(),
                enum_scalar_type: en.scalar_type.clone(),
                location: en.location.clone(),
                ..ClassNodeFields::for_enum(en.fqcn.clone())
            });
            let mut method_keep: HashSet<Arc<str>> = en
                .own_methods
                .values()
                .map(|m| Arc::<str>::from(m.name.to_lowercase().as_str()))
                .collect();
            method_keep.insert(Arc::from("cases"));
            if en.scalar_type.is_some() {
                method_keep.insert(Arc::from("from"));
                method_keep.insert(Arc::from("tryfrom"));
            }
            self.prune_class_methods(&en.fqcn, &method_keep);
            for method in en.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            // Synthesize PHP 8.1 implicit enum methods (`cases`, plus `from` /
            // `tryFrom` for backed enums) so `lookup_method_node` finds them
            // — mirrors the on-the-fly synthesis in `Codebase::get_method`.
            // Only register when the user hasn't shadowed the name (PHP forbids
            // it but be defensive).
            let synth_method = |name: &str| mir_codebase::storage::MethodStorage {
                fqcn: en.fqcn.clone(),
                name: Arc::from(name),
                params: vec![],
                return_type: Some(Union::mixed()),
                inferred_return_type: None,
                visibility: Visibility::Public,
                is_static: true,
                is_abstract: false,
                is_constructor: false,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                is_final: false,
                is_internal: false,
                is_pure: false,
                deprecated: None,
                location: None,
            };
            let already = |name: &str| {
                en.own_methods
                    .keys()
                    .any(|k| k.as_ref().eq_ignore_ascii_case(name))
            };
            if !already("cases") {
                self.upsert_method_node(&synth_method("cases"));
            }
            if en.scalar_type.is_some() {
                if !already("from") {
                    self.upsert_method_node(&synth_method("from"));
                }
                if !already("tryFrom") {
                    self.upsert_method_node(&synth_method("tryFrom"));
                }
            }
            let mut const_keep: HashSet<Arc<str>> =
                en.own_constants.values().map(|c| c.name.clone()).collect();
            for case in en.cases.values() {
                const_keep.insert(case.name.clone());
            }
            self.prune_class_constants(&en.fqcn, &const_keep);
            for constant in en.own_constants.values() {
                self.upsert_class_constant_node(&en.fqcn, constant);
            }
            for case in en.cases.values() {
                let case_const = ConstantStorage {
                    name: case.name.clone(),
                    ty: mir_types::Union::mixed(),
                    visibility: None,
                    is_final: false,
                    location: case.location.clone(),
                };
                self.upsert_class_constant_node(&en.fqcn, &case_const);
            }
        }
        for entry in codebase.functions.iter() {
            self.upsert_function_node(entry.value());
        }
        for entry in codebase.constants.iter() {
            self.upsert_global_constant_node(entry.key().clone(), entry.value().clone());
        }
    }

    /// Create or update the `GlobalConstantNode` for `fqn`.
    pub fn upsert_global_constant_node(&mut self, fqn: Arc<str>, ty: Union) -> GlobalConstantNode {
        use salsa::Setter as _;
        if let Some(&node) = self.global_constant_nodes.get(&fqn) {
            // Fast-skip identical re-ingest — see `upsert_class_node` for rationale.
            if node.active(self) && node.ty(self) == ty {
                return node;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(ty);
            node
        } else {
            let node = GlobalConstantNode::new(self, fqn.clone(), true, ty);
            self.global_constant_nodes.insert(fqn, node);
            node
        }
    }

    /// Mark the `GlobalConstantNode` for `fqn` as inactive.
    pub fn deactivate_global_constant_node(&mut self, fqn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.global_constant_nodes.get(fqn) {
            node.set_active(self).to(false);
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
// S4 Step 1: analyze_file accumulators + tracked-query skeleton
// ---------------------------------------------------------------------------
//
// First step toward S4 (issues + reference locations as Salsa accumulators,
// `analyze_file` as a tracked query).  This step is purely additive:
//
//   1. Defines `IssueAccumulator` and `RefLocAccumulator` salsa accumulator
//      types — push targets for analyzer-emitted issues and reference-index
//      entries during tracked-query evaluation.
//   2. Defines `analyze_file` as a tracked-query stub keyed on a
//      `(SourceFile, AnalyzeFileInput)` pair.  The stub does NOT perform
//      analysis — it accumulates only the parse errors (a strict subset of
//      what `collect_file_definitions` already produces, so semantics are
//      unchanged).  The full analyzer wiring follows in subsequent S4 PRs.
//
// Nothing in this module is wired into the batch (`analyze`) or LSP
// (`re_analyze_file`) paths yet.  Behavior change: zero.

/// Salsa accumulator carrying analyzer-emitted issues.  In the eventual
/// S4 design, every site that today calls `IssueBuffer::add` / `Vec::push`
/// from inside a tracked query will instead call
/// `IssueAccumulator(issue).accumulate(db)`, and `re_analyze_file` will read
/// the accumulated issues for the file with
/// `analyze_file::accumulated::<IssueAccumulator>(db, file, ...)`.
#[salsa::accumulator]
#[derive(Clone, Debug)]
pub struct IssueAccumulator(pub Issue);

/// Reference-index entry as produced during analysis.  Mirrors the tuple
/// shape that `Codebase::record_ref` accepts:
///
/// - `symbol_key`: interner-bound string (`"fn:foo"`, `"cls:Foo"`,
///   `"prop:Foo::$bar"`, `"cnst:Foo::BAR"`, `"meth:Foo::bar"` — same keys
///   `Codebase::mark_*_referenced_at` use).
/// - `file`: the file in which the reference appears.
/// - `(line, col_start, col_end)`: span within the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefLoc {
    pub symbol_key: Arc<str>,
    pub file: Arc<str>,
    pub line: u32,
    pub col_start: u16,
    pub col_end: u16,
}

/// Salsa accumulator carrying reference-index entries.  In the eventual
/// S4 design this replaces the `Codebase::mark_*_referenced_at` side
/// effects: instead of mutating the codebase's reference index inside a
/// tracked query (which Salsa cannot observe), the analyzer pushes
/// `RefLocAccumulator(loc)` and the consumer (LSP / dead-code) reads via
/// `analyze_file::accumulated::<RefLocAccumulator>(db, …)`.
#[salsa::accumulator]
#[derive(Clone, Debug)]
pub struct RefLocAccumulator(pub RefLoc);

/// Salsa tracked-query input for `analyze_file`.  Carries the analysis
/// parameters that aren't already captured by `SourceFile` itself.  Kept
/// minimal in this PR; subsequent PRs in the S4 chain will extend it as
/// the query body grows to call the full analyzer pipeline.
#[salsa::input]
pub struct AnalyzeFileInput {
    /// Resolved PHP version (`"8.1"`, `"8.2"`, …) used by the analyzer.
    /// Mirrors `ProjectAnalyzer::resolved_php_version`.
    pub php_version: Arc<str>,
}

/// Tracked-query skeleton for `analyze_file`.
///
/// **Current behavior (S4 step 1):** parses the file and emits parse-error
/// issues via `IssueAccumulator`.  Does NOT call into Pass 2 / the
/// statement / expression analyzer; full body analysis stays in
/// `Pass2Driver` until later S4 PRs migrate it.
///
/// The query exists at this stage to:
/// - validate that accumulators compile and accumulate against the
///   concrete `MirDb`,
/// - give subsequent PRs a stable signature to extend without churning
///   the public surface of `db.rs` again,
/// - provide a readable test of the accumulator round-trip
///   (`accumulate` → `accumulated::<…>(db, …)`).
#[salsa::tracked]
pub fn analyze_file(db: &dyn MirDatabase, file: SourceFile, _input: AnalyzeFileInput) {
    use salsa::Accumulator as _;
    let path = file.path(db);
    let text = file.text(db);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &text);

    for err in &parsed.errors {
        let issue = Issue::new(
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
        );
        IssueAccumulator(issue).accumulate(db);
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
        db.upsert_class_node(ClassNodeFields {
            is_interface,
            parent,
            extends,
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
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
    fn analyze_file_accumulates_parse_errors() {
        let db = MirDb::default();
        // Unterminated string literal — guaranteed parser diagnostic.
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/parse_err.php"),
            Arc::from("<?php $x = \"unterminated"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(
            !issues.is_empty(),
            "expected parse error to surface as accumulated IssueAccumulator"
        );
        assert!(matches!(
            issues[0].0.kind,
            mir_issues::IssueKind::ParseError { .. }
        ));
    }

    #[test]
    fn analyze_file_clean_input_accumulates_nothing() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/clean.php"),
            Arc::from("<?php class Foo {}"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        let refs: Vec<&RefLocAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(issues.is_empty());
        assert!(refs.is_empty());
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

    // -----------------------------------------------------------------
    // Helpers for method-related fixtures
    // -----------------------------------------------------------------

    fn upsert_class_with_traits(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        traits: &[&str],
        is_interface: bool,
        is_trait: bool,
    ) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            is_interface,
            is_trait,
            parent,
            traits: Arc::from(
                traits
                    .iter()
                    .map(|t| Arc::<str>::from(*t))
                    .collect::<Vec<_>>(),
            ),
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
    }

    fn upsert_method(db: &mut MirDb, fqcn: &str, name: &str, is_abstract: bool) -> MethodNode {
        let storage = MethodStorage {
            name: Arc::from(name),
            fqcn: Arc::from(fqcn),
            params: vec![],
            return_type: None,
            inferred_return_type: None,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract,
            is_final: false,
            is_constructor: name == "__construct",
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_internal: false,
            is_pure: false,
            location: None,
        };
        db.upsert_method_node(&storage)
    }

    fn upsert_enum(db: &mut MirDb, fqcn: &str, interfaces: &[&str], is_backed: bool) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            interfaces: Arc::from(
                interfaces
                    .iter()
                    .map(|i| Arc::<str>::from(*i))
                    .collect::<Vec<_>>(),
            ),
            is_backed_enum: is_backed,
            ..ClassNodeFields::for_enum(Arc::from(fqcn))
        })
    }

    // -----------------------------------------------------------------
    // method_exists_via_db
    // -----------------------------------------------------------------

    #[test]
    fn method_exists_via_db_finds_own_method() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "bar", false);
        assert!(method_exists_via_db(&db, "Foo", "bar"));
        assert!(!method_exists_via_db(&db, "Foo", "missing"));
    }

    #[test]
    fn method_exists_via_db_walks_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_method(&mut db, "Base", "inherited", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(method_exists_via_db(&db, "Child", "inherited"));
    }

    #[test]
    fn method_exists_via_db_walks_traits_transitively() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "InnerTrait", None, &[], false, true);
        upsert_method(&mut db, "InnerTrait", "deep_trait_method", false);
        upsert_class_with_traits(&mut db, "OuterTrait", None, &["InnerTrait"], false, true);
        upsert_class_with_traits(&mut db, "Foo", None, &["OuterTrait"], false, false);
        assert!(method_exists_via_db(&db, "Foo", "deep_trait_method"));
    }

    #[test]
    fn method_exists_via_db_is_case_insensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "doStuff", false);
        // Stored with original case; lookup must lowercase internally.
        assert!(method_exists_via_db(&db, "Foo", "DoStuff"));
        assert!(method_exists_via_db(&db, "Foo", "DOSTUFF"));
    }

    #[test]
    fn method_exists_via_db_unknown_class_returns_false() {
        let db = MirDb::default();
        assert!(!method_exists_via_db(&db, "Nope", "anything"));
    }

    #[test]
    fn method_exists_via_db_inactive_class_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "bar", false);
        db.deactivate_class_node("Foo");
        assert!(!method_exists_via_db(&db, "Foo", "bar"));
    }

    #[test]
    fn method_exists_via_db_finds_abstract_methods() {
        // Existence-only: abstracts count.  This is the difference vs.
        // method_is_concretely_implemented.
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "abstr", true);
        assert!(method_exists_via_db(&db, "Foo", "abstr"));
    }

    // -----------------------------------------------------------------
    // method_is_concretely_implemented
    // -----------------------------------------------------------------

    #[test]
    fn method_is_concretely_implemented_skips_abstract() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "abstr", true);
        assert!(!method_is_concretely_implemented(&db, "Foo", "abstr"));
    }

    #[test]
    fn method_is_concretely_implemented_finds_concrete_in_trait() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "MyTrait", None, &[], false, true);
        upsert_method(&mut db, "MyTrait", "provided", false);
        upsert_class_with_traits(&mut db, "Foo", None, &["MyTrait"], false, false);
        assert!(method_is_concretely_implemented(&db, "Foo", "provided"));
    }

    #[test]
    fn method_is_concretely_implemented_skips_interface_definitions() {
        // Interfaces don't supply implementations, regardless of how
        // their methods are stored.
        let mut db = MirDb::default();
        upsert_class(&mut db, "I", None, Arc::from([]), true);
        upsert_method(&mut db, "I", "m", false);
        upsert_class(&mut db, "C", None, Arc::from([Arc::from("I")]), false);
        // C "implements" I but has no own implementation.
        assert!(!method_is_concretely_implemented(&db, "C", "m"));
    }

    // -----------------------------------------------------------------
    // extends_or_implements_via_db
    // -----------------------------------------------------------------

    #[test]
    fn extends_or_implements_via_db_self_match() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        assert!(extends_or_implements_via_db(&db, "Foo", "Foo"));
    }

    #[test]
    fn extends_or_implements_via_db_transitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Animal", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Mammal",
            Some(Arc::from("Animal")),
            Arc::from([]),
            false,
        );
        upsert_class(
            &mut db,
            "Dog",
            Some(Arc::from("Mammal")),
            Arc::from([]),
            false,
        );
        assert!(extends_or_implements_via_db(&db, "Dog", "Animal"));
        assert!(extends_or_implements_via_db(&db, "Dog", "Mammal"));
        assert!(!extends_or_implements_via_db(&db, "Animal", "Dog"));
    }

    #[test]
    fn extends_or_implements_via_db_unknown_returns_false() {
        let db = MirDb::default();
        assert!(!extends_or_implements_via_db(&db, "Nope", "Foo"));
    }

    #[test]
    fn extends_or_implements_via_db_unit_enum_implicit() {
        let mut db = MirDb::default();
        upsert_enum(&mut db, "Status", &[], false);
        assert!(extends_or_implements_via_db(&db, "Status", "UnitEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "\\UnitEnum"));
        // Pure enum is NOT a BackedEnum.
        assert!(!extends_or_implements_via_db(&db, "Status", "BackedEnum"));
    }

    #[test]
    fn extends_or_implements_via_db_backed_enum_implicit() {
        let mut db = MirDb::default();
        upsert_enum(&mut db, "Status", &[], true);
        assert!(extends_or_implements_via_db(&db, "Status", "UnitEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "BackedEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "\\BackedEnum"));
    }

    #[test]
    fn extends_or_implements_via_db_enum_declared_interface() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Stringable", None, Arc::from([]), true);
        upsert_enum(&mut db, "Status", &["Stringable"], false);
        assert!(extends_or_implements_via_db(&db, "Status", "Stringable"));
    }

    // -----------------------------------------------------------------
    // has_unknown_ancestor_via_db
    // -----------------------------------------------------------------

    #[test]
    fn has_unknown_ancestor_via_db_clean_chain_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(!has_unknown_ancestor_via_db(&db, "Child"));
    }

    #[test]
    fn has_unknown_ancestor_via_db_missing_parent_returns_true() {
        let mut db = MirDb::default();
        // Child claims to extend Missing, but Missing isn't registered.
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Missing")),
            Arc::from([]),
            false,
        );
        assert!(has_unknown_ancestor_via_db(&db, "Child"));
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
        db.upsert_class_node(ClassNodeFields {
            template_params: Arc::from([tp.clone()]),
            ..ClassNodeFields::for_class(Arc::from("Box"))
        });
        let got = class_template_params_via_db(&db, "Box").expect("registered");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name.as_ref(), "T");

        assert!(class_template_params_via_db(&db, "Missing").is_none());
        db.deactivate_class_node("Box");
        assert!(class_template_params_via_db(&db, "Box").is_none());
    }

    // -----------------------------------------------------------------
    // lookup_method_in_chain
    // -----------------------------------------------------------------

    fn upsert_class_with_mixins(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        mixins: &[&str],
    ) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            parent,
            mixins: Arc::from(
                mixins
                    .iter()
                    .map(|m| Arc::<str>::from(*m))
                    .collect::<Vec<_>>(),
            ),
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
    }

    #[test]
    fn lookup_method_in_chain_finds_own_then_ancestor() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_method(&mut db, "Base", "shared", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        upsert_method(&mut db, "Child", "shared", false);
        // Own wins over ancestor.
        let found = lookup_method_in_chain(&db, "Child", "shared").expect("own");
        assert_eq!(found.fqcn(&db).as_ref(), "Child");
        // Inherited-only resolves to ancestor.
        upsert_method(&mut db, "Base", "only_in_base", false);
        let found = lookup_method_in_chain(&db, "Child", "only_in_base").expect("ancestor");
        assert_eq!(found.fqcn(&db).as_ref(), "Base");
    }

    #[test]
    fn lookup_method_in_chain_walks_trait_of_traits() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "InnerTrait", None, &[], false, true);
        upsert_method(&mut db, "InnerTrait", "deep", false);
        upsert_class_with_traits(&mut db, "OuterTrait", None, &["InnerTrait"], false, true);
        upsert_class_with_traits(&mut db, "Foo", None, &["OuterTrait"], false, false);
        let found = lookup_method_in_chain(&db, "Foo", "deep").expect("transitive trait");
        assert_eq!(found.fqcn(&db).as_ref(), "InnerTrait");
    }

    #[test]
    fn lookup_method_in_chain_walks_mixins() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "MixinTarget", None, Arc::from([]), false);
        upsert_method(&mut db, "MixinTarget", "magic", false);
        upsert_class_with_mixins(&mut db, "Host", None, &["MixinTarget"]);
        let found = lookup_method_in_chain(&db, "Host", "magic").expect("via @mixin");
        assert_eq!(found.fqcn(&db).as_ref(), "MixinTarget");
    }

    #[test]
    fn lookup_method_in_chain_mixin_cycle_does_not_hang() {
        let mut db = MirDb::default();
        // A → B → A (mutual @mixin); neither defines the method.
        upsert_class_with_mixins(&mut db, "A", None, &["B"]);
        upsert_class_with_mixins(&mut db, "B", None, &["A"]);
        assert!(lookup_method_in_chain(&db, "A", "missing").is_none());
    }

    #[test]
    fn lookup_method_in_chain_is_case_insensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "doStuff", false);
        assert!(lookup_method_in_chain(&db, "Foo", "DOSTUFF").is_some());
        assert!(lookup_method_in_chain(&db, "Foo", "dostuff").is_some());
    }

    #[test]
    fn lookup_method_in_chain_unknown_returns_none() {
        let db = MirDb::default();
        assert!(lookup_method_in_chain(&db, "Nope", "anything").is_none());
    }

    // -----------------------------------------------------------------
    // lookup_property_in_chain
    // -----------------------------------------------------------------

    fn upsert_property(db: &mut MirDb, fqcn: &str, name: &str, is_readonly: bool) -> PropertyNode {
        let storage = PropertyStorage {
            name: Arc::from(name),
            ty: None,
            inferred_ty: None,
            visibility: Visibility::Public,
            is_static: false,
            is_readonly,
            default: None,
            location: None,
        };
        let owner = Arc::<str>::from(fqcn);
        db.upsert_property_node(&owner, &storage);
        db.lookup_property_node(fqcn, name).expect("registered")
    }

    #[test]
    fn lookup_property_in_chain_own_then_ancestor() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_property(&mut db, "Base", "x", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        // Inherited resolves to Base.
        let found = lookup_property_in_chain(&db, "Child", "x").expect("ancestor");
        assert_eq!(found.fqcn(&db).as_ref(), "Base");
        // Own override wins.
        upsert_property(&mut db, "Child", "x", true);
        let found = lookup_property_in_chain(&db, "Child", "x").expect("own");
        assert_eq!(found.fqcn(&db).as_ref(), "Child");
        assert!(found.is_readonly(&db));
    }

    #[test]
    fn lookup_property_in_chain_walks_mixins() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "MixinTarget", None, Arc::from([]), false);
        upsert_property(&mut db, "MixinTarget", "exposed", false);
        upsert_class_with_mixins(&mut db, "Host", None, &["MixinTarget"]);
        let found = lookup_property_in_chain(&db, "Host", "exposed").expect("via @mixin");
        assert_eq!(found.fqcn(&db).as_ref(), "MixinTarget");
    }

    #[test]
    fn lookup_property_in_chain_mixin_cycle_does_not_hang() {
        let mut db = MirDb::default();
        upsert_class_with_mixins(&mut db, "A", None, &["B"]);
        upsert_class_with_mixins(&mut db, "B", None, &["A"]);
        assert!(lookup_property_in_chain(&db, "A", "missing").is_none());
    }

    #[test]
    fn lookup_property_in_chain_is_case_sensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_property(&mut db, "Foo", "myProp", false);
        assert!(lookup_property_in_chain(&db, "Foo", "myProp").is_some());
        // Property names are case-sensitive in PHP.
        assert!(lookup_property_in_chain(&db, "Foo", "MyProp").is_none());
    }

    #[test]
    fn lookup_property_in_chain_inactive_returns_none() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_property(&mut db, "Foo", "x", false);
        db.deactivate_class_node("Foo");
        assert!(lookup_property_in_chain(&db, "Foo", "x").is_none());
    }

    // -----------------------------------------------------------------
    // class_constant_exists_in_chain
    // -----------------------------------------------------------------

    fn upsert_constant(db: &mut MirDb, fqcn: &str, name: &str) {
        let storage = ConstantStorage {
            name: Arc::from(name),
            ty: mir_types::Union::mixed(),
            visibility: None,
            is_final: false,
            location: None,
        };
        let owner = Arc::<str>::from(fqcn);
        db.upsert_class_constant_node(&owner, &storage);
    }

    #[test]
    fn class_constant_exists_in_chain_finds_own() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_constant(&mut db, "Foo", "MAX");
        assert!(class_constant_exists_in_chain(&db, "Foo", "MAX"));
        assert!(!class_constant_exists_in_chain(&db, "Foo", "MIN"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_constant(&mut db, "Base", "VERSION");
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(class_constant_exists_in_chain(&db, "Child", "VERSION"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_interface() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "I", None, Arc::from([]), true);
        upsert_constant(&mut db, "I", "TYPE");
        // A class that implements I — interfaces go in the `interfaces`
        // slot, not the `extends` slot which is interface-only.
        db.upsert_class_node(ClassNodeFields {
            interfaces: Arc::from([Arc::from("I")]),
            ..ClassNodeFields::for_class(Arc::from("Impl"))
        });
        assert!(class_constant_exists_in_chain(&db, "Impl", "TYPE"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_direct_trait() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "T", None, &[], false, true);
        upsert_constant(&mut db, "T", "FROM_TRAIT");
        upsert_class_with_traits(&mut db, "Foo", None, &["T"], false, false);
        assert!(class_constant_exists_in_chain(&db, "Foo", "FROM_TRAIT"));
    }

    #[test]
    fn class_constant_exists_in_chain_unknown_class_returns_false() {
        let db = MirDb::default();
        assert!(!class_constant_exists_in_chain(&db, "Nope", "ANY"));
    }

    #[test]
    fn class_constant_exists_in_chain_inactive_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_constant(&mut db, "Foo", "X");
        db.deactivate_class_node("Foo");
        db.deactivate_class_constants("Foo");
        assert!(!class_constant_exists_in_chain(&db, "Foo", "X"));
    }
}
