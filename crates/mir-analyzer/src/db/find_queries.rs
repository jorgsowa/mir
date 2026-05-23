//! Pull-based symbol lookups.
//!
//! Per-file extractor queries (`class_in_file`, `interface_in_file`, …)
//! that read from the already-tracked `collect_file_definitions` and locate
//! a definition by name. Plus composite helpers (`find_class_like`,
//! `find_function`) that combine resolution + extraction so callers can
//! "find by FQCN" with a single call.
//!
//! These return `Arc<StorageType>` rather than salsa input handles. The data
//! lives in the `StubSlice` produced by `collect_file_definitions`; the
//! Arc-wrap makes salsa's identity comparison cheap (ptr_eq) and avoids
//! deep clones.

use std::sync::Arc;

use mir_codebase::storage::{
    ClassStorage, ConstantStorage, EnumStorage, FunctionStorage, InterfaceStorage, MethodStorage,
    PropertyStorage, TraitStorage,
};
use mir_types::Symbol;

use crate::db::{collect_file_definitions, source_file_for_fqcn, Fqcn, MirDatabase, SourceFile};

/// Tagged union over the four PHP class-like kinds. The result type of
/// composite `find_class_like` so callers receive a single response that
/// covers `class` / `interface` / `trait` / `enum`.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassLike {
    Class(Arc<ClassStorage>),
    Interface(Arc<InterfaceStorage>),
    Trait(Arc<TraitStorage>),
    Enum(Arc<EnumStorage>),
}

impl ClassLike {
    pub fn fqcn(&self) -> &Arc<str> {
        match self {
            ClassLike::Class(c) => &c.fqcn,
            ClassLike::Interface(i) => &i.fqcn,
            ClassLike::Trait(t) => &t.fqcn,
            ClassLike::Enum(e) => &e.fqcn,
        }
    }

    pub fn short_name(&self) -> &Arc<str> {
        match self {
            ClassLike::Class(c) => &c.short_name,
            ClassLike::Interface(i) => &i.short_name,
            ClassLike::Trait(t) => &t.short_name,
            ClassLike::Enum(e) => &e.short_name,
        }
    }

    /// Returns whatever this kind considers its "parents" — what salsa
    /// `class_ancestors_by_fqcn` will walk:
    ///   - Class: `parent` (single, if any) + `interfaces` + `traits`
    ///   - Interface: `extends` (multi)
    ///   - Trait: used `traits`
    ///   - Enum: `interfaces`
    ///
    /// `@mixin` FQCNs are intentionally excluded here — they are handled by
    /// `find_method_in_chain` via a separate cycle-safe walk so they don't
    /// affect `has_unknown_ancestor_via_db` checks.
    pub fn ancestor_fqcns(&self) -> Vec<Arc<str>> {
        match self {
            ClassLike::Class(c) => {
                let mut out = Vec::new();
                if let Some(p) = &c.parent {
                    out.push(p.clone());
                }
                out.extend(c.interfaces.iter().cloned());
                out.extend(c.traits.iter().cloned());
                out
            }
            ClassLike::Interface(i) => i.extends.clone(),
            ClassLike::Trait(t) => t.traits.clone(),
            ClassLike::Enum(e) => e.interfaces.clone(),
        }
    }

    /// Own methods (does not include inherited). Class / interface / trait
    /// / enum all carry these (interfaces hold abstract method signatures).
    pub fn own_methods(&self) -> &indexmap::IndexMap<Arc<str>, Arc<MethodStorage>> {
        match self {
            ClassLike::Class(c) => &c.own_methods,
            ClassLike::Interface(i) => &i.own_methods,
            ClassLike::Trait(t) => &t.own_methods,
            ClassLike::Enum(e) => &e.own_methods,
        }
    }

    /// Own properties. Interfaces don't have properties, so we return an
    /// empty map for them (avoids match callers having to special-case).
    pub fn own_properties(&self) -> Option<&indexmap::IndexMap<Arc<str>, PropertyStorage>> {
        match self {
            ClassLike::Class(c) => Some(&c.own_properties),
            ClassLike::Trait(t) => Some(&t.own_properties),
            ClassLike::Interface(_) | ClassLike::Enum(_) => None,
        }
    }

    /// Own constants.
    pub fn own_constants(&self) -> &indexmap::IndexMap<Arc<str>, ConstantStorage> {
        match self {
            ClassLike::Class(c) => &c.own_constants,
            ClassLike::Interface(i) => &i.own_constants,
            ClassLike::Trait(t) => &t.own_constants,
            ClassLike::Enum(e) => &e.own_constants,
        }
    }

    pub fn is_abstract(&self) -> bool {
        match self {
            ClassLike::Class(c) => c.is_abstract,
            ClassLike::Interface(_) => true, // interfaces are inherently abstract
            ClassLike::Trait(_) | ClassLike::Enum(_) => false,
        }
    }

    pub fn is_final(&self) -> bool {
        match self {
            ClassLike::Class(c) => c.is_final,
            ClassLike::Enum(_) => true, // enums are implicitly final
            ClassLike::Interface(_) | ClassLike::Trait(_) => false,
        }
    }

    pub fn is_interface(&self) -> bool {
        matches!(self, ClassLike::Interface(_))
    }

    pub fn is_trait(&self) -> bool {
        matches!(self, ClassLike::Trait(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, ClassLike::Enum(_))
    }

    pub fn is_class(&self) -> bool {
        matches!(self, ClassLike::Class(_))
    }

    /// `use SomeTrait;` declarations on a class or trait body. Interfaces
    /// and enums never have trait uses; they return an empty slice.
    pub fn class_traits(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Class(c) => &c.traits,
            ClassLike::Trait(t) => &t.traits,
            _ => &[],
        }
    }

    /// `@mixin` FQCNs (class only).
    pub fn mixins(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Class(c) => &c.mixins,
            _ => &[],
        }
    }

    /// `@deprecated` docblock annotation, if present.
    pub fn deprecated(&self) -> Option<&Arc<str>> {
        match self {
            ClassLike::Class(c) => c.deprecated.as_ref(),
            _ => None,
        }
    }

    /// Declared `@template` parameters.
    pub fn template_params(&self) -> &[mir_codebase::storage::TemplateParam] {
        match self {
            ClassLike::Class(c) => &c.template_params,
            ClassLike::Interface(i) => &i.template_params,
            ClassLike::Trait(t) => &t.template_params,
            ClassLike::Enum(_) => &[],
        }
    }

    /// Source location of the declaration.
    pub fn location(&self) -> Option<&mir_codebase::storage::Location> {
        match self {
            ClassLike::Class(c) => c.location.as_ref(),
            ClassLike::Interface(i) => i.location.as_ref(),
            ClassLike::Trait(t) => t.location.as_ref(),
            ClassLike::Enum(e) => e.location.as_ref(),
        }
    }

    /// Implemented interfaces (classes + enums; empty for interfaces and
    /// traits — interfaces use `extends`, traits use `traits`).
    pub fn interfaces(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Class(c) => &c.interfaces,
            ClassLike::Enum(e) => &e.interfaces,
            _ => &[],
        }
    }

    /// Parent class (Class only; None otherwise).
    pub fn parent(&self) -> Option<&Arc<str>> {
        match self {
            ClassLike::Class(c) => c.parent.as_ref(),
            _ => None,
        }
    }

    /// For backed enums: the scalar type they map to.
    pub fn enum_scalar_type(&self) -> Option<&mir_types::Union> {
        match self {
            ClassLike::Enum(e) => e.scalar_type.as_ref(),
            _ => None,
        }
    }

    /// `extends` list (interfaces only; class uses `parent`).
    pub fn extends(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Interface(i) => &i.extends,
            _ => &[],
        }
    }

    /// `@extends Parent<T1, T2>` type args (class only).
    pub fn extends_type_args(&self) -> &[mir_types::Union] {
        match self {
            ClassLike::Class(c) => &c.extends_type_args,
            _ => &[],
        }
    }

    /// `@implements Iface<T1, T2>` type args (class only).
    pub fn implements_type_args(&self) -> &[(Arc<str>, Vec<mir_types::Union>)] {
        match self {
            ClassLike::Class(c) => &c.implements_type_args,
            _ => &[],
        }
    }

    /// Per-`use SomeTrait;` declaration locations (class + trait).
    pub fn trait_use_locations(&self) -> &[(Arc<str>, mir_codebase::storage::Location)] {
        match self {
            ClassLike::Class(c) => &c.trait_use_locations,
            _ => &[],
        }
    }

    /// Whether the class is `readonly` (PHP 8.2+).
    pub fn is_readonly(&self) -> bool {
        match self {
            ClassLike::Class(c) => c.is_readonly,
            _ => false,
        }
    }

    /// Whether the class is marked `@internal`.
    pub fn is_internal(&self) -> bool {
        match self {
            ClassLike::Class(c) => c.is_internal,
            _ => false,
        }
    }

    /// Backed-enum check (`enum Foo: int { ... }`).
    pub fn is_backed_enum(&self) -> bool {
        match self {
            ClassLike::Enum(e) => e.scalar_type.is_some(),
            _ => false,
        }
    }
}

/// Locate a plain `class` named `fqcn` defined in `file`. Returns `None`
/// if `file` doesn't define a class by that name (an interface / trait
/// / enum with that name is not returned — use [`find_class_like`] for
/// kind-agnostic lookup).
///
/// Tracked: result is memoized per `(file, fqcn)` pair and invalidated
/// when the file's text changes (via `collect_file_definitions`'s
/// dependency on `SourceFile::text`).
#[salsa::tracked]
pub fn class_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<ClassStorage>> {
    let defs = collect_file_definitions(db, file);
    let target = fqcn.name(db);
    defs.slice
        .classes
        .iter()
        .find(|c| c.fqcn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
}

/// Locate an `interface` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn interface_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<InterfaceStorage>> {
    let defs = collect_file_definitions(db, file);
    let target = fqcn.name(db);
    defs.slice
        .interfaces
        .iter()
        .find(|i| i.fqcn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
}

/// Locate a `trait` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn trait_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<TraitStorage>> {
    let defs = collect_file_definitions(db, file);
    let target = fqcn.name(db);
    defs.slice
        .traits
        .iter()
        .find(|t| t.fqcn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
}

/// Locate an `enum` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn enum_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<EnumStorage>> {
    let defs = collect_file_definitions(db, file);
    let target = fqcn.name(db);
    defs.slice
        .enums
        .iter()
        .find(|e| e.fqcn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
}

/// Locate a function named `fqn` defined in `file`.
#[salsa::tracked]
pub fn function_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqn: Fqcn<'db>,
) -> Option<Arc<FunctionStorage>> {
    let defs = collect_file_definitions(db, file);
    let target = fqn.name(db);
    defs.slice
        .functions
        .iter()
        .find(|f| f.fqn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
}

/// Locate a global constant `fqn` defined in `file`. Returns
/// `Option<Arc<Union>>` where `Union` is its inferred type.
#[salsa::tracked]
pub fn global_constant_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqn: Fqcn<'db>,
) -> Option<Arc<mir_types::Union>> {
    let defs = collect_file_definitions(db, file);
    let target = fqn.name(db);
    defs.slice
        .constants
        .iter()
        .find(|(name, _)| name.as_ref() == target.as_ref())
        .map(|(_, ty)| Arc::new(ty.clone()))
}

/// Composite: resolve `fqcn` to its defining file, then locate any
/// class-like definition (class / interface / trait / enum) within it.
///
/// **The headline pull-based lookup.** Demands `collect_file_definitions`
/// on the resolved file as a salsa tracked dependency — callers reading
/// this from a tracked context are correctly invalidated when either the
/// resolver or the defining file's text changes. No prior `ingest_file`
/// call is required: the file's text must be registered (via
/// `set_file_text` or `set_workspace_files`), but Pass-1 collection
/// happens on demand inside salsa.
/// Salsa-tracked per-(file, idx) class storage. One memo entry per distinct
/// class ever queried; subsequent calls return the same Arc cheaply.
#[salsa::tracked]
pub fn class_storage_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<ClassStorage>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.classes.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn interface_storage_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<InterfaceStorage>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.interfaces.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn trait_storage_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<TraitStorage>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.traits.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn enum_storage_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<EnumStorage>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.enums.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn function_storage_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<FunctionStorage>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.functions.get(idx as usize).cloned()
}

pub fn find_class_like<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<ClassLike> {
    use crate::db::SymbolLoc;
    // O(1) HashMap lookup in the workspace symbol index, then a per-(file, idx)
    // salsa-memoized fetch of the Arc<Storage>.
    //
    // `Symbol::ascii_lowercase` is memoized — first call per unique FQCN
    // allocates the lowercase string and interns it; subsequent calls hit a
    // process-global DashMap. The hot Pass-2 path becomes alloc-free after
    // warmup.
    let key = fqcn.name(db).ascii_lowercase();
    let index = crate::db::workspace_index(db);
    let loc = index.class_like.get(&key).copied()?;
    match loc {
        SymbolLoc::Class { file, idx } => {
            class_storage_at(db, file, idx as u32).map(ClassLike::Class)
        }
        SymbolLoc::Interface { file, idx } => {
            interface_storage_at(db, file, idx as u32).map(ClassLike::Interface)
        }
        SymbolLoc::Trait { file, idx } => {
            trait_storage_at(db, file, idx as u32).map(ClassLike::Trait)
        }
        SymbolLoc::Enum { file, idx } => enum_storage_at(db, file, idx as u32).map(ClassLike::Enum),
        SymbolLoc::Function { .. } | SymbolLoc::Constant { .. } => None,
    }
}

/// Composite: resolve `fqn` to its defining file, then locate the
/// function within it.
pub fn find_function<'db>(
    db: &'db dyn MirDatabase,
    fqn: Fqcn<'db>,
) -> Option<Arc<FunctionStorage>> {
    use crate::db::SymbolLoc;
    let key = fqn.name(db).ascii_lowercase();
    let index = crate::db::workspace_index(db);
    let SymbolLoc::Function { file, idx } = index.functions.get(&key).copied()? else {
        return None;
    };
    function_storage_at(db, file, idx as u32)
}

/// Composite: resolve `fqn` to its defining file, then locate a global
/// constant within it.
pub fn find_global_constant<'db>(
    db: &'db dyn MirDatabase,
    fqn: Fqcn<'db>,
) -> Option<Arc<mir_types::Union>> {
    use crate::db::SymbolLoc;
    let key = fqn.name(db);
    let index = crate::db::workspace_index(db);
    if let Some(SymbolLoc::Constant { file, idx }) = index.constants.get(&key).copied() {
        let defs = collect_file_definitions(db, file);
        if let Some((_, ty)) = defs.slice.constants.get(idx) {
            return Some(Arc::new(ty.clone()));
        }
    }
    let file = source_file_for_fqcn(db, fqn)?;
    global_constant_in_file(db, file, fqn)
}

/// Locate a method named `name` (case-insensitive PHP semantics) on the
/// class `fqcn`'s **own** methods only — no inheritance walk. Use
/// [`find_method_in_chain`] for the inherited variant.
///
/// For enums, also synthesizes the built-in `cases()`, `from()`, and
/// `tryFrom()` static methods that PHP provides at runtime but that the
/// collector does not emit.
pub fn find_method_in_class<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<Arc<MethodStorage>> {
    let class = find_class_like(db, fqcn)?;
    if let Some(m) = class.own_methods().iter().find_map(|(k, v)| {
        if k.as_ref().eq_ignore_ascii_case(name) {
            Some(v.clone())
        } else {
            None
        }
    }) {
        return Some(m);
    }
    // Synthesize PHP built-in enum static methods.
    if let ClassLike::Enum(e) = &class {
        let lower = name.to_ascii_lowercase();
        let is_backed = e.scalar_type.is_some();
        let synth = |method_name: &str| {
            Arc::new(mir_codebase::storage::MethodStorage {
                fqcn: e.fqcn.clone(),
                name: Arc::from(method_name),
                params: Arc::from([].as_ref()),
                return_type: Some(Arc::new(mir_types::Union::mixed())),
                inferred_return_type: None,
                visibility: mir_codebase::storage::Visibility::Public,
                is_static: true,
                is_abstract: false,
                is_constructor: false,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                is_final: false,
                is_virtual: false,
                is_internal: false,
                is_pure: false,
                deprecated: None,
                location: None,
                docstring: None,
            })
        };
        if lower == "cases" {
            return Some(synth("cases"));
        }
        if is_backed && (lower == "from" || lower == "tryfrom") {
            return Some(synth(name));
        }
    }
    None
}

/// Locate a property named `name` on the class `fqcn`'s **own**
/// properties only. Interface and enum return `None` (they don't carry
/// properties).
pub fn find_property_in_class<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<PropertyStorage> {
    let class = find_class_like(db, fqcn)?;
    class.own_properties()?.get(name).cloned()
}

/// Locate a class constant named `name` on the class `fqcn`'s **own**
/// constants only. For enums, also checks cases (which the collector stores
/// separately in `EnumStorage.cases`, not in `own_constants`).
pub fn find_class_constant_in_class<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<ConstantStorage> {
    let class = find_class_like(db, fqcn)?;
    if let Some(c) = class.own_constants().get(name) {
        return Some(c.clone());
    }
    // Enum cases live in EnumStorage.cases, not own_constants.
    if let ClassLike::Enum(e) = &class {
        if let Some(case) = e.cases.get(name) {
            return Some(mir_codebase::storage::ConstantStorage {
                name: case.name.clone(),
                ty: mir_types::Union::mixed(),
                visibility: None,
                is_final: false,
                location: case.location.clone(),
            });
        }
    }
    None
}

/// Walk the ancestor chain of `fqcn` (parent class + interfaces + traits,
/// transitively) and return ancestor FQCNs in BFS order. The first entry
/// is `fqcn` itself; the rest are parents, parents' parents, etc.
///
/// Cycle-safe via a visited set (PHP allows accidental cycles in `@extends`
/// docblocks; we treat them as terminated at the second visit).
///
/// Tracked: the walk is memoized per `fqcn`, so repeated lookups on
/// member-resolution paths don't re-traverse. Returns `Arc<[Arc<str>]>`
/// for cheap salsa identity comparison via ptr_eq.
#[salsa::tracked]
pub fn class_ancestors_by_fqcn<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Arc<[Arc<str>]> {
    let mut visited = std::collections::HashSet::<Arc<str>>::new();
    let mut order = Vec::<Arc<str>>::new();
    let mut queue = std::collections::VecDeque::<Arc<str>>::new();

    let initial: Arc<str> = fqcn.name(db).into();
    queue.push_back(initial.clone());
    visited.insert(initial);

    while let Some(name) = queue.pop_front() {
        order.push(name.clone());
        let here = Fqcn::new(db, Symbol::new(name.as_ref()));
        if let Some(class) = find_class_like(db, here) {
            for parent in class.ancestor_fqcns() {
                if visited.insert(parent.clone()) {
                    queue.push_back(parent);
                }
            }
        }
    }

    order.into()
}

/// Existence check for "does any ancestor of `fqcn` have a method named
/// `name`?". Used for magic-method dispatch checks (`__call`, `__callstatic`,
/// `__toString`, `__invoke`, `__get`, …) where callers only need a boolean.
pub fn has_method_in_chain(db: &dyn MirDatabase, fqcn: &str, name: &str) -> bool {
    let here = Fqcn::new(db, Symbol::new(fqcn));
    find_method_in_chain(db, here, name).is_some()
}

/// Walk the inheritance chain of `fqcn` and return the first method
/// matching `name` (case-insensitive PHP semantics), along with the FQCN
/// of the class that declared it. Also searches `@mixin` classes via a
/// separate cycle-safe walk so they don't pollute `has_unknown_ancestor_via_db`.
pub fn find_method_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, Arc<MethodStorage>)> {
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::new(db, Symbol::new(ancestor.as_ref()));
        if let Some(m) = find_method_in_class(db, here, name) {
            return Some((ancestor.clone(), m));
        }
    }
    // Separate @mixin walk — cycle-safe, depth-first.
    let mut visited_mixins = std::collections::HashSet::<Arc<str>>::new();
    find_method_in_mixins(db, fqcn, name, &mut visited_mixins)
}

fn find_method_in_mixins<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
    visited: &mut std::collections::HashSet<Arc<str>>,
) -> Option<(Arc<str>, Arc<MethodStorage>)> {
    let class = find_class_like(db, fqcn)?;
    for m in class.mixins() {
        let mixin_fqcn: Arc<str> = if let Some(pos) = m.find('<') {
            Arc::from(&m[..pos])
        } else {
            m.clone()
        };
        if !visited.insert(mixin_fqcn.clone()) {
            continue;
        }
        let mixin_here = Fqcn::new(db, Symbol::new(mixin_fqcn.as_ref()));
        // Walk the mixin's full inheritance chain.
        for ancestor in class_ancestors_by_fqcn(db, mixin_here).iter() {
            let here = Fqcn::new(db, Symbol::new(ancestor.as_ref()));
            if let Some(m) = find_method_in_class(db, here, name) {
                return Some((ancestor.clone(), m));
            }
        }
        // Recurse into the mixin's own mixins.
        if let Some(result) = find_method_in_mixins(db, mixin_here, name, visited) {
            return Some(result);
        }
    }
    None
}

/// Walk the inheritance chain of `fqcn` and return the first property
/// matching `name`, along with the FQCN of the class that declared it.
/// Properties are case-sensitive in PHP.
pub fn find_property_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, PropertyStorage)> {
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::new(db, Symbol::new(ancestor.as_ref()));
        if let Some(p) = find_property_in_class(db, here, name) {
            return Some((ancestor.clone(), p));
        }
    }
    None
}

/// Existence-check for "is `name` concretely implemented (non-abstract,
/// non-interface) somewhere reachable from `fqcn`'s inheritance chain?".
/// Used to flag UnimplementedAbstractMethod.
pub fn is_method_concretely_implemented(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> bool {
    let lower = method_name.to_lowercase();
    let here = Fqcn::from_str(db, fqcn);
    let Some(self_class) = find_class_like(db, here) else {
        return false;
    };
    if self_class.is_interface() {
        return false;
    }
    for ancestor_fqcn in class_ancestors_by_fqcn(db, here).iter() {
        let here2 = Fqcn::from_str(db, ancestor_fqcn.as_ref());
        let Some(class) = find_class_like(db, here2) else {
            continue;
        };
        if class.is_interface() {
            continue;
        }
        for (k, m) in class.own_methods().iter() {
            if k.as_ref().eq_ignore_ascii_case(&lower) && !m.is_abstract {
                return true;
            }
        }
    }
    false
}

/// Walk the inheritance chain of `fqcn` and return the first class
/// constant matching `name`.
pub fn find_class_constant_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, ConstantStorage)> {
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::new(db, Symbol::new(ancestor.as_ref()));
        if let Some(c) = find_class_constant_in_class(db, here, name) {
            return Some((ancestor.clone(), c));
        }
    }
    None
}
