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

use mir_codebase::definitions::{
    ClassDef, ConstantDef, DeclaredParam, EnumDef, FunctionDef, InterfaceDef, MethodDef,
    PropertyDef, TraitDef,
};
use mir_types::{Atomic, Name, Type};
use rustc_hash::FxHashMap;

use crate::db::{collect_file_definitions, source_file_for_fqcn, Fqcn, MirDatabase, SourceFile};

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
) -> Option<Arc<InterfaceDef>> {
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
) -> Option<Arc<TraitDef>> {
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
) -> Option<Arc<EnumDef>> {
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
) -> Option<Arc<FunctionDef>> {
    let defs = collect_file_definitions(db, file);
    let target = fqn.name(db);
    defs.slice
        .functions
        .iter()
        .find(|f| f.fqn.eq_ignore_ascii_case(target.as_ref()))
        .cloned()
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

/// Salsa-tracked per-(file, idx) class storage. One memo entry per distinct
/// class ever queried; subsequent calls return the same Arc cheaply.
#[salsa::tracked]
pub fn class_def_at(db: &dyn MirDatabase, file: SourceFile, idx: u32) -> Option<Arc<ClassDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.classes.get(idx as usize).cloned()
}

/// Plain classes (not interfaces/traits/enums) defined in `analyzed_files`,
/// each materialized exactly once and the whole list sorted by FQCN for
/// deterministic issue order across runs.
///
/// Decomposes per file via the memoized [`collect_file_definitions`] query
/// rather than walking the global symbol index: in batch mode `analyzed_files`
/// is the project file set, so vendor / stub classes are never enumerated at
/// all (they aren't in the set). An empty `analyzed_files` means "all files"
/// — used by the `new()`/unit-test path — and falls back to every registered
/// source file. Incremental edits only recompute the touched files' slices.
pub fn analyzed_class_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, ClassLike)> {
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

    let mut out: Vec<(Arc<str>, ClassLike)> = Vec::new();
    for sf in files {
        let defs = collect_file_definitions(db, sf);
        for class in defs.slice.classes.iter() {
            out.push((class.fqcn.clone(), ClassLike::Class(class.clone())));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Like [`analyzed_class_defs`] but returns interface definitions from
/// analyzed files. Used to check `#[Override]` on interface methods.
pub fn analyzed_interface_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::InterfaceDef>)> {
    let mut files: Vec<SourceFile> = if analyzed_files.is_empty() {
        db.all_source_files()
    } else {
        analyzed_files
            .iter()
            .filter_map(|p| db.lookup_source_file(p))
            .collect()
    };
    files.sort_by_key(|a| a.path(db));
    let mut out = Vec::new();
    for sf in files {
        let defs = collect_file_definitions(db, sf);
        for iface in defs.slice.interfaces.iter() {
            out.push((iface.fqcn.clone(), iface.clone()));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Like [`analyzed_interface_defs`] but returns enum definitions from analyzed files.
pub fn analyzed_enum_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::EnumDef>)> {
    let mut files: Vec<SourceFile> = if analyzed_files.is_empty() {
        db.all_source_files()
    } else {
        analyzed_files
            .iter()
            .filter_map(|p| db.lookup_source_file(p))
            .collect()
    };
    files.sort_by_key(|a| a.path(db));
    let mut out = Vec::new();
    for sf in files {
        let defs = collect_file_definitions(db, sf);
        for e in defs.slice.enums.iter() {
            out.push((e.fqcn.clone(), e.clone()));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn analyzed_trait_defs(
    db: &dyn MirDatabase,
    analyzed_files: &rustc_hash::FxHashSet<Arc<str>>,
) -> Vec<(Arc<str>, Arc<mir_codebase::definitions::TraitDef>)> {
    let mut files: Vec<SourceFile> = if analyzed_files.is_empty() {
        db.all_source_files()
    } else {
        analyzed_files
            .iter()
            .filter_map(|p| db.lookup_source_file(p))
            .collect()
    };
    files.sort_by_key(|a| a.path(db));
    let mut out = Vec::new();
    for sf in files {
        let defs = collect_file_definitions(db, sf);
        for t in defs.slice.traits.iter() {
            out.push((t.fqcn.clone(), t.clone()));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[salsa::tracked]
pub fn interface_def_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<InterfaceDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.interfaces.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn trait_def_at(db: &dyn MirDatabase, file: SourceFile, idx: u32) -> Option<Arc<TraitDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.traits.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn enum_def_at(db: &dyn MirDatabase, file: SourceFile, idx: u32) -> Option<Arc<EnumDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.enums.get(idx as usize).cloned()
}

#[salsa::tracked]
pub fn function_def_at(
    db: &dyn MirDatabase,
    file: SourceFile,
    idx: u32,
) -> Option<Arc<FunctionDef>> {
    let defs = collect_file_definitions(db, file);
    defs.slice.functions.get(idx as usize).cloned()
}

/// Composite: resolve `fqcn` to its defining file, then locate any
/// class-like definition (class / interface / trait / enum) within it.
///
/// **The headline pull-based lookup.** Demands `collect_file_definitions`
/// on the resolved file as a salsa tracked dependency — callers reading
/// this from a tracked context are correctly invalidated when either the
/// resolver or the defining file's text changes. No prior `ingest_file`
/// call is required: the file's text must be registered (via
/// `set_file_text` or `set_workspace_files`), but definition collection
/// happens on demand inside salsa.
pub fn find_class_like<'db>(db: &'db dyn MirDatabase, fqcn: Fqcn<'db>) -> Option<ClassLike> {
    use crate::db::SymbolLoc;
    // O(1) HashMap lookup in the workspace symbol index, then a per-(file, idx)
    // salsa-memoized fetch of the Arc<Storage>.
    //
    // `Name::ascii_lowercase` is memoized — first call per unique FQCN
    // allocates the lowercase string and interns it; subsequent calls hit a
    // process-global DashMap. The hot body-analysis path becomes alloc-free after
    // warmup.
    let key = fqcn.name(db).ascii_lowercase();
    // Prefer the frozen, borrow-only index (set on the batch body/class pass)
    // to avoid cloning the singleton's three Arcs on every call; fall back to
    // the live index on the canonical/open-file db. `.copied()` ends the borrow
    // before the `*_def_at` salsa calls below.
    let loc = match db.frozen_workspace_index() {
        Some(frozen) => frozen.class_like.get(&key).copied(),
        None => crate::db::workspace_index(db).class_like.get(&key).copied(),
    }?;
    match loc {
        SymbolLoc::Class { file, idx } => class_def_at(db, file, idx as u32)
            .clone()
            .map(ClassLike::Class),
        SymbolLoc::Interface { file, idx } => interface_def_at(db, file, idx as u32)
            .clone()
            .map(ClassLike::Interface),
        SymbolLoc::Trait { file, idx } => trait_def_at(db, file, idx as u32)
            .clone()
            .map(ClassLike::Trait),
        SymbolLoc::Enum { file, idx } => enum_def_at(db, file, idx as u32)
            .clone()
            .map(ClassLike::Enum),
        SymbolLoc::Function { .. } | SymbolLoc::Constant { .. } => None,
    }
}

/// The file a class-like symbol is declared in, if known.
pub fn class_like_decl_file(db: &dyn MirDatabase, fqcn: Fqcn<'_>) -> Option<Arc<str>> {
    let key = fqcn.name(db).ascii_lowercase();
    let loc = match db.frozen_workspace_index() {
        Some(frozen) => frozen.class_like.get(&key).copied(),
        None => crate::db::workspace_index(db).class_like.get(&key).copied(),
    }?;
    Some(loc.file().path(db).clone())
}

/// Composite: resolve `fqn` to its defining file, then locate the
/// function within it.
pub fn find_function<'db>(db: &'db dyn MirDatabase, fqn: Fqcn<'db>) -> Option<Arc<FunctionDef>> {
    use crate::db::SymbolLoc;
    let key = fqn.name(db).ascii_lowercase();
    let loc = match db.frozen_workspace_index() {
        Some(frozen) => frozen.functions.get(&key).copied(),
        None => crate::db::workspace_index(db).functions.get(&key).copied(),
    };
    let SymbolLoc::Function { file, idx } = loc? else {
        return None;
    };
    function_def_at(db, file, idx as u32).clone()
}

/// Composite: resolve `fqn` to its defining file, then locate a global
/// constant within it.
pub fn find_global_constant<'db>(
    db: &'db dyn MirDatabase,
    fqn: Fqcn<'db>,
) -> Option<Arc<mir_types::Type>> {
    use crate::db::SymbolLoc;
    // Constants are keyed case-sensitively (raw name), unlike class_like/functions.
    let key = fqn.name(db);
    let const_loc = match db.frozen_workspace_index() {
        Some(frozen) => frozen.constants.get(key).copied(),
        None => crate::db::workspace_index(db).constants.get(key).copied(),
    };
    if let Some(SymbolLoc::Constant { file, idx }) = const_loc {
        let defs = collect_file_definitions(db, file);
        if let Some((_, ty)) = defs.slice.constants.get(idx) {
            return Some(Arc::new(ty.clone()));
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
    let class = find_class_like(db, fqcn)?;
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
    if let ClassLike::Enum(e) = &class {
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
                is_pure: false,
                no_named_arguments: false,
                is_override: false,
                deprecated: None,
                location: None,
                docstring: None,
                taint_sink_params: vec![],
                is_taint_source: false,
                if_this_is: None,
                self_out: None,
                is_inherit_doc: false,
                is_mutation_free: false,
                is_external_mutation_free: false,
                data_provider_targets: vec![],
            }));
        }
        if is_backed && (lower == "from" || lower == "tryfrom") {
            let value_param = DeclaredParam {
                name: Name::from("value"),
                ty: e.scalar_type.as_ref().map(|t| Arc::new(t.clone())),
                out_ty: None,
                has_default: false,
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
                is_pure: false,
                no_named_arguments: false,
                is_override: false,
                deprecated: None,
                location: None,
                docstring: None,
                taint_sink_params: vec![],
                is_taint_source: false,
                if_this_is: None,
                self_out: None,
                is_inherit_doc: false,
                is_mutation_free: false,
                is_external_mutation_free: false,
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
    let class = find_class_like(db, fqcn)?;
    class.own_properties()?.get(name).cloned()
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
        let here = Fqcn::new(db, Name::new(name.as_ref()));
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
            ExprKind::String(s) => Some(s.to_string()),
            ExprKind::Int(n) => Some(n.to_string()),
            ExprKind::ClassConstAccess(cca) => match (&cca.class.kind, &cca.member.kind) {
                (ExprKind::Identifier(cls), ExprKind::Identifier(m)) if m.as_ref() == "class" => {
                    Some(crate::db::resolve_name(db, path.as_ref(), cls))
                }
                _ => None,
            },
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
        let Some(name) = decl.name.as_ref().and_then(|n| n.as_deref()) else {
            return;
        };
        if !crate::db::resolve_name(db, path.as_ref(), name).eq_ignore_ascii_case(target.as_str()) {
            return;
        }
        for member in decl.body.members.iter() {
            let ClassMemberKind::Property(p) = &member.kind else {
                continue;
            };
            let (Some(prop_name), Some(default)) = (p.name.as_deref(), p.default.as_ref()) else {
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
                let Some(value) = expr_to_string(&el.value) else {
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
    let here = Fqcn::new(db, Name::new(fqcn));
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
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::new(db, Name::new(ancestor.as_ref()));
        if let Some(m) = find_method_in_class(db, here, name) {
            return Some((ancestor.clone(), m));
        }
    }
    // Separate @mixin walk — cycle-safe, depth-first.
    let mut visited_mixins = std::collections::HashSet::<Arc<str>>::new();
    find_method_in_mixins(db, fqcn, name, &mut visited_mixins)
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
        let anc_fqcn = Fqcn::new(db, Name::new(anc));
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
    let class = find_class_like(db, fqcn)?;

    // Check this class/trait's own methods first.
    if let Some(m) = find_method_in_class(db, fqcn, method_lower) {
        return Some((class_name, m));
    }

    // For a plain class: respect its insteadof exclusions when walking its traits.
    if let ClassLike::Class(cls) = &class {
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
                let here = Fqcn::new(db, Name::new(trait_fqcn.as_ref()));
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
                    return Some((class_name, Arc::new(m_clone)));
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
            let here = Fqcn::new(db, Name::new(trait_fqcn.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        if let Some(ref parent) = cls.parent {
            let here = Fqcn::new(db, Name::new(parent.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        for iface_fqcn in cls.interfaces.iter() {
            let here = Fqcn::new(db, Name::new(iface_fqcn.as_ref()));
            if let Some(result) = walk_method_with_precedence(db, here, method_lower, visited) {
                return Some(result);
            }
        }
        return None;
    }

    // For traits and interfaces: plain ancestor walk (no per-class insteadof).
    for ancestor_fqcn in class.ancestor_fqcns() {
        let here = Fqcn::new(db, Name::new(ancestor_fqcn.as_ref()));
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
        let mixin_here = Fqcn::new(db, Name::new(mixin_fqcn.as_ref()));
        // Walk the mixin's full inheritance chain.
        for ancestor in class_ancestors_by_fqcn(db, mixin_here).iter() {
            let here = Fqcn::new(db, Name::new(ancestor.as_ref()));
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
    for ancestor in class_ancestors_by_fqcn(db, fqcn).iter() {
        let here = Fqcn::new(db, Name::new(ancestor.as_ref()));
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
        let here = Fqcn::new(db, Name::new(trait_fqcn.as_ref()));
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
        let mixin_here = Fqcn::new(db, Name::new(mixin_fqcn.as_ref()));
        // Walk the mixin's full inheritance chain.
        for ancestor in class_ancestors_by_fqcn(db, mixin_here).iter() {
            let here = Fqcn::new(db, Name::new(ancestor.as_ref()));
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
        let here = Fqcn::new(db, Name::new(ancestor.as_ref()));
        if let Some(c) = find_class_constant_in_class(db, here, name) {
            return Some((ancestor.clone(), c));
        }
    }
    None
}
