use std::sync::Arc;

use mir_codebase::storage::{Assertion, FnParam, Location, TemplateParam, Visibility};
use mir_codebase::StubSlice;
use mir_issues::Issue;
use mir_types::Union;

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

// SAFETY: FileDefinitions contains Arc pointers and Vec, which are Move-safe.
// The pointer passed to maybe_update is provided by Salsa and points to
// properly aligned and initialized memory. We have exclusive write access
// through the mutable pointer (Salsa guarantees this). The in-place update
// is safe because we own both the old and new values.
//
// Optimization: Use PartialEq to skip downstream recomputation when definitions
// haven't changed (e.g., no-op file saves in LSP). This is especially valuable
// in incremental scenarios where many files are unchanged.
unsafe impl salsa::Update for FileDefinitions {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false; // Content unchanged; Salsa skips dependent queries
        }
        *old = new_val;
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
    /// Per-`use` locations for each used trait, parallel to `traits`.
    /// Empty for non-class nodes and for slices loaded from older caches.
    pub trait_use_locations: Arc<[(Arc<str>, Location)]>,
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
    pub docstring: Option<Arc<str>>,
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
    pub docstring: Option<Arc<str>>,
    pub is_internal: bool,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_constructor: bool,
    pub is_pure: bool,
    /// True for methods added via `@method` docblock annotations.
    pub is_virtual: bool,
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

// SAFETY: Ancestors contains Arc pointers, which are Move-safe.
// The pointer passed to maybe_update is provided by Salsa and points to
// properly aligned and initialized memory. We dereference it to check equality
// and conditionally update. Salsa guarantees exclusive write access through
// the mutable pointer. The comparison is safe because we're comparing valid
// initialized values.
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
