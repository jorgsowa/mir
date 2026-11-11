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

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameResolveExt as _;
use std::sync::Arc;

use mir_codebase::definitions::{
    ClassDef, ConstantDef, DeclaredParam, EnumDef, FunctionDef, InterfaceDef, MethodDef,
    PropertyDef, TraitDef,
};
use mir_types::{Atomic, Name, Type};
use rustc_hash::FxHashMap;

use crate::db::{
    collect_file_definitions, source_file_for_fqcn, Fqcn, MirDatabase, SourceFile, SymbolLoc,
    WorkspaceSymbolIndex,
};

fn find_named_def<T>(
    defs: &[Arc<T>],
    target: &str,
    name_of: impl Fn(&T) -> &Arc<str>,
) -> Option<Arc<T>> {
    defs.iter()
        .find(|def| name_of(def.as_ref()).eq_ignore_ascii_case(target))
        .cloned()
}

fn analyzed_source_files(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<SourceFile> {
    let mut files: Vec<SourceFile> = if analyzed_files.is_empty() {
        db.all_source_files()
    } else {
        analyzed_files
            .iter()
            .filter_map(|p| db.lookup_source_file(p))
            .collect()
    };
    // Iterate files in a stable order so the FQCN sort below — which is stable
    // and therefore preserves input order on equal keys — yields a fully
    // deterministic result even when two files declare the same class name.
    files.sort_by_key(|a| a.path(db));
    files
}

fn collect_analyzed_defs<T, U>(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
    defs_of: impl Fn(&mir_codebase::definitions::StubSlice) -> &[Arc<T>],
    map: impl Fn(Arc<T>) -> U,
    fqcn_of: impl Fn(&T) -> &Arc<str>,
) -> Vec<(Arc<str>, U)> {
    let mut out = Vec::new();
    for sf in analyzed_source_files(db, analyzed_files) {
        let defs = collect_file_definitions(db, sf);
        for def in defs_of(&defs.slice) {
            out.push((fqcn_of(def.as_ref()).clone(), map(def.clone())));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn workspace_symbol_loc(
    db: &dyn MirDatabase,
    lookup: impl Fn(&WorkspaceSymbolIndex) -> Option<SymbolLoc>,
) -> Option<SymbolLoc> {
    match db.frozen_workspace_index() {
        Some(frozen) => lookup(frozen),
        None => lookup(crate::db::workspace_index(db)),
    }
}

/// Register the embedded stub at `path` on demand.
fn stub_on_demand(db: &dyn MirDatabase, path: &'static str) -> Option<SourceFile> {
    db.load_on_demand(path, salsa::Durability::HIGH, &|| {
        crate::stubs::stub_content_for_path(path).map(Arc::from)
    })
}

/// Anchors an on-demand lookup on the workspace revision, so a miss is
/// retried once the workspace changes.
fn read_workspace_revision(db: &dyn MirDatabase) {
    let _ = db.workspace_revision().revision(db);
}

/// Where class-like `fqcn` is declared when the symbol index doesn't know
/// it: the file the resolver maps it to, else its embedded stub, both
/// registered on demand. The order matches the index, which ranks user files
/// above native stubs (a vendor polyfill of `Stringable` wins). Loading this
/// way is a pure read, so it runs on any snapshot without the owner.
#[salsa::tracked]
fn class_like_loc_on_demand<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<SymbolLoc> {
    read_workspace_revision(db);
    let name = fqcn.name(db);
    let key = name.ascii_lowercase();
    let declared_in = |sf: SourceFile| {
        crate::db::collect_file_declarations(db, sf)
            .class_like()
            .find(|d| d.lookup_key() == key)
            .map(|d| d.loc)
    };
    resolver_file_on_demand(db, fqcn)
        .and_then(declared_in)
        .or_else(|| {
            let stub = stub_on_demand(db, crate::stubs::stub_path_for_class(name.as_str())?)?;
            declared_in(stub)
        })
}

/// Register the file the resolver maps `fqcn` to on demand.
fn resolver_file_on_demand(db: &dyn MirDatabase, fqcn: Fqcn<'_>) -> Option<SourceFile> {
    let path = crate::db::resolve_fqcn_to_path(db, fqcn).clone()?;
    // The path query backdates across a provider swap; anchor the read itself.
    let _ = db.resolver_config().revision(db);
    let provider = db.source_provider()?;
    db.load_on_demand(&path, crate::db::durability_for_path(&path), &|| {
        provider.read(&path)
    })
}

// Per-name projections of the symbol index. An index write re-runs only
// these lookups; unchanged ones backdate, so their dependents stay memoized.

#[salsa::tracked]
fn indexed_class_like_loc<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<SymbolLoc> {
    let key = fqcn.name(db).ascii_lowercase();
    workspace_symbol_loc(db, |index| index.class_like_loc(key))
}

#[salsa::tracked]
fn indexed_function_loc<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<SymbolLoc> {
    let key = fqn.name(db).ascii_lowercase();
    workspace_symbol_loc(db, |index| index.function_loc(key))
}

/// Constants are keyed case-sensitively, unlike class-likes and functions.
#[salsa::tracked]
fn indexed_constant_loc<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<SymbolLoc> {
    let key = *fqn.name(db);
    workspace_symbol_loc(db, |index| index.constant_loc(key))
}

/// Where a class-like, function or global constant named `fqn` is declared,
/// per the symbol index alone.
pub(crate) fn indexed_symbol_loc(db: &dyn MirDatabase, fqn: Fqcn<'_>) -> Option<SymbolLoc> {
    indexed_class_like_loc(db, fqn)
        .or_else(|| *indexed_function_loc(db, fqn))
        .or_else(|| *indexed_constant_loc(db, fqn))
}

/// Where class-like `fqcn` is declared: the symbol index, else on demand.
pub(crate) fn class_like_loc(db: &dyn MirDatabase, fqcn: Fqcn<'_>) -> Option<SymbolLoc> {
    indexed_class_like_loc(db, fqcn).or_else(|| *class_like_loc_on_demand(db, fqcn))
}

/// Where function `fqn` is declared: the symbol index, else on demand.
pub(crate) fn function_loc(db: &dyn MirDatabase, fqn: Fqcn<'_>) -> Option<SymbolLoc> {
    indexed_function_loc(db, fqn).or_else(|| *function_loc_on_demand(db, fqn))
}

/// Where global constant `fqn` is declared: the symbol index, else on demand.
pub(crate) fn constant_loc(db: &dyn MirDatabase, fqn: Fqcn<'_>) -> Option<SymbolLoc> {
    indexed_constant_loc(db, fqn).or_else(|| *constant_loc_on_demand(db, fqn))
}

/// Where a class-like, function or global constant named `symbol` is
/// declared, trying every kind in the index before loading any on demand.
pub(crate) fn symbol_loc(db: &dyn MirDatabase, symbol: &str) -> Option<SymbolLoc> {
    let fqn = Fqcn::from_str(db, symbol);
    indexed_symbol_loc(db, fqn)
        .or_else(|| *class_like_loc_on_demand(db, fqn))
        .or_else(|| *function_loc_on_demand(db, fqn))
        .or_else(|| *constant_loc_on_demand(db, fqn))
}

/// [`class_like_loc_on_demand`] for a built-in function's stub.
#[salsa::tracked]
fn function_loc_on_demand<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<SymbolLoc> {
    read_workspace_revision(db);
    let name = fqn.name(db);
    let key = name.ascii_lowercase();
    let path = crate::stubs::stub_path_for_function(name.as_str().trim_start_matches('\\'))?;
    crate::db::collect_file_declarations(db, stub_on_demand(db, path)?)
        .functions()
        .find(|d| d.lookup_key() == key)
        .map(|d| d.loc)
}

/// [`class_like_loc_on_demand`] for a built-in global constant's stub.
#[salsa::tracked]
fn constant_loc_on_demand<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<SymbolLoc> {
    read_workspace_revision(db);
    let name = fqn.name(db);
    let path = crate::stubs::stub_path_for_constant(name.as_str())?;
    crate::db::collect_file_declarations(db, stub_on_demand(db, path)?)
        .constants()
        .find(|d| d.lookup_key() == *name)
        .map(|d| d.loc)
}

fn class_like_from_loc(db: &dyn MirDatabase, loc: SymbolLoc) -> Option<ClassLike> {
    class_like_ref_from_loc(db, loc).map(ClassLikeRef::to_class_like)
}

fn class_like_ref_from_loc(db: &dyn MirDatabase, loc: SymbolLoc) -> Option<ClassLikeRef<'_>> {
    match loc {
        SymbolLoc::Class { file, idx } => {
            file_class_defs(db, file).get(idx).map(ClassLikeRef::Class)
        }
        SymbolLoc::Interface { file, idx } => file_interface_defs(db, file)
            .get(idx)
            .map(ClassLikeRef::Interface),
        SymbolLoc::Trait { file, idx } => {
            file_trait_defs(db, file).get(idx).map(ClassLikeRef::Trait)
        }
        SymbolLoc::Enum { file, idx } => file_enum_defs(db, file).get(idx).map(ClassLikeRef::Enum),
        SymbolLoc::Function { .. } | SymbolLoc::Constant { .. } => None,
    }
}

/// A memoized class-like borrowed from the database; reading it skips the
/// `Arc` refcount traffic that [`ClassLike`] costs on hot shared classes.
#[derive(Debug, Clone, Copy)]
pub enum ClassLikeRef<'db> {
    Class(&'db Arc<ClassDef>),
    Interface(&'db Arc<InterfaceDef>),
    Trait(&'db Arc<TraitDef>),
    Enum(&'db Arc<EnumDef>),
}

impl<'db> ClassLikeRef<'db> {
    pub fn to_class_like(self) -> ClassLike {
        match self {
            ClassLikeRef::Class(c) => ClassLike::Class(c.clone()),
            ClassLikeRef::Interface(i) => ClassLike::Interface(i.clone()),
            ClassLikeRef::Trait(t) => ClassLike::Trait(t.clone()),
            ClassLikeRef::Enum(e) => ClassLike::Enum(e.clone()),
        }
    }

    pub fn fqcn(self) -> &'db Arc<str> {
        match self {
            ClassLikeRef::Class(c) => &c.fqcn,
            ClassLikeRef::Interface(i) => &i.fqcn,
            ClassLikeRef::Trait(t) => &t.fqcn,
            ClassLikeRef::Enum(e) => &e.fqcn,
        }
    }

    pub fn own_methods(self) -> &'db mir_codebase::definitions::MemberMap<Arc<MethodDef>> {
        match self {
            ClassLikeRef::Class(c) => &c.own_methods,
            ClassLikeRef::Interface(i) => &i.own_methods,
            ClassLikeRef::Trait(t) => &t.own_methods,
            ClassLikeRef::Enum(e) => &e.own_methods,
        }
    }

    pub fn own_properties(self) -> &'db mir_codebase::definitions::MemberMap<PropertyDef> {
        match self {
            ClassLikeRef::Class(c) => &c.own_properties,
            ClassLikeRef::Interface(i) => &i.own_properties,
            ClassLikeRef::Trait(t) => &t.own_properties,
            ClassLikeRef::Enum(e) => &e.own_properties,
        }
    }

    pub fn template_params(self) -> &'db [mir_codebase::definitions::TemplateParam] {
        match self {
            ClassLikeRef::Class(c) => &c.template_params,
            ClassLikeRef::Interface(i) => &i.template_params,
            ClassLikeRef::Trait(t) => &t.template_params,
            ClassLikeRef::Enum(_) => &[],
        }
    }

    pub fn is_class(self) -> bool {
        matches!(self, ClassLikeRef::Class(_))
    }
}

fn global_constant_type_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: usize,
) -> Option<Arc<mir_types::Type>> {
    let defs = collect_file_definitions(db, file);
    defs.slice
        .constants
        .get(idx)
        .map(|(_, ty)| Arc::new(ty.clone()))
}

/// Tagged union over the four PHP class-like kinds. The result type of
/// composite `find_class_like` so callers receive a single response that
/// covers `class` / `interface` / `trait` / `enum`.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassLike {
    Class(Arc<ClassDef>),
    Interface(Arc<InterfaceDef>),
    Trait(Arc<TraitDef>),
    Enum(Arc<EnumDef>),
}

impl ClassLike {
    pub fn as_ref(&self) -> ClassLikeRef<'_> {
        match self {
            ClassLike::Class(c) => ClassLikeRef::Class(c),
            ClassLike::Interface(i) => ClassLikeRef::Interface(i),
            ClassLike::Trait(t) => ClassLikeRef::Trait(t),
            ClassLike::Enum(e) => ClassLikeRef::Enum(e),
        }
    }

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
    ///   - Class: `traits` (first, PHP precedence) + `parent` + `interfaces`
    ///   - Interface: `extends` (multi)
    ///   - Trait: used `traits`
    ///   - Enum: `interfaces`
    ///
    /// Traits come before the parent class so that DFS in
    /// `class_ancestors_by_fqcn` exhausts the full trait sub-tree before
    /// visiting the parent, matching PHP's rule that trait methods override
    /// inherited parent methods.
    ///
    /// `@mixin` FQCNs are intentionally excluded here — they are handled by
    /// `find_method_in_chain` via a separate cycle-safe walk so they don't
    /// affect `has_unknown_ancestor` checks.
    pub fn ancestor_fqcns(&self) -> Vec<Arc<str>> {
        match self {
            ClassLike::Class(c) => {
                let mut out = Vec::new();
                out.extend(c.traits.iter().cloned());
                if let Some(p) = &c.parent {
                    out.push(p.clone());
                }
                out.extend(c.interfaces.iter().cloned());
                out
            }
            ClassLike::Interface(i) => i.extends.clone(),
            ClassLike::Trait(t) => t.traits.clone(),
            ClassLike::Enum(e) => {
                let mut out = e.traits.clone();
                out.extend(e.interfaces.iter().cloned());
                out
            }
        }
    }

    /// Own methods (does not include inherited). Class / interface / trait
    /// / enum all carry these (interfaces hold abstract method signatures).
    pub fn own_methods(&self) -> &mir_codebase::definitions::MemberMap<Arc<MethodDef>> {
        match self {
            ClassLike::Class(c) => &c.own_methods,
            ClassLike::Interface(i) => &i.own_methods,
            ClassLike::Trait(t) => &t.own_methods,
            ClassLike::Enum(e) => &e.own_methods,
        }
    }

    /// Own properties. Interfaces and enums can both declare `@property*`
    /// docblock properties (no real storage, but still valid access
    /// targets), so both carry their own populated map too.
    pub fn own_properties(&self) -> Option<&mir_codebase::definitions::MemberMap<PropertyDef>> {
        match self {
            ClassLike::Class(c) => Some(&c.own_properties),
            ClassLike::Trait(t) => Some(&t.own_properties),
            ClassLike::Interface(i) => Some(&i.own_properties),
            ClassLike::Enum(e) => Some(&e.own_properties),
        }
    }

    /// Own constants.
    pub fn own_constants(&self) -> &mir_codebase::definitions::MemberMap<ConstantDef> {
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

    /// `use SomeTrait;` declarations on a class, trait, or enum body.
    /// Interfaces never have trait uses; they return an empty slice.
    pub fn class_traits(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Class(c) => &c.traits,
            ClassLike::Trait(t) => &t.traits,
            ClassLike::Enum(e) => &e.traits,
            ClassLike::Interface(_) => &[],
        }
    }

    /// `@mixin` FQCNs (class only).
    pub fn mixins(&self) -> &[Arc<str>] {
        match self {
            ClassLike::Class(c) => &c.mixins,
            _ => &[],
        }
    }

    pub fn mixin_locations(&self) -> &[mir_types::Location] {
        match self {
            ClassLike::Class(c) => &c.mixin_locations,
            _ => &[],
        }
    }

    /// `@psalm-import-type`/`@phpstan-import-type` declarations not resolved
    /// against a same-file source (class only). Each entry is `(local_name,
    /// original_name, from_fqcn)`.
    pub fn pending_import_types(&self) -> &[(Arc<str>, Arc<str>, Arc<str>)] {
        match self {
            ClassLike::Class(c) => &c.pending_import_types,
            _ => &[],
        }
    }

    /// Own `@psalm-type`/`@phpstan-type` aliases declared on this class-like's
    /// docblock, already fully fixpoint-expanded (chains like `A = B`, `B =
    /// SomeClass` resolve to `SomeClass` — see `ClassDef::type_aliases`).
    pub fn type_aliases(&self) -> &FxHashMap<Arc<str>, Type> {
        match self {
            ClassLike::Class(c) => &c.type_aliases,
            ClassLike::Interface(i) => &i.type_aliases,
            ClassLike::Trait(t) => &t.type_aliases,
            ClassLike::Enum(e) => &e.type_aliases,
        }
    }

    /// `@deprecated` docblock annotation, if present.
    pub fn deprecated(&self) -> Option<&Arc<str>> {
        match self {
            ClassLike::Class(c) => c.deprecated.as_ref(),
            ClassLike::Interface(i) => i.deprecated.as_ref(),
            ClassLike::Trait(t) => t.deprecated.as_ref(),
            ClassLike::Enum(e) => e.deprecated.as_ref(),
        }
    }

    /// Declared `@template` parameters.
    pub fn template_params(&self) -> &[mir_codebase::definitions::TemplateParam] {
        match self {
            ClassLike::Class(c) => &c.template_params,
            ClassLike::Interface(i) => &i.template_params,
            ClassLike::Trait(t) => &t.template_params,
            ClassLike::Enum(_) => &[],
        }
    }

    /// Source location of the declaration.
    pub fn location(&self) -> Option<&mir_types::Location> {
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
    pub fn enum_scalar_type(&self) -> Option<&mir_types::Type> {
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
    pub fn extends_type_args(&self) -> &[mir_types::Type] {
        match self {
            ClassLike::Class(c) => &c.extends_type_args,
            _ => &[],
        }
    }

    /// `@implements Iface<T1, T2>` type args (class or enum).
    pub fn implements_type_args(&self) -> &[(Arc<str>, Vec<mir_types::Type>)] {
        match self {
            ClassLike::Class(c) => &c.implements_type_args,
            ClassLike::Enum(e) => &e.implements_type_args,
            _ => &[],
        }
    }

    /// `@use TraitName<T1, T2>` type args (class, trait, or enum — anything
    /// that can `use` a trait).
    pub fn trait_use_type_args(&self) -> &[(Arc<str>, Vec<mir_types::Type>)] {
        match self {
            ClassLike::Class(c) => &c.trait_use_type_args,
            ClassLike::Trait(t) => &t.trait_use_type_args,
            ClassLike::Enum(e) => &e.trait_use_type_args,
            _ => &[],
        }
    }

    /// `@extends BaseIface<T1, T2>` type args, keyed by base interface FQCN
    /// (interface only — unlike a class's single-parent `extends_type_args`,
    /// an interface's native `extends A, B` clause may name several bases).
    pub fn interface_extends_type_args(&self) -> &[(Arc<str>, Vec<mir_types::Type>)] {
        match self {
            ClassLike::Interface(i) => &i.extends_type_args,
            _ => &[],
        }
    }

    /// Per-`use SomeTrait;` declaration locations (class + enum + trait).
    pub fn trait_use_locations(&self) -> &[(Arc<str>, mir_types::Location)] {
        match self {
            ClassLike::Class(c) => &c.trait_use_locations,
            ClassLike::Enum(e) => &e.trait_use_locations,
            ClassLike::Trait(t) => &t.trait_use_locations,
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

    /// Whether the class-like is marked `@api` / `@psalm-api`.
    pub fn is_api(&self) -> bool {
        match self {
            ClassLike::Class(c) => c.is_api,
            ClassLike::Interface(i) => i.is_api,
            ClassLike::Trait(t) => t.is_api,
            ClassLike::Enum(e) => e.is_api,
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
) -> Option<Arc<ClassDef>> {
    let defs = collect_file_definitions(db, file);
    find_named_def(&defs.slice.classes, fqcn.name(db).as_ref(), |c| &c.fqcn)
}

/// Locate an `interface` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn interface_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<InterfaceDef>> {
    let defs = collect_file_definitions(db, file);
    find_named_def(&defs.slice.interfaces, fqcn.name(db).as_ref(), |i| &i.fqcn)
}

/// Locate a `trait` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn trait_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<TraitDef>> {
    let defs = collect_file_definitions(db, file);
    find_named_def(&defs.slice.traits, fqcn.name(db).as_ref(), |t| &t.fqcn)
}

/// Locate an `enum` named `fqcn` defined in `file`.
#[salsa::tracked]
pub fn enum_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqcn: Fqcn<'db>,
) -> Option<Arc<EnumDef>> {
    let defs = collect_file_definitions(db, file);
    find_named_def(&defs.slice.enums, fqcn.name(db).as_ref(), |e| &e.fqcn)
}

/// Locate a function named `fqn` defined in `file`.
#[salsa::tracked]
pub fn function_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqn: Fqcn<'db>,
) -> Option<Arc<FunctionDef>> {
    let defs = collect_file_definitions(db, file);
    find_named_def(&defs.slice.functions, fqn.name(db).as_ref(), |f| &f.fqn)
}

/// Locate a global constant `fqn` defined in `file`. Returns
/// `Option<Arc<Type>>` where `Type` is its inferred type.
#[salsa::tracked]
pub fn global_constant_in_file<'db>(
    db: &'db dyn MirDatabase,
    file: SourceFile,
    fqn: Fqcn<'db>,
) -> Option<Arc<mir_types::Type>> {
    let defs = collect_file_definitions(db, file);
    let target = fqn.name(db);
    defs.slice
        .constants
        .iter()
        .find(|(name, _)| name.as_ref() == target.as_ref())
        .map(|(_, ty)| Arc::new(ty.clone()))
}

/// The classes `file` declares, indexed like its collected slice, with
/// cross-file imports resolved. Keyed by file alone: a single-input query
/// skips salsa's argument interner, a shared lock on every class lookup.
#[salsa::tracked(returns(ref))]
fn file_class_defs(db: &dyn MirDatabase, file: SourceFile) -> Vec<Arc<ClassDef>> {
    let defs = collect_file_definitions(db, file);
    let in_user_stub = db.is_user_stub_source_file(file);
    defs.slice
        .classes
        .iter()
        .map(|class| {
            let class = resolve_cross_file_imports(db, class.clone());
            match in_user_stub
                .then(|| real_class_shadowed_by_stub(db, &class))
                .flatten()
            {
                Some(real) => Arc::new(merge_stub_over_real(&class, &real)),
                None => class,
            }
        })
        .collect()
}

/// The project/vendor declaration of a class that a user stub also declares.
/// User stubs are usually partial (a handful of annotated members), so the
/// real class still contributes everything the stub leaves out.
fn real_class_shadowed_by_stub(db: &dyn MirDatabase, stub: &ClassDef) -> Option<Arc<ClassDef>> {
    let fqcn = Fqcn::from_str(db, stub.fqcn.as_ref());
    let real_file = resolver_file_on_demand(db, fqcn)?;
    let key = stub.fqcn.to_ascii_lowercase();
    let defs = collect_file_definitions(db, real_file);
    let real = defs
        .slice
        .classes
        .iter()
        .find(|c| c.fqcn.to_ascii_lowercase() == key)?;
    Some(resolve_cross_file_imports(db, real.clone()))
}

/// Stub members win; members only the real class declares are added.
fn merge_stub_over_real(stub: &ClassDef, real: &ClassDef) -> ClassDef {
    let mut merged = stub.clone();
    add_missing_members(&mut merged.own_methods, &real.own_methods);
    add_missing_members(&mut merged.own_properties, &real.own_properties);
    add_missing_members(&mut merged.own_constants, &real.own_constants);
    if merged.parent.is_none() {
        merged.parent = real.parent.clone();
    }
    add_missing_names(&mut merged.interfaces, &real.interfaces);
    add_missing_names(&mut merged.traits, &real.traits);
    add_missing_names(&mut merged.mixins, &real.mixins);
    merged
}

/// [`real_class_shadowed_by_stub`] for an interface.
fn real_interface_shadowed_by_stub(
    db: &dyn MirDatabase,
    stub: &InterfaceDef,
) -> Option<Arc<InterfaceDef>> {
    let fqcn = Fqcn::from_str(db, stub.fqcn.as_ref());
    let real_file = resolver_file_on_demand(db, fqcn)?;
    let key = stub.fqcn.to_ascii_lowercase();
    let defs = collect_file_definitions(db, real_file);
    let real = defs
        .slice
        .interfaces
        .iter()
        .find(|i| i.fqcn.to_ascii_lowercase() == key)?;
    Some(resolve_interface_cross_file_imports(db, real.clone()))
}

/// [`merge_stub_over_real`] for an interface.
fn merge_interface_stub_over_real(stub: &InterfaceDef, real: &InterfaceDef) -> InterfaceDef {
    let mut merged = stub.clone();
    add_missing_members(&mut merged.own_methods, &real.own_methods);
    add_missing_members(&mut merged.own_properties, &real.own_properties);
    add_missing_members(&mut merged.own_constants, &real.own_constants);
    add_missing_names(&mut merged.extends, &real.extends);
    merged
}

fn add_missing_members<V: Clone>(
    into: &mut mir_codebase::definitions::MemberMap<V>,
    from: &mir_codebase::definitions::MemberMap<V>,
) {
    for (name, member) in from.iter() {
        into.entry(name.clone()).or_insert_with(|| member.clone());
    }
}

fn add_missing_names(into: &mut Vec<Arc<str>>, from: &[Arc<str>]) {
    for name in from {
        if !into.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            into.push(name.clone());
        }
    }
}

/// Alias body declared on `from_class`, read from the raw collected slice (not
/// `file_class_defs`) so mutually-importing classes can't form a query cycle.
fn declared_type_alias(db: &dyn MirDatabase, from_class: &str, name: &str) -> Option<Type> {
    let loc = class_like_loc(db, Fqcn::from_str(db, from_class))?;
    let defs = collect_file_definitions(db, loc.file());
    let slice = &defs.slice;
    let aliases = match loc {
        SymbolLoc::Class { idx, .. } => &slice.classes.get(idx)?.type_aliases,
        SymbolLoc::Interface { idx, .. } => &slice.interfaces.get(idx)?.type_aliases,
        SymbolLoc::Trait { idx, .. } => &slice.traits.get(idx)?.type_aliases,
        SymbolLoc::Enum { idx, .. } => &slice.enums.get(idx)?.type_aliases,
        SymbolLoc::Function { .. } | SymbolLoc::Constant { .. } => return None,
    };
    aliases.get(name).cloned()
}

/// Follows an import chain (`A` imports from `B`, which re-imports from `C`).
/// An unknown source or alias is `mixed`: the missing class is reported
/// elsewhere, and a guessed type would only add false positives.
fn imported_alias_body(db: &dyn MirDatabase, from_class: &str, name: &str) -> Type {
    const MAX_CHAIN: usize = 8;
    let (mut from, mut name) = (from_class.to_string(), name.to_string());
    for _ in 0..MAX_CHAIN {
        let Some(body) = declared_type_alias(db, &from, &name) else {
            break;
        };
        match crate::collector::import_placeholder_parts(&body) {
            Some((next_from, next_name)) => {
                (from, name) = (next_from.to_string(), next_name.to_string())
            }
            None => return body,
        }
    }
    Type::mixed()
}

/// Source alias bodies for the import placeholders in `aliases`, keyed by
/// placeholder name so they can drive `expand_aliases_only`.
fn import_bodies(
    db: &dyn MirDatabase,
    aliases: &FxHashMap<Arc<str>, Type>,
) -> FxHashMap<String, Type> {
    aliases
        .values()
        .filter_map(crate::collector::import_placeholder_parts)
        .map(|(from, name)| {
            (
                format!(
                    "{}{from}::{name}",
                    crate::collector::IMPORT_PLACEHOLDER_PREFIX
                ),
                imported_alias_body(db, from, name),
            )
        })
        .collect()
}

fn expand_arc_type(ty: &mut Option<Arc<Type>>, bodies: &FxHashMap<String, Type>) {
    if let Some(arc) = ty {
        *arc = Arc::new(crate::collector::expand_aliases_only(
            (**arc).clone(),
            bodies,
        ));
    }
}

/// Defines `$name`, which swaps the `@psalm-import-type` placeholders the
/// collector left for imports from other files with the source alias's body,
/// in a class-like's alias map and member types.
macro_rules! resolve_cross_file_imports_fn {
    ($name:ident, $def:ty) => {
        fn $name(db: &dyn MirDatabase, def: Arc<$def>) -> Arc<$def> {
            let bodies = import_bodies(db, &def.type_aliases);
            if bodies.is_empty() {
                return def;
            }
            let expand =
                |ty: &mut Type| *ty = crate::collector::expand_aliases_only(ty.clone(), &bodies);

            let mut def = (*def).clone();
            def.type_aliases.values_mut().for_each(expand);
            for prop in def.own_properties.values_mut() {
                expand_arc_type(&mut prop.ty, &bodies);
            }
            for constant in def.own_constants.values_mut() {
                expand(&mut constant.ty);
            }
            for method in def.own_methods.values_mut() {
                let method = Arc::make_mut(method);
                expand_arc_type(&mut method.return_type, &bodies);
                let mut params = method.params.to_vec();
                for param in &mut params {
                    expand_arc_type(&mut param.ty, &bodies);
                }
                method.params = params.into();
            }
            Arc::new(def)
        }
    };
}

resolve_cross_file_imports_fn!(resolve_cross_file_imports, ClassDef);
resolve_cross_file_imports_fn!(resolve_interface_cross_file_imports, InterfaceDef);
resolve_cross_file_imports_fn!(resolve_trait_cross_file_imports, TraitDef);
resolve_cross_file_imports_fn!(resolve_enum_cross_file_imports, EnumDef);

/// Plain classes (not interfaces/traits/enums) defined in `analyzed_files`,
/// each materialized exactly once and the whole list sorted by FQCN for
/// deterministic issue order across runs.
///
/// Decomposes per file via the memoized `collect_file_definitions` query
/// rather than walking the global symbol index: in batch mode `analyzed_files`
/// is the project file set, so vendor / stub classes are never enumerated at
/// all (they aren't in the set). An empty `analyzed_files` means "all files"
/// — used by the `new()`/unit-test path — and falls back to every registered
/// source file. Incremental edits only recompute the touched files' slices.
pub fn analyzed_class_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, ClassLike)> {
    let mut classes = collect_analyzed_defs(
        db,
        analyzed_files,
        |slice| &slice.classes,
        ClassLike::Class,
        |class| &class.fqcn,
    );
    classes.retain(|(_, class)| !matches!(class, ClassLike::Class(c) if c.is_alias));
    classes
}

/// Like [`analyzed_class_defs`] but returns interface definitions from
/// analyzed files. Used to check `#[Override]` on interface methods.
pub fn analyzed_interface_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::InterfaceDef>)> {
    collect_analyzed_defs(
        db,
        analyzed_files,
        |slice| &slice.interfaces,
        |iface| iface,
        |iface| &iface.fqcn,
    )
}

/// Like [`analyzed_interface_defs`] but returns enum definitions from analyzed files.
pub fn analyzed_enum_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::EnumDef>)> {
    collect_analyzed_defs(db, analyzed_files, |slice| &slice.enums, |e| e, |e| &e.fqcn)
}

pub fn analyzed_trait_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::TraitDef>)> {
    collect_analyzed_defs(
        db,
        analyzed_files,
        |slice| &slice.traits,
        |t| t,
        |t| &t.fqcn,
    )
}

/// Every class-like declared in `analyzed_files`, from one pass over their
/// definitions, sorted by FQCN.
pub fn analyzed_class_likes(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, ClassLike)> {
    let mut out = Vec::new();
    for sf in analyzed_source_files(db, analyzed_files) {
        let slice = &collect_file_definitions(db, sf).slice;
        out.extend(
            slice
                .classes
                .iter()
                .filter(|d| !d.is_alias)
                .map(|d| (d.fqcn.clone(), ClassLike::Class(d.clone()))),
        );
        out.extend(
            slice
                .interfaces
                .iter()
                .map(|d| (d.fqcn.clone(), ClassLike::Interface(d.clone()))),
        );
        out.extend(
            slice
                .traits
                .iter()
                .map(|d| (d.fqcn.clone(), ClassLike::Trait(d.clone()))),
        );
        out.extend(
            slice
                .enums
                .iter()
                .map(|d| (d.fqcn.clone(), ClassLike::Enum(d.clone()))),
        );
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Free functions declared in `analyzed_files`, one entry per declaration
/// (a name declared under several `function_exists` guards appears once per site).
pub fn analyzed_function_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::FunctionDef>)> {
    collect_analyzed_defs(
        db,
        analyzed_files,
        |slice| &slice.functions,
        |f| f,
        |f| &f.fqn,
    )
}

/// [`file_class_defs`] for interfaces.
#[salsa::tracked(returns(ref))]
fn file_interface_defs(db: &dyn MirDatabase, file: SourceFile) -> Vec<Arc<InterfaceDef>> {
    let defs = collect_file_definitions(db, file);
    let in_user_stub = db.is_user_stub_source_file(file);
    defs.slice
        .interfaces
        .iter()
        .map(|def| {
            let def = resolve_interface_cross_file_imports(db, def.clone());
            match in_user_stub
                .then(|| real_interface_shadowed_by_stub(db, &def))
                .flatten()
            {
                Some(real) => Arc::new(merge_interface_stub_over_real(&def, &real)),
                None => def,
            }
        })
        .collect()
}

/// [`file_class_defs`] for traits.
#[salsa::tracked(returns(ref))]
fn file_trait_defs(db: &dyn MirDatabase, file: SourceFile) -> Vec<Arc<TraitDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice
        .traits
        .iter()
        .map(|def| resolve_trait_cross_file_imports(db, def.clone()))
        .collect()
}

/// [`file_class_defs`] for enums.
#[salsa::tracked(returns(ref))]
fn file_enum_defs(db: &dyn MirDatabase, file: SourceFile) -> Vec<Arc<EnumDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice
        .enums
        .iter()
        .map(|def| resolve_enum_cross_file_imports(db, def.clone()))
        .collect()
}

/// [`file_class_defs`] for free functions.
#[salsa::tracked(returns(ref))]
fn file_function_defs(db: &dyn MirDatabase, file: SourceFile) -> Vec<Arc<FunctionDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice
        .functions
        .iter()
        .map(|function| resolve_function_cross_file_imports(db, function.clone()))
        .collect()
}

/// Function `idx` of `file`'s collected slice, imports resolved.
pub fn function_def_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<FunctionDef>> {
    file_function_defs(db, file).get(idx as usize).cloned()
}

/// [`resolve_cross_file_imports`] for a free function's signature.
fn resolve_function_cross_file_imports(
    db: &dyn MirDatabase,
    function: Arc<FunctionDef>,
) -> Arc<FunctionDef> {
    let bodies = import_bodies(db, &function.type_aliases);
    if bodies.is_empty() {
        return function;
    }
    let mut function = (*function).clone();
    for ty in function.type_aliases.values_mut() {
        *ty = crate::collector::expand_aliases_only(ty.clone(), &bodies);
    }
    expand_arc_type(&mut function.return_type, &bodies);
    let mut params = function.params.to_vec();
    for param in &mut params {
        expand_arc_type(&mut param.ty, &bodies);
    }
    function.params = params.into();
    Arc::new(function)
}

/// Composite: resolve `fqcn` to its defining file, then locate any
/// class-like definition (class / interface / trait / enum) within it.
///
/// **The headline pull-based lookup.** Demands `collect_file_definitions`
/// on the resolved file as a salsa tracked dependency — callers reading
/// this from a tracked context are correctly invalidated when either the
/// resolver or the defining file's text changes. No prior `ingest_file`
/// call is required: a class the symbol index lacks is found in the file
/// the resolver maps it to, else its embedded stub, both loaded on demand.
pub fn find_class_like<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<ClassLike> {
    // O(1) index lookup (borrowing the frozen index on batch passes), then a
    // per-(file, idx) salsa-memoized fetch of the Arc<Storage>.
    class_like_by_name(db, fqcn).clone()
}

/// [`find_class_like`] borrowing the memoized definition instead of cloning it.
pub fn find_class_like_ref<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
) -> Option<ClassLikeRef<'db>> {
    class_like_by_name(db, fqcn).as_ref().map(ClassLike::as_ref)
}

/// The class-like `fqcn` names, memoized per name: one fetch per lookup
/// instead of one for its location and one for its file's definitions.
#[salsa::tracked(returns(ref))]
fn class_like_by_name<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<ClassLike> {
    class_like_loc(db, fqcn).and_then(|loc| class_like_from_loc(db, loc))
}

/// Whether the workspace symbol index already holds `fqcn`, as opposed to
/// [`find_class_like`] finding it on demand.
pub fn class_like_indexed(db: &dyn MirDatabase, fqcn: Fqcn<'_>) -> bool {
    indexed_class_like_loc(db, fqcn).is_some()
}

/// The file a class-like symbol is declared in, if known.
pub fn class_like_decl_file(db: &dyn MirDatabase, fqcn: Fqcn<'_>) -> Option<Arc<str>> {
    Some(class_like_loc(db, fqcn)?.file().path(db).clone())
}

/// Composite: resolve `fqn` to its defining file, then locate the
/// function within it.
pub fn find_function<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<Arc<FunctionDef>> {
    function_by_name(db, fqn).clone()
}

/// [`find_function`] memoized per name, like [`class_like_by_name`].
#[salsa::tracked(returns(ref))]
fn function_by_name<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<Arc<FunctionDef>> {
    let SymbolLoc::Function { file, idx } = function_loc(db, fqn)? else {
        return None;
    };
    function_def_at(db, file, idx as u32)
}

/// Where function `fqn` is declared as seen from `file`: a declaration in
/// `file` itself wins over the workspace-wide one, since standalone scripts
/// reuse function names.
pub(crate) fn function_loc_from(
    db: &dyn MirDatabase,
    file: &str,
    fqn: Fqcn<'_>,
) -> Option<SymbolLoc> {
    let global = function_loc(db, fqn)?;
    let Some(here) = db
        .lookup_source_file(file)
        .filter(|sf| *sf != global.file())
    else {
        return Some(global);
    };
    let key = fqn.name(db).ascii_lowercase();
    let own = file_function_locs(db, here).get(&key).copied();
    Some(own.unwrap_or(global))
}

/// The functions `file` declares, by lowercased name; the first declaration wins.
#[salsa::tracked(returns(ref))]
fn file_function_locs(db: &dyn MirDatabase, file: SourceFile) -> FxHashMap<Name, SymbolLoc> {
    let mut locs = FxHashMap::default();
    for decl in crate::db::collect_file_declarations(db, file).functions() {
        locs.entry(decl.lookup_key()).or_insert(decl.loc);
    }
    locs
}

/// [`find_function`] as seen from `file`, preferring a declaration in that file
/// over a workspace-wide declaration with the same name.
pub fn find_function_from<'db>(
    db: &'db dyn MirDatabase,
    file: &str,
    fqn: Fqcn<'db>,
) -> Option<Arc<FunctionDef>> {
    let SymbolLoc::Function { file, idx } = function_loc_from(db, file, fqn)? else {
        return None;
    };
    function_def_at(db, file, idx as u32)
}

/// Composite: resolve `fqn` to its defining file, then locate a global
/// constant within it.
pub fn find_global_constant<'db>(
    db: &'db dyn MirDatabase,
    fqn: Fqcn<'db>,
) -> Option<Arc<mir_types::Type>> {
    if let Some(SymbolLoc::Constant { file, idx }) = constant_loc(db, fqn) {
        if let Some(ty) = global_constant_type_at(db, file, idx) {
            return Some(ty);
        }
    }
    let file = source_file_for_fqcn(db, fqn)?;
    global_constant_in_file(db, file, fqn).clone()
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
) -> Option<Arc<MethodDef>> {
    let class = find_class_like_ref(db, fqcn)?;
    // Keys are lowercase-normalized at collection time, so one lowercase of
    // the query gives an O(1) hashed lookup instead of a scan of all methods.
    let lower: std::borrow::Cow<str> = if name.bytes().any(|b| b.is_ascii_uppercase()) {
        std::borrow::Cow::Owned(name.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(name)
    };
    if let Some(m) = class.own_methods().get(lower.as_ref()) {
        return Some(m.clone());
    }
    // Synthesize PHP built-in enum static methods.
    if let ClassLikeRef::Enum(e) = class {
        let is_backed = e.scalar_type.is_some();
        if lower == "cases" {
            let enum_ty = mir_types::Type::single(Atomic::TNamedObject {
                fqcn: Name::new(e.fqcn.as_ref()),
                type_params: mir_types::union::empty_type_params(),
            });
            let cases_return = mir_types::Type::single(Atomic::TList {
                value: Box::new(enum_ty),
            });
            return Some(Arc::new(mir_codebase::definitions::MethodDef {
                fqcn: e.fqcn.clone(),
                name: Arc::from("cases"),
                params: Arc::from([].as_ref()),
                return_type: Some(Arc::new(cases_return)),
                inferred_return_type: None,
                native_return: None,
                visibility: mir_codebase::definitions::Visibility::Public,
                is_static: true,
                is_abstract: false,
                is_constructor: false,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                is_final: false,
                is_virtual: false,
                is_internal: false,
                internal_scope: None,
                is_pure: false,
                is_impure: false,
                no_named_arguments: false,
                is_override: false,
                no_discard: None,
                deprecated: None,
                location: None,
                docstring: None,
                tag_descriptions: None,
                taint_sink_params: vec![],
                is_taint_source: false,
                taint: None,
                if_this_is: None,
                self_out: None,
                is_inherit_doc: false,
                returns_by_ref: false,
                is_mutation_free: false,
                is_external_mutation_free: false,
                inferred_mutation_free: false,
                data_provider_targets: vec![],
            }));
        }
        if is_backed && (lower == "from" || lower == "tryfrom") {
            let value_param = DeclaredParam {
                closure_this: None,
                name: Name::from("value"),
                ty: e.scalar_type.as_ref().map(|t| Arc::new(t.clone())),
                out_ty: None,
                doc_type_raw: None,
                doc_type_file: None,
                has_default: false,
                default_text: None,
                is_variadic: false,
                is_byref: false,
                is_optional: false,
            };
            // Use canonical PHP casing for the synthesized method name so that
            // case-sensitivity checks compare against the correct form.
            let canonical_name = if lower == "tryfrom" {
                "tryFrom"
            } else {
                "from"
            };
            let enum_ty = mir_types::Type::single(Atomic::TNamedObject {
                fqcn: Name::new(e.fqcn.as_ref()),
                type_params: mir_types::union::empty_type_params(),
            });
            let return_ty = if lower == "tryfrom" {
                let mut t = enum_ty;
                t.add_type(Atomic::TNull);
                t
            } else {
                enum_ty
            };
            return Some(Arc::new(mir_codebase::definitions::MethodDef {
                fqcn: e.fqcn.clone(),
                name: Arc::from(canonical_name),
                params: Arc::from(vec![value_param]),
                return_type: Some(Arc::new(return_ty)),
                inferred_return_type: None,
                native_return: None,
                visibility: mir_codebase::definitions::Visibility::Public,
                is_static: true,
                is_abstract: false,
                is_constructor: false,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                is_final: false,
                is_virtual: false,
                is_internal: false,
                internal_scope: None,
                is_pure: false,
                is_impure: false,
                no_named_arguments: false,
                is_override: false,
                no_discard: None,
                deprecated: None,
                location: None,
                docstring: None,
                tag_descriptions: None,
                taint_sink_params: vec![],
                is_taint_source: false,
                taint: None,
                if_this_is: None,
                self_out: None,
                is_inherit_doc: false,
                returns_by_ref: false,
                is_mutation_free: false,
                is_external_mutation_free: false,
                inferred_mutation_free: false,
                data_provider_targets: vec![],
            }));
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
) -> Option<PropertyDef> {
    let class = find_class_like_ref(db, fqcn)?;
    class.own_properties().get(name).cloned()
}

/// Locate a class constant named `name` on the class `fqcn`'s **own**
/// constants only. For enums, also checks cases (which the collector stores
/// separately in `EnumDef.cases`, not in `own_constants`).
pub fn find_class_constant_in_class<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<ConstantDef> {
    let class = find_class_like(db, fqcn)?;
    if let Some(c) = class.own_constants().get(name) {
        return Some(c.clone());
    }
    // Enum cases live in EnumDef.cases, not own_constants.
    if let ClassLike::Enum(e) = &class {
        if let Some(case) = e.cases.get(name) {
            return Some(mir_codebase::definitions::ConstantDef {
                name: case.name.clone(),
                ty: mir_types::Type::single(Atomic::TNamedObject {
                    fqcn: Name::new(e.fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                }),
                literal_value: None,
                visibility: None,
                is_final: false,
                location: case.location.clone(),
                deprecated: case.deprecated.clone(),
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
    // DFS (stack) so the full trait sub-tree is exhausted before the parent
    // class is visited. Combined with `ancestor_fqcns` returning traits before
    // the parent, this matches PHP's rule: trait methods take priority over
    // inherited parent methods.
    let mut stack = Vec::<Arc<str>>::new();

    let initial: Arc<str> = (*fqcn.name(db)).into();
    stack.push(initial.clone());
    visited.insert(initial);

    while let Some(name) = stack.pop() {
        order.push(name.clone());
        let here = Fqcn::interned(db, Name::new(name.as_ref()));
        if let Some(class) = find_class_like(db, here) {
            // Push in reverse so the first ancestor in the list ends up on
            // top of the stack and is visited next (LIFO / pre-order DFS).
            for parent in class.ancestor_fqcns().into_iter().rev() {
                if visited.insert(parent.clone()) {
                    stack.push(parent);
                }
            }
        }
    }

    order.into()
}

/// Whether `fqcn` or any class-like it extends, implements or uses declares `@template` params.
#[salsa::tracked]
pub fn ancestry_declares_templates<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> bool {
    let initial: Arc<str> = (*fqcn.name(db)).into();
    let mut visited = rustc_hash::FxHashSet::<Arc<str>>::default();
    let mut stack = vec![initial];
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(class) = find_class_like(db, Fqcn::interned(db, Name::new(name.as_ref()))) else {
            continue;
        };
        if !class.template_params().is_empty() {
            return true;
        }
        stack.extend(class.ancestor_fqcns());
        stack.extend(class.interfaces().iter().cloned());
        stack.extend(class.extends().iter().cloned());
        stack.extend(class.class_traits().iter().cloned());
        stack.extend(class.parent().cloned());
        for (target, _) in class
            .implements_type_args()
            .iter()
            .chain(class.interface_extends_type_args())
            .chain(class.trait_use_type_args())
        {
            stack.push(target.clone());
        }
    }
    false
}

/// Array-literal property defaults declared directly on `fqcn` (own class,
/// not ancestors). Each entry is a property whose declared default is an
/// array literal, with its `(key, value)` pairs flattened to strings: string
/// literals unquoted, `Foo::class` resolved to a FQCN, positional (list)
/// entries keyed by their index.
///
/// Lazily parses the defining file — only the class-property-provider plugin
/// path (`ExpressionAnalyzer::class_property_from_plugin`) demands it, so the
/// parse cost is bounded to classes that actually miss a property under a
/// registered marker ancestor (e.g. Eloquent models). Salsa memoizes per `fqcn`.
#[salsa::tracked]
pub fn class_array_property_defaults<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
) -> Arc<Vec<mir_plugin::ArrayPropertyDefault>> {
    use php_ast::owned::{ClassMemberKind, ExprKind, StmtKind};

    let empty = Arc::new(Vec::new());
    let Some(file) = source_file_for_fqcn(db, fqcn) else {
        return empty;
    };
    let path = file.path(db);
    let target = fqcn.name(db);
    let parsed = crate::db::parse_file(db, file);
    let program = &parsed.0.program;

    let expr_to_string = |e: &php_ast::owned::Expr| -> Option<String> {
        match &e.kind {
            ExprKind::String(s) => Some(s.as_str().to_string()),
            ExprKind::Int(n) => Some(n.to_string()),
            ExprKind::ClassConstAccess(cca) => {
                match (
                    cca.class.kind.resolve_name_ref(db, path.as_ref()),
                    &cca.member.kind,
                ) {
                    (Some(cls), ExprKind::Identifier(m)) if m.as_ref() == "class" => Some(cls),
                    _ => None,
                }
            }
            _ => None,
        }
    };

    let mut result: Vec<mir_plugin::ArrayPropertyDefault> = Vec::new();
    crate::body_analysis::for_each_file_scope_decl(&program.stmts, &mut |stmt| {
        if !result.is_empty() {
            return;
        }
        let StmtKind::Class(decl) = &stmt.kind else {
            return;
        };
        let Some(name) = decl.name.as_ref().and_then(|n| n.as_str()) else {
            return;
        };
        let _scope = crate::db::ns_scope::enter(db, path, stmt.span.start);
        if !crate::db::prefix_file_namespace(db, path.as_ref(), name)
            .eq_ignore_ascii_case(target.as_str())
        {
            return;
        }
        for member in decl.body.members.iter() {
            let ClassMemberKind::Property(p) = &member.kind else {
                continue;
            };
            let (Some(prop_name), Some(default)) = (p.name.as_str(), p.default.as_ref()) else {
                continue;
            };
            let ExprKind::Array(elements) = &default.kind else {
                continue;
            };
            let mut entries = Vec::new();
            for (idx, el) in elements.iter().enumerate() {
                let key = match el.key.as_ref() {
                    Some(k) => match expr_to_string(k) {
                        Some(s) => s,
                        None => continue,
                    },
                    None => idx.to_string(),
                };
                let Some(value) = expr_to_string(el.value()) else {
                    continue;
                };
                entries.push((key, value));
            }
            result.push(mir_plugin::ArrayPropertyDefault {
                property: prop_name.to_string(),
                entries,
            });
        }
    });

    Arc::new(result)
}

/// Existence check for "does any ancestor of `fqcn` have a method named
/// `name`?". Used for magic-method dispatch checks (`__call`, `__callstatic`,
/// `__toString`, `__invoke`, `__get`, …) where callers only need a boolean.
pub fn has_method_in_chain(db: &dyn MirDatabase, fqcn: &str, name: &str) -> bool {
    let here = Fqcn::interned(db, Name::new(fqcn));
    find_method_in_chain(db, here, name).is_some()
}

/// Walk the inheritance chain of `fqcn` and return the first method
/// matching `name` (case-insensitive PHP semantics), along with the FQCN
/// of the class that declared it. Also searches `@mixin` classes via a
/// separate cycle-safe walk so they don't pollute `has_unknown_ancestor`.
pub fn find_method_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
    method_in_chain(db, fqcn, Name::new(name)).clone()
}

/// [`find_method_in_chain`] memoized per class and name as written.
#[salsa::tracked(returns(ref))]
fn method_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: Name,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
    let name = name.as_str();
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
        if let Some(m) = find_method_in_class(db, here, name) {
            return Some((ancestor.clone(), m));
        }
    }
    // Separate @mixin walk — cycle-safe, depth-first.
    let mut visited_mixins = std::collections::HashSet::<Arc<str>>::new();
    find_method_in_mixins(db, fqcn, name, &mut visited_mixins)
}

/// Whether a method is pure by its own declaration or by an inheritable
/// ancestor contract. A public/protected `@pure` method remains pure when an
/// implementation overrides it: callers may hold the ancestor type, and the
/// implementation must be checked against that promise. Private ancestor
/// methods are deliberately excluded because they are not overridden in PHP.
pub fn method_is_pure_in_chain(db: &dyn MirDatabase, fqcn: Fqcn<'_>, name: &str) -> bool {
    // Constructors are not overrides, so a constructor declared on this class
    // must not inherit an ancestor's purity contract. A class that does not
    // declare one, however, executes its inherited constructor at runtime;
    // preserve that implementation's purity for calls such as
    // `parent::__construct()` through an intermediate exception class.
    if name.eq_ignore_ascii_case("__construct") {
        return find_method_in_class(db, fqcn, name)
            .map(|method| method.is_pure && !method.is_impure)
            .or_else(|| {
                find_method_in_chain(db, fqcn, name)
                    .map(|(_, method)| method.is_pure && !method.is_impure)
            })
            .unwrap_or(false);
    }

    let lower: std::borrow::Cow<str> = if name.bytes().any(|b| b.is_ascii_uppercase()) {
        std::borrow::Cow::Owned(name.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(name)
    };
    chain_method_purity(db, fqcn)
        .get(lower.as_ref())
        .copied()
        .unwrap_or(false)
}

/// Lowercased method names whose purity `fqcn`'s ancestor chain decides: the
/// nearest `@impure` or inheritable `@pure` declaration wins.
#[salsa::tracked(returns(ref))]
fn chain_method_purity<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
) -> Arc<FxHashMap<Arc<str>, bool>> {
    let mut purity = FxHashMap::default();
    let mut user_overridden: rustc_hash::FxHashSet<Arc<str>> = Default::default();
    for (depth, ancestor) in class_ancestors_by_fqcn(db, fqcn).iter().enumerate() {
        let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
        let Some(class) = find_class_like_ref(db, here) else {
            continue;
        };
        for (name, method) in class.own_methods() {
            let from_stub = method
                .location
                .as_ref()
                .is_some_and(|loc| crate::stubs::is_stub_path(loc.file.as_ref()));
            if !from_stub {
                user_overridden.insert(name.clone());
            } else if user_overridden.contains(name) {
                // A stub's `#[Pure]` is an IDE hint about the builtin, not a contract on overrides.
                continue;
            }
            // An explicit impure declaration is more specific than every
            // inherited purity annotation. It is commonly used by adapters
            // that deliberately implement a generally pure interface with an
            // effectful legacy implementation.
            let decided = if method.is_impure {
                false
            } else if method.is_pure
                && (depth == 0
                    || method.visibility != mir_codebase::definitions::Visibility::Private)
            {
                true
            } else {
                continue;
            };
            purity.entry(name.clone()).or_insert(decided);
        }
    }
    Arc::new(purity)
}

/// If `method` has `@inheritDoc`, walks the ancestor chain of `receiver_fqcn`
/// (the class where analysis is happening, which may differ from `owner_fqcn`
/// when the method is pulled in from a trait) to find the best docblock parent.
///
/// Resolution strategy:
/// 1. Skip `receiver_fqcn` itself and the immediate `owner_fqcn` (the trait/class
///    that declares the `@inheritdoc` method).
/// 2. Prefer the **first** ancestor that has a `from_docblock` return type —
///    this skips intermediate `@inheritdoc` hops in multi-level chains.
/// 3. Fall back to the first ancestor that has any declared return type.
pub fn find_inheritdoc_parent<'db>(
    db: &'db dyn MirDatabase,
    receiver_fqcn: Fqcn<'db>,
    owner_fqcn: Fqcn<'db>,
    method_name_lower: &str,
    method: &MethodDef,
) -> Option<Arc<MethodDef>> {
    if !method.is_inherit_doc {
        return None;
    }
    let receiver_name = receiver_fqcn.name(db);
    let owner_name = owner_fqcn.name(db);
    let ancestors = class_ancestors_by_fqcn(db, receiver_fqcn);

    // First pass: find the nearest ancestor with a docblock @return.
    let mut first_any: Option<Arc<MethodDef>> = None;
    for ancestor in ancestors.iter() {
        let anc = ancestor.as_ref();
        if anc == receiver_name.as_str() || anc == owner_name.as_str() {
            continue;
        }
        let anc_fqcn = Fqcn::interned(db, Name::new(anc));
        if let Some(m) = find_method_in_class(db, anc_fqcn, method_name_lower) {
            if m.return_type
                .as_deref()
                .map(|t| t.from_docblock)
                .unwrap_or(false)
            {
                return Some(m);
            }
            if first_any.is_none() {
                first_any = Some(m);
            }
        }
    }
    first_any
}

/// The parent param `own[i]` inherits under `@inheritDoc`: the same-named one, else the one
/// at the same position unless that name moved elsewhere in `own` (an inserted param).
pub fn inheritdoc_param<'p>(
    parent: &'p [mir_codebase::definitions::DeclaredParam],
    own: &[mir_codebase::definitions::DeclaredParam],
    i: usize,
) -> Option<&'p mir_codebase::definitions::DeclaredParam> {
    let name = &own.get(i)?.name;
    parent.iter().find(|p| p.name == *name).or_else(|| {
        parent
            .get(i)
            .filter(|p| !own.iter().any(|o| o.name == p.name))
    })
}

/// Docblock parent of an override that declares no docblock types of its own
/// (no `@inheritDoc`): the nearest ancestor method whose docblock `@return`
/// refines the same bare class the override returns natively
/// (`: Result` vs `@return Result<U, E>`), or whose conditional `@return` fits
/// the native return. Narrower or unrelated native return types never inherit.
pub fn find_implicit_doc_parent<'db>(
    db: &'db dyn MirDatabase,
    receiver_fqcn: Fqcn<'db>,
    owner_fqcn: Fqcn<'db>,
    method_name_lower: &str,
    method: &MethodDef,
) -> Option<Arc<MethodDef>> {
    let own_return = method.return_type.as_deref()?;
    let own_class = match own_return.types.as_slice() {
        [Atomic::TNamedObject { fqcn, type_params }] if type_params.is_empty() => Some(fqcn),
        [Atomic::TNamedObject { .. }] => return None,
        _ => None,
    };
    let has_own_docblock = own_return.from_docblock
        || !method.template_params.is_empty()
        || method
            .params
            .iter()
            .any(|p| p.ty.as_deref().is_some_and(|t| t.from_docblock));
    if method.is_inherit_doc || has_own_docblock {
        return None;
    }
    let receiver_name = receiver_fqcn.name(db);
    let owner_name = owner_fqcn.name(db);
    class_ancestors_by_fqcn(db, receiver_fqcn)
        .iter()
        .filter(|anc| anc.as_ref() != receiver_name.as_str() && anc.as_ref() != owner_name.as_str())
        .find_map(|anc| {
            let m = find_method_in_class(
                db,
                Fqcn::interned(db, Name::new(anc.as_ref())),
                method_name_lower,
            )?;
            if m.visibility == mir_codebase::definitions::Visibility::Private {
                return None;
            }
            let parent_return = m.return_type.as_deref().filter(|t| t.from_docblock)?;
            match parent_return.types.as_slice() {
                [Atomic::TNamedObject {
                    fqcn: parent_class, ..
                }] => own_class
                    .is_some_and(|own| parent_class.eq_ignore_ascii_case(own))
                    .then_some(m),
                // A conditional `@return` is kept when every branch fits the native return.
                [Atomic::TConditional { .. }] => {
                    let bounds: FxHashMap<Name, Type> = m
                        .template_params
                        .iter()
                        .map(|tp| {
                            let bound = tp.bound.as_deref().cloned();
                            (
                                Name::from(tp.name.as_ref()),
                                bound.unwrap_or_else(Type::mixed),
                            )
                        })
                        .collect();
                    let widened = parent_return
                        .substitute_templates(&bounds)
                        .resolve_conditional_returns(|_| None);
                    crate::subtype::is_subtype(db, &widened, own_return).then_some(m)
                }
                _ => None,
            }
        })
}

/// Walk the inheritance chain of `fqcn` respecting `insteadof` trait precedence
/// rules. When a class declares `use A, B { B::hello insteadof A; }`, this
/// function skips `A::hello` and returns `B::hello` instead.
///
/// Falls back to the plain [`find_method_in_chain`] walk for non-class kinds
/// (traits, interfaces) and includes `@mixin` resolution.
pub fn find_method_respecting_precedence<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
    method_by_precedence(db, fqcn, Name::new(name)).clone()
}

/// [`find_method_respecting_precedence`] memoized per class and name as written.
#[salsa::tracked(returns(ref))]
fn method_by_precedence<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: Name,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
    let name = name.as_str();
    let lower = name.to_ascii_lowercase();
    let mut visited = std::collections::HashSet::<Arc<str>>::new();
    walk_method_with_precedence(db, fqcn, &lower, &mut visited).or_else(|| {
        // @mixin fallback — same as find_method_in_chain
        let mut visited_mixins = std::collections::HashSet::<Arc<str>>::new();
        find_method_in_mixins(db, fqcn, name, &mut visited_mixins)
    })
}

fn walk_method_with_precedence<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    method_lower: &str,
    visited: &mut std::collections::HashSet<Arc<str>>,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
    let class_name: Arc<str> = (*fqcn.name(db)).into();
    if !visited.insert(class_name.clone()) {
        return None;
    }
    let class = find_class_like_ref(db, fqcn)?;

    // Check this class/trait's own methods first.
    if let Some(m) = find_method_in_class(db, fqcn, method_lower) {
        // Return the class's own canonically-cased FQCN, not the (possibly
        // differently-cased) FQCN string the caller queried with — PHP class
        // names are case-insensitive, but posting keys must be canonical.
        return Some((class.fqcn().clone(), m));
    }

    // For a plain class: respect its insteadof exclusions when walking its traits.
    if let ClassLikeRef::Class(cls) = class {
        // Check trait aliases: `use Trait { orig_method as alias_name; }`
        if let Some((opt_trait_fqcn, orig_method, vis_override, alias_cased)) =
            cls.trait_aliases.get(method_lower)
        {
            let search_traits: Vec<Arc<str>> = if let Some(tfqcn) = opt_trait_fqcn {
                vec![tfqcn.clone()]
            } else {
                cls.traits.clone()
            };
            for trait_fqcn in &search_traits {
                let here = Fqcn::interned(db, Name::new(trait_fqcn.as_ref()));
                if let Some((_trait_fqcn, m)) =
                    walk_method_with_precedence(db, here, orig_method, visited)
                {
                    let mut m_clone = (*m).clone();
                    // Use the alias name (original PHP casing) so WrongCaseMethod checks
                    // and error messages use the alias, not the original trait method name.
                    m_clone.name = alias_cased.clone();
                    // Apply the visibility override declared in `foo as private alias`.
                    if let Some(vis) = vis_override {
                        m_clone.visibility = *vis;
                    }
                    // Return the declaring class as owner so visibility checks use
                    // the class that declared the alias (not the trait).
                    return Some((class.fqcn().clone(), Arc::new(m_clone)));
                }
            }
        }

        let excluded: std::collections::HashSet<Arc<str>> = cls
            .trait_insteadof
            .get(method_lower)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default();

        for trait_fqcn in cls.traits.iter() {
            if excluded.contains(trait_fqcn) {
                continue;
            }
            let here = Fqcn::interned(db, Name::new(trait_fqcn.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        if let Some(ref parent) = cls.parent {
            let here = Fqcn::interned(db, Name::new(parent.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        for iface_fqcn in cls.interfaces.iter() {
            let here = Fqcn::interned(db, Name::new(iface_fqcn.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        return None;
    }

    // For traits and interfaces: plain ancestor walk (no per-class insteadof).
    for ancestor_fqcn in class.to_class_like().ancestor_fqcns() {
        let here = Fqcn::interned(db, Name::new(ancestor_fqcn.as_ref()));
        if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
            return Some(result);
        }
    }
    None
}

fn find_method_in_mixins<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
    visited: &mut std::collections::HashSet<Arc<str>>,
) -> Option<(Arc<str>, Arc<MethodDef>)> {
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
        let mixin_here = Fqcn::interned(db, Name::new(mixin_fqcn.as_ref()));
        // Walk the mixin's full inheritance chain.
        for ancestor in class_ancestors_by_fqcn(db, mixin_here).iter() {
            let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
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
/// Properties are case-sensitive in PHP. Also searches `@mixin` classes via a
/// separate cycle-safe walk so they don't pollute `has_unknown_ancestor`.
pub fn find_property_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, PropertyDef)> {
    property_in_chain(db, fqcn, Name::new(name)).clone()
}

/// [`find_property_in_chain`] memoized per class and name.
#[salsa::tracked(returns(ref))]
fn property_in_chain<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: Name,
) -> Option<(Arc<str>, PropertyDef)> {
    let (owner, mut prop) = find_property_in_chain_uncached(db, fqcn, name.as_str())?;
    // A property's `@var` docblock type may carry a bare well-known-global-
    // builtin name purely because of `collector::resolution`'s leniency
    // (P28's class-member sibling to P27's param/return one) — see
    // `crate::util::reconcile_property_ty_against_native`'s doc comment.
    // Reconciled centrally here, the single choke point every property-type
    // consumer (flow analysis, member lookup, assignment checks) reads
    // through.
    if let Some(ty) = prop.ty.take() {
        prop.ty = Some(Arc::new(crate::util::reconcile_property_ty_against_native(
            (*ty).clone(),
            prop.native_ty.as_deref(),
        )));
    }
    Some((owner, prop))
}

fn find_property_in_chain_uncached<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> Option<(Arc<str>, PropertyDef)> {
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
        if let Some(p) = find_property_in_class(db, here, name) {
            return Some((ancestor.clone(), p));
        }
    }
    // Separate @mixin walk — cycle-safe, depth-first.
    let mut visited_mixins = std::collections::HashSet::<Arc<str>>::new();
    find_property_in_mixins(db, fqcn, name, &mut visited_mixins)
}

/// Whether `name` is part of `fqcn`'s own composition — declared directly on
/// `fqcn`, or pulled in transitively through a trait `fqcn` (or one of its
/// traits) `use`s. Never crosses an `extends` boundary, unlike
/// [`find_property_in_chain`] — traits are PHP copy-paste semantics, so a
/// trait-contributed property is initializable from the *consuming* class's
/// own scope, while a genuinely inherited property (declared on a real
/// ancestor class) is not. Used to scope readonly-property initialization.
pub fn property_in_own_composition<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
) -> bool {
    if find_property_in_class(db, fqcn, name).is_some() {
        return true;
    }
    let mut visited = std::collections::HashSet::<Arc<str>>::new();
    let mut stack: Vec<Arc<str>> = Vec::new();
    if let Some(class) = find_class_like(db, fqcn) {
        stack.extend(class.class_traits().iter().cloned());
    }
    while let Some(trait_fqcn) = stack.pop() {
        if !visited.insert(trait_fqcn.clone()) {
            continue;
        }
        let here = Fqcn::interned(db, Name::new(trait_fqcn.as_ref()));
        if find_property_in_class(db, here, name).is_some() {
            return true;
        }
        if let Some(class) = find_class_like(db, here) {
            stack.extend(class.class_traits().iter().cloned());
        }
    }
    false
}

fn find_property_in_mixins<'db>(
    db: &'db dyn MirDatabase,
    fqcn: Fqcn<'db>,
    name: &str,
    visited: &mut std::collections::HashSet<Arc<str>>,
) -> Option<(Arc<str>, PropertyDef)> {
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
        let mixin_here = Fqcn::interned(db, Name::new(mixin_fqcn.as_ref()));
        // Walk the mixin's full inheritance chain.
        for ancestor in class_ancestors_by_fqcn(db, mixin_here).iter() {
            let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
            if let Some(p) = find_property_in_class(db, here, name) {
                return Some((ancestor.clone(), p));
            }
        }
        // Recurse into the mixin's own mixins.
        if let Some(result) = find_property_in_mixins(db, mixin_here, name, visited) {
            return Some(result);
        }
    }
    None
}

/// Existence-check for "is `name` concretely implemented (non-abstract,
/// non-interface) somewhere reachable from `fqcn`'s inheritance chain?".
/// Used to flag UnimplementedAbstractMethod and UnimplementedInterfaceMethod.
pub fn is_method_concretely_implemented(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> bool {
    let lower = crate::util::php_ident_lowercase(method_name);
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
        if let Some(m) = class.own_methods().get(lower.as_str()) {
            if !m.is_abstract {
                return true;
            }
        }
        // A method fulfilled only via `use Trait { orig as alias; }` never
        // materializes a literally-named entry in any ancestor's `own_methods()` —
        // resolve the alias to its original trait method and check that instead.
        if let ClassLike::Class(cls) = &class {
            if let Some((opt_trait_fqcn, orig_method, _vis_override, _alias_cased)) =
                cls.trait_aliases.get(lower.as_str())
            {
                let search_traits: &[Arc<str>] = match opt_trait_fqcn {
                    Some(tfqcn) => std::slice::from_ref(tfqcn),
                    None => &cls.traits,
                };
                if search_traits
                    .iter()
                    .any(|t| is_method_concretely_implemented(db, t.as_ref(), orig_method))
                {
                    return true;
                }
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
) -> Option<(Arc<str>, ConstantDef)> {
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::interned(db, Name::new(ancestor.as_ref()));
        if let Some(c) = find_class_constant_in_class(db, here, name) {
            return Some((ancestor.clone(), c));
        }
    }
    None
}
