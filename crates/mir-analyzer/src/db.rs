use std::collections::{HashMap, HashSet};

use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_codebase::storage::{
    Assertion, ConstantStorage, FnParam, FunctionStorage, Location, MethodStorage, PropertyStorage,
    TemplateParam, Visibility,
};
use mir_codebase::StubSlice;
use mir_issues::Issue;
use mir_types::Union;

use crate::pass2::Pass2Driver;
use crate::PhpVersion;

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
}

// SourceFile input (S1)

/// Source file registered as a Salsa input.
/// Setting `text` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    pub text: Arc<str>,
}

// FileDefinitions (S1)

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

// ClassNode input (S2)

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
    /// `class_ancestors` query (it returns empty for traits), but registering
    /// them as `ClassNode`s lets callers answer `type_exists`-style questions
    /// through the db.
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
    /// Used by `lookup_method_in_chain` for delegated magic-method lookup.
    /// Empty for interfaces, traits, and enums (mixin is a class-only
    /// docblock concept).
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
/// `MirDb::ingest_stub_slice`, so a `None` here means the type genuinely
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
/// nodes.  After `MirDb::ingest_stub_slice` has been called for all
/// collected slices, this is the authoritative answer — bundled and user
/// types are both mirrored.
pub fn type_exists_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    db.lookup_class_node(fqcn).is_some_and(|n| n.active(db))
}

pub fn function_exists_via_db(db: &dyn MirDatabase, fqn: &str) -> bool {
    db.lookup_function_node(fqn).is_some_and(|n| n.active(db))
}

pub fn constant_exists_via_db(db: &dyn MirDatabase, fqn: &str) -> bool {
    db.lookup_global_constant_node(fqn)
        .is_some_and(|n| n.active(db))
}

pub fn resolve_name_via_db(db: &dyn MirDatabase, file: &str, name: &str) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_string();
    }

    let lower = name.to_ascii_lowercase();
    if matches!(lower.as_str(), "self" | "static" | "parent") {
        return name.to_string();
    }

    if name.contains('\\') {
        if let Some(imports) = (!name.starts_with('\\')).then(|| db.file_imports(file)) {
            if let Some((first, rest)) = name.split_once('\\') {
                if let Some(base) = imports.get(first) {
                    return format!("{base}\\{rest}");
                }
            }
        }
        if type_exists_via_db(db, name) {
            return name.to_string();
        }
        if let Some(ns) = db.file_namespace(file) {
            let qualified = format!("{}\\{}", ns, name);
            if type_exists_via_db(db, &qualified) {
                return qualified;
            }
        }
        return name.to_string();
    }

    let imports = db.file_imports(file);
    if let Some(fqcn) = imports.get(name) {
        return fqcn.clone();
    }
    if let Some((_, fqcn)) = imports
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
    {
        return fqcn.clone();
    }
    if let Some(ns) = db.file_namespace(file) {
        return format!("{}\\{}", ns, name);
    }
    name.to_string()
}

/// Return the declared `@template` parameters for `fqcn` from an active
/// `ClassNode`, if one is registered.  Returns `None` for unregistered
/// or inactive nodes.  Authoritative after all collected slices have been
/// fed through `ingest_stub_slice`.
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
    let mut visited: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();
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

// FunctionNode input (S5-PR2)

/// Salsa input representing a single global function.
///
/// `inferred_return_type` is the Pass-2-derived return type, populated
/// per-function by the priming sweep.  It is committed to Salsa serially
/// after the parallel sweep returns (so worker db clones have dropped
/// and `Storage::cancel_others` sees strong-count==1).  The buffer-and-
/// commit pattern lives in [`InferredReturnTypes`] and
/// [`MirDb::commit_inferred_return_types`].
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
    pub return_type: Option<Arc<Union>>,
    pub inferred_return_type: Option<Arc<Union>>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
    /// Source location of the declaration.  `None` for functions registered
    /// without a known origin (e.g. some legacy test fixtures).
    pub location: Option<Location>,
}

// MethodNode input (S5-PR3)

/// Salsa input representing a single method or interface/trait method.
///
/// `inferred_return_type` is the Pass-2-derived return type, populated per
/// method by the priming sweep.  Committed to Salsa serially after the
/// parallel sweep returns; see [`FunctionNode`] for the buffer-and-commit
/// pattern that resolves the historical "S3 deadlock".
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
    pub return_type: Option<Arc<Union>>,
    pub inferred_return_type: Option<Arc<Union>>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub is_internal: bool,
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

// PropertyNode input (S5-PR4)

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

// ClassConstantNode input (S5-PR4)

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
    /// Source location of the declaration.  Mirrors `ConstantStorage::location`
    /// for class/interface/trait constants, and `EnumCaseStorage::location` for
    /// enum cases.  `None` for nodes registered without a source span.
    pub location: Option<Location>,
}

// GlobalConstantNode input (S5-PR47)

/// Salsa input representing a global PHP constant (e.g. `PHP_EOL`).
/// Mirrors `Codebase::constants`.
#[salsa::input]
pub struct GlobalConstantNode {
    pub fqn: Arc<str>,
    pub active: bool,
    pub ty: Union,
}

// Ancestors return type (S2)

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

// class_ancestors tracked query (S2)

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
    // - Enums: enum membership questions go through
    //   `extends_or_implements_via_db`, which reads `interfaces` /
    //   `is_backed_enum` directly.
    // - Traits: trait-of-trait walking is handled by
    //   `method_is_concretely_implemented` / `trait_provides_method`
    //   directly via the `traits` field.
    // Do not lift either short-circuit without also auditing every caller
    // of `class_ancestors`.
    if node.is_enum(db) || node.is_trait(db) {
        return Ancestors(vec![]);
    }

    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();

    let add =
        |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut rustc_hash::FxHashSet<Arc<str>>| {
            if seen.insert(fqcn.clone()) {
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

/// Predicate: does `fqcn` have any registered ancestor that lacks a
/// `ClassNode` in the db?
///
/// `ingest_stub_slice` mirrors bundled stubs, user stubs, and PSR-4
/// lazy-loaded definitions into the db before any Pass 2 driver runs, so
/// a class with no active `ClassNode` is one that genuinely doesn't
/// exist — and an unknown class trivially has no known ancestors.
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
    let mut visited_traits: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
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
    visited: &mut rustc_hash::FxHashSet<String>,
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
    let mut visited_mixins: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
    lookup_method_in_chain_inner(db, fqcn, &method_name.to_lowercase(), &mut visited_mixins)
}

fn lookup_method_in_chain_inner(
    db: &dyn MirDatabase,
    fqcn: &str,
    lower: &str,
    visited_mixins: &mut rustc_hash::FxHashSet<String>,
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
    let mut visited_traits: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
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
    visited: &mut rustc_hash::FxHashSet<String>,
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
    let mut visited_traits: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
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
    visited: &mut rustc_hash::FxHashSet<String>,
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
    let mut visited_mixins: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
    lookup_property_in_chain_inner(db, fqcn, prop_name, &mut visited_mixins)
}

fn lookup_property_in_chain_inner(
    db: &dyn MirDatabase,
    fqcn: &str,
    prop_name: &str,
    visited_mixins: &mut rustc_hash::FxHashSet<String>,
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

/// Look up the source location of a class member (method, property, or
/// class/interface/trait/enum constant including enum cases).  Walks the
/// inheritance chain via the same helpers used by analyzer call sites
/// (`lookup_method_in_chain`, `lookup_property_in_chain`,
/// `class_ancestors` for constants), so members defined on an ancestor
/// are still found.  Returns `None` if no member with that name exists,
/// or if the member exists but has no recorded location (e.g. a
/// synthesized enum implicit method).
pub fn member_location_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
    member_name: &str,
) -> Option<Location> {
    if let Some(node) = lookup_method_in_chain(db, fqcn, member_name) {
        if let Some(loc) = node.location(db) {
            return Some(loc);
        }
    }
    if let Some(node) = lookup_property_in_chain(db, fqcn, member_name) {
        if let Some(loc) = node.location(db) {
            return Some(loc);
        }
    }
    // Class/interface/trait/enum constants and enum cases.
    if let Some(node) = db
        .lookup_class_constant_node(fqcn, member_name)
        .filter(|n| n.active(db))
    {
        if let Some(loc) = node.location(db) {
            return Some(loc);
        }
    }
    let class_node = db.lookup_class_node(fqcn).filter(|n| n.active(db))?;
    for ancestor in class_ancestors(db, class_node).0.iter() {
        if let Some(node) = db
            .lookup_class_constant_node(ancestor.as_ref(), member_name)
            .filter(|n| n.active(db))
        {
            if let Some(loc) = node.location(db) {
                return Some(loc);
            }
        }
    }
    None
}

/// Predicate variant of [`Codebase::extends_or_implements`] backed by the
/// Salsa db.
///
/// Returns `true` iff `child` is `ancestor`, or `child`'s transitive
/// ancestor list (via [`class_ancestors`]) contains `ancestor`.  For enums
/// the ancestor list is empty by construction; membership is answered
/// directly from the enum's directly-declared interfaces and the implicit
/// `UnitEnum` / `BackedEnum` interfaces.
///
/// Unregistered classes return `false` — `ingest_stub_slice` populates
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
        // Enum semantics: only directly-declared interfaces participate
        // (no transitive walk), plus the implicit UnitEnum / BackedEnum
        // interfaces.
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

// collect_file_definitions tracked query (S1)

/// Uncached version of collect_file_definitions for bulk operations like vendor
/// collection, where we don't need Salsa to cache the intermediate StubSlice
/// results. This avoids holding Arc<StubSlice> in Salsa's query cache after
/// ingestion.
pub fn collect_file_definitions_uncached(
    db: &dyn MirDatabase,
    file: SourceFile,
) -> FileDefinitions {
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

#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    collect_file_definitions_uncached(db, file)
}

// MirDb concrete database

/// Concrete in-process Salsa database.
///
/// `Clone` is required for parallel batch analysis: salsa's supported
/// pattern for sharing a db across threads is to give each worker its
/// own clone (each clone gets a fresh `ZalsaLocal`, sharing the
/// underlying memoization storage).  Sharing `&MirDb` across threads is
/// **not** supported because `salsa::Database: Send` (not `Sync`).
type MemberRegistry<V> = Arc<FxHashMap<Arc<str>, FxHashMap<Arc<str>, V>>>;
type ReferenceLocations =
    Arc<std::sync::Mutex<FxHashMap<Arc<str>, Vec<(Arc<str>, u32, u16, u16)>>>>;

#[salsa::db]
#[derive(Default, Clone)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
    // Keep registries behind `Arc`s so `MirDb::clone()` stays cheap for
    // parallel analysis workers. The salsa storage is already shared by clone;
    // these maps only hold stable input handles, so copy-on-write insertion is
    // enough for the canonical mutable db paths.
    /// FQCN → ClassNode handle registry (not tracked by Salsa; see
    /// `lookup_class_node` for the rationale). Keys are canonical FQCNs;
    /// case-insensitive lookups go through `class_node_keys_lower`.
    class_nodes: Arc<FxHashMap<Arc<str>, ClassNode>>,
    /// Lowercased FQCN → canonical FQCN. Maintained in lockstep with
    /// `class_nodes` so callers can resolve PHP's case-insensitive class
    /// names (`new arrayobject()` → `ArrayObject`).
    class_node_keys_lower: Arc<FxHashMap<String, Arc<str>>>,
    /// FQN → FunctionNode handle registry. Keys are canonical FQNs;
    /// case-insensitive lookups go through `function_node_keys_lower`.
    function_nodes: Arc<FxHashMap<Arc<str>, FunctionNode>>,
    /// Lowercased FQN → canonical FQN. Maintained in lockstep with
    /// `function_nodes` so callers can resolve PHP's case-insensitive
    /// function names (`STRLEN($x)` → `strlen`).
    function_node_keys_lower: Arc<FxHashMap<String, Arc<str>>>,
    /// (owner FQCN) → (method_name_lower → MethodNode) handle registry.
    method_nodes: MemberRegistry<MethodNode>,
    /// (owner FQCN) → (prop_name → PropertyNode) handle registry.
    property_nodes: MemberRegistry<PropertyNode>,
    /// (owner FQCN) → (const_name → ClassConstantNode) handle registry.
    class_constant_nodes: MemberRegistry<ClassConstantNode>,
    /// FQN → GlobalConstantNode handle registry.
    global_constant_nodes: Arc<FxHashMap<Arc<str>, GlobalConstantNode>>,
    /// File path → first declared namespace.
    file_namespaces: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// File path → use-alias imports.
    file_imports: Arc<FxHashMap<Arc<str>, HashMap<String, String>>>,
    /// Global variable name (without `$`) → collected type.
    global_vars: Arc<FxHashMap<Arc<str>, Union>>,
    /// Symbol FQN → defining file.
    symbol_to_file: Arc<FxHashMap<Arc<str>, Arc<str>>>,
    /// Public symbol key → reference locations.
    reference_locations: ReferenceLocations,
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        Arc::from("8.2")
    }

    fn lookup_class_node(&self, fqcn: &str) -> Option<ClassNode> {
        if let Some(&node) = self.class_nodes.get(fqcn) {
            return Some(node);
        }
        let lower = fqcn.to_ascii_lowercase();
        let canonical = self.class_node_keys_lower.get(&lower)?;
        self.class_nodes.get(canonical.as_ref()).copied()
    }

    fn lookup_function_node(&self, fqn: &str) -> Option<FunctionNode> {
        if let Some(&node) = self.function_nodes.get(fqn) {
            return Some(node);
        }
        let lower = fqn.to_ascii_lowercase();
        let canonical = self.function_node_keys_lower.get(&lower)?;
        self.function_nodes.get(canonical.as_ref()).copied()
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

    fn file_namespace(&self, file: &str) -> Option<Arc<str>> {
        self.file_namespaces.get(file).cloned()
    }

    fn file_imports(&self, file: &str) -> HashMap<String, String> {
        self.file_imports.get(file).cloned().unwrap_or_default()
    }

    fn global_var_type(&self, name: &str) -> Option<Union> {
        self.global_vars.get(name).cloned()
    }

    fn file_import_snapshots(&self) -> Vec<(Arc<str>, HashMap<String, String>)> {
        self.file_imports
            .iter()
            .map(|(file, imports)| (file.clone(), imports.clone()))
            .collect()
    }

    fn symbol_defining_file(&self, symbol: &str) -> Option<Arc<str>> {
        self.symbol_to_file.get(symbol).cloned()
    }

    fn symbols_defined_in_file(&self, file: &str) -> Vec<Arc<str>> {
        self.symbol_to_file
            .iter()
            .filter_map(|(sym, defining_file)| {
                if defining_file.as_ref() == file {
                    Some(sym.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn record_reference_location(&self, loc: RefLoc) {
        let mut refs = self
            .reference_locations
            .lock()
            .expect("reference lock poisoned");
        let entry = refs.entry(loc.symbol_key).or_default();
        let tuple = (loc.file, loc.line, loc.col_start, loc.col_end);
        if !entry.iter().any(|existing| existing == &tuple) {
            entry.push(tuple);
        }
    }

    fn replay_reference_locations(&self, file: Arc<str>, locs: &[(String, u32, u16, u16)]) {
        for (symbol, line, col_start, col_end) in locs {
            self.record_reference_location(RefLoc {
                symbol_key: Arc::from(symbol.as_str()),
                file: file.clone(),
                line: *line,
                col_start: *col_start,
                col_end: *col_end,
            });
        }
    }

    fn extract_file_reference_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self
            .reference_locations
            .lock()
            .expect("reference lock poisoned");
        let mut out = Vec::new();
        for (symbol, locs) in refs.iter() {
            for (loc_file, line, col_start, col_end) in locs {
                if loc_file.as_ref() == file {
                    out.push((symbol.clone(), *line, *col_start, *col_end));
                }
            }
        }
        out
    }

    fn reference_locations(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let refs = self
            .reference_locations
            .lock()
            .expect("reference lock poisoned");
        refs.get(symbol).cloned().unwrap_or_default()
    }

    fn has_reference(&self, symbol: &str) -> bool {
        let refs = self
            .reference_locations
            .lock()
            .expect("reference lock poisoned");
        refs.get(symbol).is_some_and(|locs| !locs.is_empty())
    }

    fn clear_file_references(&self, file: &str) {
        let mut refs = self
            .reference_locations
            .lock()
            .expect("reference lock poisoned");
        for locs in refs.values_mut() {
            locs.retain(|(loc_file, _, _, _)| loc_file.as_ref() != file);
        }
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
    pub fn remove_file_definitions(&mut self, file: &str) {
        let symbols = self.symbols_defined_in_file(file);
        for symbol in &symbols {
            self.deactivate_class_node(symbol);
            self.deactivate_function_node(symbol);
            self.deactivate_class_methods(symbol);
            self.deactivate_class_properties(symbol);
            self.deactivate_class_constants(symbol);
            self.deactivate_global_constant_node(symbol);
        }
        let symbol_set: HashSet<Arc<str>> = symbols.into_iter().collect();
        Arc::make_mut(&mut self.symbol_to_file).retain(|sym, defining_file| {
            defining_file.as_ref() != file && !symbol_set.contains(sym)
        });
        Arc::make_mut(&mut self.file_namespaces).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.file_imports).retain(|path, _| path.as_ref() != file);
        Arc::make_mut(&mut self.global_vars).retain(|name, _| !symbol_set.contains(name));
        self.clear_file_references(file);
    }

    pub fn type_count(&self) -> usize {
        self.class_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    pub fn function_count(&self) -> usize {
        self.function_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    pub fn constant_count(&self) -> usize {
        self.global_constant_nodes
            .values()
            .filter(|node| node.active(self))
            .count()
    }

    /// Walk one collected [`StubSlice`] and upsert the corresponding db nodes.
    ///
    /// This is the canonical post-Pass-1 ingestion path: each file's slice is
    /// fed in directly, so batch analysis does not need any intermediate
    /// mutable codebase store between Pass 1 and Pass 2.
    pub fn ingest_stub_slice(&mut self, slice: &StubSlice) {
        use std::collections::HashSet;

        // Deduplicate param lists to save memory (many methods share identical signatures).
        // This reduces cold-start memory usage by ~100-150 MiB when analyzing vendor code.
        let mut slice = slice.clone();
        mir_codebase::storage::deduplicate_params_in_slice(&mut slice);

        if let Some(file) = &slice.file {
            let file_cloned = file.clone();
            if let Some(namespace) = &slice.namespace {
                Arc::make_mut(&mut self.file_namespaces)
                    .insert(file_cloned.clone(), namespace.clone());
            }
            if !slice.imports.is_empty() {
                Arc::make_mut(&mut self.file_imports)
                    .insert(file_cloned.clone(), slice.imports.clone());
            }
            for (name, _) in &slice.global_vars {
                let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
                Arc::make_mut(&mut self.symbol_to_file)
                    .insert(Arc::from(global_name), file_cloned.clone());
            }
        }
        for (name, ty) in &slice.global_vars {
            let global_name = name.strip_prefix('$').unwrap_or(name.as_ref());
            Arc::make_mut(&mut self.global_vars).insert(Arc::from(global_name), ty.clone());
        }

        let slice_file = slice.file.as_ref().map(|f| f.clone());
        for cls in &slice.classes {
            if let Some(file) = &slice_file {
                Arc::make_mut(&mut self.symbol_to_file).insert(cls.fqcn.clone(), file.clone());
            }
            let fqcn_cloned = cls.fqcn.clone();
            self.upsert_class_node(ClassNodeFields {
                is_abstract: cls.is_abstract,
                parent: cls.parent.clone(),
                interfaces: Arc::from(cls.interfaces.as_ref()),
                traits: Arc::from(cls.traits.as_ref()),
                template_params: Arc::from(cls.template_params.as_ref()),
                mixins: Arc::from(cls.mixins.as_ref()),
                deprecated: cls.deprecated.clone(),
                is_final: cls.is_final,
                is_readonly: cls.is_readonly,
                location: cls.location.clone(),
                extends_type_args: Arc::from(cls.extends_type_args.as_ref()),
                implements_type_args: Arc::from(
                    cls.implements_type_args
                        .iter()
                        .map(|(iface, args)| (iface.clone(), Arc::from(args.as_ref())))
                        .collect::<Vec<_>>(),
                ),
                ..ClassNodeFields::for_class(fqcn_cloned)
            });
            if self.method_nodes.contains_key(cls.fqcn.as_ref()) {
                let method_keep: HashSet<&str> =
                    cls.own_methods.keys().map(|m| m.as_ref()).collect();
                self.prune_class_methods(&cls.fqcn, &method_keep);
            }
            for method in cls.own_methods.values() {
                // Avoid cloning complex return type Unions during vendor ingestion
                // by wrapping in Arc upfront. This is a per-method operation during
                // vendor type collection (rare after initialization), so the Arc
                // allocation is amortized.
                self.upsert_method_node(method.as_ref());
            }
            if self.property_nodes.contains_key(cls.fqcn.as_ref()) {
                let prop_keep: HashSet<&str> =
                    cls.own_properties.keys().map(|p| p.as_ref()).collect();
                self.prune_class_properties(&cls.fqcn, &prop_keep);
            }
            for prop in cls.own_properties.values() {
                self.upsert_property_node(&cls.fqcn, prop);
            }
            if self.class_constant_nodes.contains_key(cls.fqcn.as_ref()) {
                let const_keep: HashSet<&str> =
                    cls.own_constants.keys().map(|c| c.as_ref()).collect();
                self.prune_class_constants(&cls.fqcn, &const_keep);
            }
            for constant in cls.own_constants.values() {
                self.upsert_class_constant_node(&cls.fqcn, constant);
            }
        }

        for iface in &slice.interfaces {
            if let Some(file) = &slice_file {
                Arc::make_mut(&mut self.symbol_to_file).insert(iface.fqcn.clone(), file.clone());
            }
            let fqcn_cloned = iface.fqcn.clone();
            self.upsert_class_node(ClassNodeFields {
                extends: Arc::from(iface.extends.as_ref()),
                template_params: Arc::from(iface.template_params.as_ref()),
                location: iface.location.clone(),
                ..ClassNodeFields::for_interface(fqcn_cloned)
            });
            if self.method_nodes.contains_key(iface.fqcn.as_ref()) {
                let method_keep: HashSet<&str> =
                    iface.own_methods.keys().map(|m| m.as_ref()).collect();
                self.prune_class_methods(&iface.fqcn, &method_keep);
            }
            for method in iface.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            if self.class_constant_nodes.contains_key(iface.fqcn.as_ref()) {
                let const_keep: HashSet<&str> =
                    iface.own_constants.keys().map(|c| c.as_ref()).collect();
                self.prune_class_constants(&iface.fqcn, &const_keep);
            }
            for constant in iface.own_constants.values() {
                self.upsert_class_constant_node(&iface.fqcn, constant);
            }
        }

        for tr in &slice.traits {
            if let Some(file) = &slice_file {
                Arc::make_mut(&mut self.symbol_to_file).insert(tr.fqcn.clone(), file.clone());
            }
            let fqcn_cloned = tr.fqcn.clone();
            self.upsert_class_node(ClassNodeFields {
                traits: Arc::from(tr.traits.as_ref()),
                template_params: Arc::from(tr.template_params.as_ref()),
                require_extends: Arc::from(tr.require_extends.as_ref()),
                require_implements: Arc::from(tr.require_implements.as_ref()),
                location: tr.location.clone(),
                ..ClassNodeFields::for_trait(fqcn_cloned)
            });
            if self.method_nodes.contains_key(tr.fqcn.as_ref()) {
                let method_keep: HashSet<&str> =
                    tr.own_methods.keys().map(|m| m.as_ref()).collect();
                self.prune_class_methods(&tr.fqcn, &method_keep);
            }
            for method in tr.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            if self.property_nodes.contains_key(tr.fqcn.as_ref()) {
                let prop_keep: HashSet<&str> =
                    tr.own_properties.keys().map(|p| p.as_ref()).collect();
                self.prune_class_properties(&tr.fqcn, &prop_keep);
            }
            for prop in tr.own_properties.values() {
                self.upsert_property_node(&tr.fqcn, prop);
            }
            if self.class_constant_nodes.contains_key(tr.fqcn.as_ref()) {
                let const_keep: HashSet<&str> =
                    tr.own_constants.keys().map(|c| c.as_ref()).collect();
                self.prune_class_constants(&tr.fqcn, &const_keep);
            }
            for constant in tr.own_constants.values() {
                self.upsert_class_constant_node(&tr.fqcn, constant);
            }
        }

        for en in &slice.enums {
            if let Some(file) = &slice_file {
                Arc::make_mut(&mut self.symbol_to_file).insert(en.fqcn.clone(), file.clone());
            }
            let fqcn_cloned = en.fqcn.clone();
            self.upsert_class_node(ClassNodeFields {
                interfaces: Arc::from(en.interfaces.as_ref()),
                is_backed_enum: en.scalar_type.is_some(),
                enum_scalar_type: en.scalar_type.clone(),
                location: en.location.clone(),
                ..ClassNodeFields::for_enum(fqcn_cloned)
            });
            if self.method_nodes.contains_key(en.fqcn.as_ref()) {
                let mut method_keep: HashSet<&str> =
                    en.own_methods.keys().map(|m| m.as_ref()).collect();
                method_keep.insert("cases");
                if en.scalar_type.is_some() {
                    method_keep.insert("from");
                    method_keep.insert("tryfrom");
                }
                self.prune_class_methods(&en.fqcn, &method_keep);
            }
            for method in en.own_methods.values() {
                self.upsert_method_node(method.as_ref());
            }
            let synth_method = |name: &str| mir_codebase::storage::MethodStorage {
                fqcn: en.fqcn.clone(),
                name: Arc::from(name),
                params: Arc::from([].as_ref()),
                return_type: Some(Arc::new(Union::mixed())),
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
            if self.class_constant_nodes.contains_key(en.fqcn.as_ref()) {
                let mut const_keep: HashSet<&str> =
                    en.own_constants.keys().map(|c| c.as_ref()).collect();
                for case in en.cases.values() {
                    const_keep.insert(case.name.as_ref());
                }
                self.prune_class_constants(&en.fqcn, &const_keep);
            }
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

        for func in &slice.functions {
            if let Some(file) = &slice_file {
                Arc::make_mut(&mut self.symbol_to_file).insert(func.fqn.clone(), file.clone());
            }
            self.upsert_function_node(func);
        }
        for (fqn, ty) in &slice.constants {
            let fqn_cloned = fqn.clone();
            self.upsert_global_constant_node(fqn_cloned, ty.clone());
        }
    }

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
            // (`ingest_stub_slice` / `lazy_load_*`) call this for every class
            // on every iteration; without the skip each call fires 13
            // setters, each acquiring the Salsa write lock.  Schema doesn't
            // mutate after Pass 1 (Pass 2 only writes `inferred_return_type`),
            // so an active node with matching fields is by construction up
            // to date.
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
            Arc::make_mut(&mut self.class_node_keys_lower)
                .insert(fqcn.to_ascii_lowercase(), fqcn.clone());
            Arc::make_mut(&mut self.class_nodes).insert(fqcn, node);
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
            // `inferred_return_type` is intentionally NOT compared / written:
            // it is owned by the priming sweep's serial commit phase
            // (`commit_inferred_return_types`) and Pass-1 re-ingest must not
            // clobber a previously-inferred value.
            if node.active(self)
                && node.short_name(self) == storage.short_name
                && node.is_pure(self) == storage.is_pure
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self).as_deref() == storage.return_type.as_deref()
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_ref()
                && *node.template_params(self) == *storage.template_params
                && *node.assertions(self) == *storage.assertions
                && *node.throws(self) == *storage.throws
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_short_name(self).to(storage.short_name.clone());
            node.set_params(self).to(storage.params.clone());
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
                storage.params.clone(),
                storage.return_type.clone(),
                storage
                    .inferred_return_type
                    .as_ref()
                    .map(|t| Arc::new(t.clone())),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.is_pure,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.function_node_keys_lower)
                .insert(fqn.to_ascii_lowercase(), fqn.clone());
            Arc::make_mut(&mut self.function_nodes).insert(fqn.clone(), node);
            node
        }
    }

    /// Commit a parallel-sweep-collected [`InferredReturnTypes`] buffer
    /// into the Salsa db.  **Must be called serially**, after all rayon
    /// workers from the priming sweep have dropped their db clones, so
    /// that `Storage::cancel_others` sees strong-count==1 inside the
    /// setter.  Calling this from inside a `for_each_with` / `map_with`
    /// closure will deadlock.
    ///
    /// Skips writes whose value already matches the current Salsa-tracked
    /// value (preserves PR21's fast-skip semantics).  Skips inactive
    /// nodes — there's no point committing an inferred return for a node
    /// that has been deactivated by a re-analyze.
    /// Commit inferred return types collected during the priming sweep.
    /// Takes ownership of the function and method inferred type vectors.
    pub fn commit_inferred_return_types(
        &mut self,
        functions: Vec<(Arc<str>, mir_types::Union)>,
        methods: Vec<(Arc<str>, Arc<str>, mir_types::Union)>,
    ) {
        use salsa::Setter as _;
        for (fqn, inferred) in functions {
            if let Some(&node) = self.function_nodes.get(fqn.as_ref()) {
                if !node.active(self) {
                    continue;
                }
                let new = Some(Arc::new(inferred));
                if node.inferred_return_type(self) == new {
                    continue;
                }
                node.set_inferred_return_type(self).to(new);
            }
        }
        for (fqcn, name, inferred) in methods {
            let name_lower: Arc<str> = if name.chars().all(|c| !c.is_uppercase()) {
                name.clone()
            } else {
                Arc::from(name.to_lowercase().as_str())
            };
            let node = self
                .method_nodes
                .get(fqcn.as_ref())
                .and_then(|m| m.get(&name_lower))
                .copied();
            if let Some(node) = node {
                if !node.active(self) {
                    continue;
                }
                let new = Some(Arc::new(inferred));
                if node.inferred_return_type(self) == new {
                    continue;
                }
                node.set_inferred_return_type(self).to(new);
            }
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
            // `inferred_return_type` intentionally not compared / written here;
            // ownership is in the priming-sweep commit phase.
            if node.active(self)
                && node.visibility(self) == storage.visibility
                && node.is_static(self) == storage.is_static
                && node.is_abstract(self) == storage.is_abstract
                && node.is_final(self) == storage.is_final
                && node.is_constructor(self) == storage.is_constructor
                && node.is_pure(self) == storage.is_pure
                && node.is_internal(self) == storage.is_internal
                && node.deprecated(self) == storage.deprecated
                && node.return_type(self).as_deref() == storage.return_type.as_deref()
                && node.location(self) == storage.location
                && *node.params(self) == *storage.params.as_ref()
                && *node.template_params(self) == *storage.template_params
                && *node.assertions(self) == *storage.assertions
                && *node.throws(self) == *storage.throws
            {
                return node;
            }
            node.set_active(self).to(true);
            node.set_params(self).to(storage.params.clone());
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_is_internal(self).to(storage.is_internal);
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
                storage.params.clone(),
                storage.return_type.clone(),
                storage
                    .inferred_return_type
                    .as_ref()
                    .map(|t| Arc::new(t.clone())),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.is_internal,
                storage.visibility,
                storage.is_static,
                storage.is_abstract,
                storage.is_final,
                storage.is_constructor,
                storage.is_pure,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.method_nodes)
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
    /// `keep_lower`.  Used by `ingest_stub_slice` to prune stale stub methods
    /// when a user file shadows a bundled-stub class with a different method
    /// set.  Active-only check preserves PR21's fast-skip — already-inactive
    /// nodes don't fire a setter.
    pub fn prune_class_methods<T>(&mut self, fqcn: &str, keep_lower: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
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
    pub fn prune_class_properties<T>(&mut self, fqcn: &str, keep: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
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
    pub fn prune_class_constants<T>(&mut self, fqcn: &str, keep: &std::collections::HashSet<T>)
    where
        T: Eq + std::hash::Hash + std::borrow::Borrow<str>,
    {
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
            Arc::make_mut(&mut self.property_nodes)
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
                && node.location(self) == storage.location
            {
                return;
            }
            node.set_active(self).to(true);
            node.set_ty(self).to(storage.ty.clone());
            node.set_visibility(self).to(storage.visibility);
            node.set_is_final(self).to(storage.is_final);
            node.set_location(self).to(storage.location.clone());
        } else {
            let node = ClassConstantNode::new(
                self,
                fqcn.clone(),
                storage.name.clone(),
                true,
                storage.ty.clone(),
                storage.visibility,
                storage.is_final,
                storage.location.clone(),
            );
            Arc::make_mut(&mut self.class_constant_nodes)
                .entry(fqcn.clone())
                .or_default()
                .insert(storage.name.clone(), node);
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
            Arc::make_mut(&mut self.global_constant_nodes).insert(fqn, node);
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

// S4 Step 1: analyze_file accumulators + tracked-query skeleton
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

// S4 Step 3: Lazy inferred-type queries
//
// These tracked queries compute inferred return types on-demand during Pass 2.
// When `Pass2Driver` encounters a function/method call, it reads the inferred
// type via these queries instead of from a pre-computed buffer.
//
// This enables two key optimizations:
// 1. Single-pass execution: inferred types are computed as needed, not upfront
// 2. Incremental caching: if a dependent file doesn't call a function, its
//    inferred type is never computed (Salsa skips the query)

/// Lazily computes the inferred return type for a function.
/// Called on-demand during Pass 2 analysis when we encounter a call to this function.
/// Results are cached by Salsa; re-analysis of dependent files that don't call this
/// function re-uses the cached inferred type.
///
/// **Current behavior (S4 PR3):** Reads from the already-committed `inferred_return_type`
/// field on `FunctionNode`. Double-pass orchestration (Pass 2a inference + commit) still
/// happens in `project.rs::analyze()`.
///
/// **Future (S4 PR4):** Will compute types on-demand by extracting the function body
/// from source and running inference-only Pass 2, eliminating the double-pass.
#[salsa::tracked]
pub fn inferred_function_return_type(db: &dyn MirDatabase, node: FunctionNode) -> Arc<Union> {
    // For now, read the already-committed inferred type from the FunctionNode input.
    // This is set via commit_inferred_return_types() after Pass 2a completes.
    node.inferred_return_type(db)
        .unwrap_or_else(|| Arc::new(Union::mixed()))
}

/// Lazily computes the inferred return type for a method.
///
/// **Current behavior (S4 PR3):** Reads from the already-committed `inferred_return_type`
/// field on `MethodNode`.
///
/// **Future (S4 PR4):** Will compute types on-demand by extracting the method body
/// from source and running inference-only Pass 2.
#[salsa::tracked]
pub fn inferred_method_return_type(db: &dyn MirDatabase, node: MethodNode) -> Arc<Union> {
    // For now, read the already-committed inferred type from the MethodNode input.
    node.inferred_return_type(db)
        .unwrap_or_else(|| Arc::new(Union::mixed()))
}

// Helper: collect analysis results via tracked query accumulators

/// Collects all accumulated issues from a set of files analyzed via the
/// `analyze_file` tracked query. Used during batch analysis to read issues
/// that were emitted during tracked-query evaluation.
#[allow(dead_code)]
pub(crate) fn collect_accumulated_issues(
    db: &dyn MirDatabase,
    files: &[(Arc<str>, SourceFile)],
    php_version: &str,
) -> Vec<Issue> {
    let mut all_issues = Vec::new();
    let input = AnalyzeFileInput::new(db, Arc::from(php_version));

    for (_path, file) in files {
        // Call the tracked query to trigger analysis + accumulation
        analyze_file(db, *file, input);

        // Read back the accumulated issues for this file
        let accumulated: Vec<&IssueAccumulator> = analyze_file::accumulated(db, *file, input);
        for acc in accumulated {
            all_issues.push(acc.0.clone());
        }
    }

    all_issues
}

/// Tracked-query skeleton for `analyze_file`.
///
/// **Current behavior (S4 step 2):** parses the file, emits parse-error issues,
/// and calls Pass 2 to analyze function/method bodies. Issues and reference
/// locations are emitted via `IssueAccumulator` and `RefLocAccumulator`.
///
/// This is still a hybrid: inferred types come from the prior
/// `run_inference_sweep` → `commit_inferred_return_types` in the double-pass
/// orchestration. Future S4 PRs will replace that with lazy
/// `inferred_return_type(node)` tracked queries.
#[salsa::tracked]
pub fn analyze_file(db: &dyn MirDatabase, file: SourceFile, input: AnalyzeFileInput) {
    use salsa::Accumulator as _;
    let path = file.path(db);
    let text = file.text(db);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &text);

    // Emit parse errors
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

    // If no parse errors, run full analysis via Pass2Driver
    if parsed.errors.is_empty() {
        use std::str::FromStr as _;
        let php_version =
            PhpVersion::from_str(input.php_version(db).as_ref()).unwrap_or(PhpVersion::LATEST);
        let driver = Pass2Driver::new(db, php_version);
        let (issues, _symbols) = driver.analyze_bodies(
            &parsed.program,
            path.clone(),
            text.as_ref(),
            &parsed.source_map,
        );

        // Emit issues via accumulator
        for issue in issues {
            IssueAccumulator(issue).accumulate(db);
        }

        // Emit reference locations via accumulator
        let ref_locs = db.extract_file_reference_locations(&path);
        for (symbol_key, line, col_start, col_end) in ref_locs {
            let ref_loc = RefLoc {
                symbol_key,
                file: path.clone(),
                line,
                col_start,
                col_end,
            };
            RefLocAccumulator(ref_loc).accumulate(db);
        }
    }
}

// Tests

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
    fn analyze_file_calls_pass2_for_undefined_class() {
        let mut db = MirDb::default();
        // Load stubs so we have a baseline codebase
        for slice in crate::stubs::builtin_stub_slices_for_version(crate::PhpVersion::LATEST) {
            db.ingest_stub_slice(&slice);
        }

        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/test_pass2.php"),
            Arc::from("<?php function foo() { new UndefinedClass(); }"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);

        assert!(
            !issues.is_empty(),
            "Pass2Driver should emit UndefinedClass issue"
        );
        assert!(issues
            .iter()
            .any(|acc| matches!(acc.0.kind, mir_issues::IssueKind::UndefinedClass { .. })));
    }

    #[test]
    fn inferred_function_return_type_query_defined() {
        let mut db = MirDb::default();

        // Create a simple function via FunctionStorage
        let func_storage = FunctionStorage {
            fqn: Arc::from("test_fn"),
            short_name: Arc::from("test_fn"),
            params: Arc::from([]),
            return_type: None,
            inferred_return_type: Some(Union::int()),
            template_params: Vec::new(),
            assertions: Vec::new(),
            throws: Vec::new(),
            deprecated: None,
            is_pure: false,
            location: None,
        };
        let node = db.upsert_function_node(&func_storage);

        // Query should return the inferred type
        let inferred = inferred_function_return_type(&db, node);
        assert_eq!(inferred.as_ref(), &Union::int());
    }

    #[test]
    fn inferred_method_return_type_query_defined() {
        let mut db = MirDb::default();

        // Create a simple method via MethodStorage
        let method_storage = MethodStorage {
            fqcn: Arc::from("TestClass"),
            name: Arc::from("testMethod"),
            params: Arc::from([]),
            return_type: None,
            inferred_return_type: Some(Union::string()),
            template_params: Vec::new(),
            assertions: Vec::new(),
            throws: Vec::new(),
            deprecated: None,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract: false,
            is_final: false,
            is_constructor: false,
            is_pure: false,
            is_internal: false,
            location: None,
        };
        let node = db.upsert_method_node(&method_storage);

        // Query should return the inferred type
        let inferred = inferred_method_return_type(&db, node);
        assert_eq!(inferred.as_ref(), &Union::string());
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
            params: Arc::from([].as_slice()),
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

    /// Validates the S3-deadlock premise.  After `for_each_with` returns,
    /// all worker clones must drop so that a subsequent setter on the
    /// canonical db (strong-count==1) does not block on
    /// `Storage::cancel_others`.  Wrapped in a join-with-timeout so a
    /// regression hangs for at most 30s instead of forever.
    #[test]
    fn parallel_reads_then_serial_write_does_not_deadlock() {
        use rayon::prelude::*;
        use std::sync::mpsc;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            let mut db = MirDb::default();
            let storage = mir_codebase::storage::FunctionStorage {
                fqn: Arc::from("foo"),
                short_name: Arc::from("foo"),
                params: Arc::from([].as_slice()),
                return_type: None,
                inferred_return_type: None,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                deprecated: None,
                is_pure: false,
                location: None,
            };
            let node = db.upsert_function_node(&storage);

            // Parallel sweep with cloned dbs; each worker reads via &dyn MirDatabase.
            let db_for_sweep = db.clone();
            (0..256u32)
                .into_par_iter()
                .for_each_with(db_for_sweep, |db, _| {
                    let _ = node.return_type(&*db as &dyn MirDatabase);
                });

            // Sweep is done — clones owned by `for_each_with` are dropped.
            // If any worker-thread retains thread-local Salsa state pointing
            // at a clone, this setter will hang in `Storage::cancel_others`.
            node.set_return_type(&mut db)
                .to(Some(Arc::new(Union::mixed())));
            assert_eq!(node.return_type(&db), Some(Arc::new(Union::mixed())));
            tx.send(()).unwrap();
        });

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(()) => {}
            Err(_) => {
                panic!("S3 deadlock repro: setter after for_each_with did not return within 30s")
            }
        }
    }

    /// Pins the actual root cause of the original S3 deadlock: a sibling
    /// `MirDb` clone (e.g. the `class_db` used by `ClassAnalyzer` in
    /// `project.rs`) being alive when a setter runs blocks
    /// `Storage::cancel_others` indefinitely.  Dropping the sibling before
    /// the setter unblocks it.
    ///
    /// This is the regression guard for `commit_inferred_return_types`: if
    /// a future refactor hoists a clone past the commit point, this test
    /// fails (either the "while sibling alive, setter is blocked" half
    /// or the "after drop, setter completes" half).
    #[test]
    fn sibling_clone_blocks_setter_until_dropped() {
        use std::sync::mpsc;
        use std::time::Duration;

        let mut db = MirDb::default();
        let storage = mir_codebase::storage::FunctionStorage {
            fqn: Arc::from("foo"),
            short_name: Arc::from("foo"),
            params: Arc::from([].as_slice()),
            return_type: None,
            inferred_return_type: None,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_pure: false,
            location: None,
        };
        let node = db.upsert_function_node(&storage);

        let sibling = db.clone();

        // Move the writer into a worker thread so we can probe its progress
        // without blocking the test.  Channel signals when the setter returns.
        let (tx, rx) = mpsc::channel::<()>();
        let writer = std::thread::spawn(move || {
            node.set_return_type(&mut db)
                .to(Some(Arc::new(Union::mixed())));
            tx.send(()).unwrap();
        });

        // While the sibling clone is alive the setter must NOT make progress —
        // strong-count > 1 forces `cancel_others` to wait.
        match rx.recv_timeout(Duration::from_millis(500)) {
            Err(mpsc::RecvTimeoutError::Timeout) => { /* expected */ }
            Ok(()) => panic!(
                "setter completed while sibling clone was alive — strong-count==1 \
                 invariant of `cancel_others` is broken; commit_inferred_return_types \
                 cannot rely on tight-scoping clones"
            ),
            Err(e) => panic!("unexpected channel error: {e:?}"),
        }

        // Drop the sibling.  Strong-count drops to 1 and the setter unblocks.
        drop(sibling);

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(()) => {}
            Err(_) => panic!("setter did not complete within 5s after sibling clone dropped"),
        }
        writer.join().expect("writer thread panicked");
    }
}
