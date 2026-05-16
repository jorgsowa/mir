use std::sync::Arc;

use mir_codebase::storage::{Location, TemplateParam};
use mir_issues::Issue;
use mir_types::Union;
use rustc_hash::FxHashMap;

use super::*;

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

pub fn type_exists_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    db.lookup_class_node(fqcn).is_some_and(|n| n.active(db))
}

#[allow(dead_code)]
pub fn function_exists_via_db(db: &dyn MirDatabase, fqn: &str) -> bool {
    db.lookup_function_node(fqn).is_some_and(|n| n.active(db))
}

#[allow(dead_code)]
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

/// Existence-only sibling of [`trait_provides_method`].  Returns true iff the
/// trait or any sub-trait declares a method named `method_lower` (abstract
/// counts).  Cycle-safe via `visited`.
#[allow(dead_code)]
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

#[allow(dead_code)]
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

    let arena = crate::arena::create_parse_arena(text.len());
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
    let (mut slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);
    mir_codebase::storage::deduplicate_params_in_slice(&mut slice);

    FileDefinitions {
        slice: Arc::new(slice),
        issues: Arc::new(all_issues),
    }
}

#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    collect_file_definitions_uncached(db, file)
}

// File-level inferred-type Salsa query
//
// `infer_file_return_types` runs an inference-only Pass 2 over one file and
// returns all inferred return types as maps.  The pre-sweep in `project.rs`
// calls this in parallel for all files, then commits the results to Salsa
// INPUT fields.  The full Pass 2 then reads those INPUT fields directly (O(1)
// per call site, no lock contention at scale).
//
// Cross-file PHP call graphs commonly have A→B→A cycles.  This query reads
// only *committed* INPUT fields for external symbols (never calls back into
// another file's `infer_file_return_types`), so cycles degrade only to `mixed`
// on the first fixpoint pass, not recursively.

type MethodInferMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Union>>;

/// Inferred return types for all functions and methods in a single file,
/// collected by a single inference-only Pass 2 run.
#[derive(Clone, Debug)]
pub struct InferredFileTypes {
    /// FQN → inferred return type for top-level functions declared in the file.
    pub functions: Arc<FxHashMap<Arc<str>, Arc<Union>>>,
    /// (FQCN, method_name_lower) → inferred return type for methods in the file.
    pub methods: Arc<MethodInferMap>,
}

impl InferredFileTypes {
    pub fn empty() -> Self {
        Self {
            functions: Arc::new(FxHashMap::default()),
            methods: Arc::new(MethodInferMap::default()),
        }
    }
}

impl PartialEq for InferredFileTypes {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.methods, &other.methods)
        {
            return true;
        }
        if self.functions.len() != other.functions.len()
            || self.methods.len() != other.methods.len()
        {
            return false;
        }
        for (k, v) in self.functions.iter() {
            if other.functions.get(k).is_none_or(|ov| *ov != *v) {
                return false;
            }
        }
        for (k, v) in self.methods.iter() {
            if other.methods.get(k).is_none_or(|ov| *ov != *v) {
                return false;
            }
        }
        true
    }
}

unsafe impl salsa::Update for InferredFileTypes {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

/// Run inference-only Pass 2 over `file` and return all inferred return types.
///
/// Salsa tracks the dependency on `file.text`, so this query is automatically
/// invalidated when the file changes and re-used when it hasn't.
///
/// Called in parallel by the pre-sweep in `project.rs` and `session.rs`.
/// Results are committed to `FunctionNode`/`MethodNode` INPUT fields before
/// the full Pass 2 runs, so the hot path pays only O(1) INPUT-field reads.
#[salsa::tracked]
pub fn infer_file_return_types(db: &dyn MirDatabase, file: SourceFile) -> InferredFileTypes {
    use std::str::FromStr as _;
    let path = file.path(db);
    let text = file.text(db);
    let php_version = crate::php_version::PhpVersion::from_str(db.php_version_str().as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST);

    let arena = crate::arena::create_parse_arena(text.len());
    let parsed = php_rs_parser::parse(&arena, text.as_ref());

    if !parsed.errors.is_empty() {
        return InferredFileTypes::empty();
    }

    let driver = crate::pass2::Pass2Driver::new_inference_only(db, php_version);
    driver.analyze_bodies(&parsed.program, path, text.as_ref(), &parsed.source_map);
    let inferred = driver.take_inferred_types();

    let mut functions: FxHashMap<Arc<str>, Arc<Union>> =
        FxHashMap::with_capacity_and_hasher(inferred.functions.len(), Default::default());
    for (fqn, ty) in inferred.functions {
        functions.insert(fqn, Arc::new(ty));
    }

    let mut methods: FxHashMap<(Arc<str>, Arc<str>), Arc<Union>> =
        FxHashMap::with_capacity_and_hasher(inferred.methods.len(), Default::default());
    for (fqcn, name, ty) in inferred.methods {
        let name_lower: Arc<str> = if name.chars().all(|c| !c.is_uppercase()) {
            name
        } else {
            Arc::from(name.to_lowercase().as_str())
        };
        methods.insert((fqcn, name_lower), Arc::new(ty));
    }

    InferredFileTypes {
        functions: Arc::new(functions),
        methods: Arc::new(methods),
    }
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

/// Return `true` if `fqcn` descends from `RuntimeException` or
/// `LogicException`, which by PHP convention are "unchecked" exceptions —
/// programmer errors that callers are not expected to document or recover
/// from.  The analyzer suppresses [`MissingThrowsDocblock`] diagnostics for
/// these by default to avoid drowning real signal in @throws noise.
pub fn is_unchecked_exception_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    extends_or_implements_via_db(db, fqcn, "RuntimeException")
        || extends_or_implements_via_db(db, fqcn, "LogicException")
}
