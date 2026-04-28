use std::sync::Arc;

use indexmap::IndexMap;
use mir_types::Union;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn is_at_least(&self, required: Visibility) -> bool {
        *self <= required
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Protected => write!(f, "protected"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: Arc<str>,
    pub bound: Option<Union>,
    /// The entity (class or function FQN) that declared this template param.
    pub defining_entity: Arc<str>,
    pub variance: mir_types::Variance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FnParam {
    pub name: Arc<str>,
    pub ty: Option<Union>,
    pub default: Option<Union>,
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
}

// ---------------------------------------------------------------------------
// Location — file + pre-computed line/col span
// ---------------------------------------------------------------------------

/// Declaration location.
///
/// Columns are 0-based Unicode scalar value (code-point) counts, equivalent to
/// LSP `utf-32` position encoding. Convert to UTF-16 code units at the LSP
/// boundary for clients that do not advertise `utf-32` support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub file: Arc<str>,
    /// 1-based start line.
    pub line: u32,
    /// 1-based end line (inclusive). Equal to `line` for single-line spans.
    pub line_end: u32,
    /// 0-based Unicode code-point column of the span start.
    pub col_start: u16,
    /// 0-based Unicode code-point column of the span end (exclusive).
    pub col_end: u16,
}

impl Location {
    pub fn new(file: Arc<str>, line: u32, line_end: u32, col_start: u16, col_end: u16) -> Self {
        Self {
            file,
            line,
            line_end,
            col_start,
            col_end,
        }
    }
}

// ---------------------------------------------------------------------------
// Assertion — `@psalm-assert`, `@psalm-assert-if-true`, etc.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssertionKind {
    Assert,
    AssertIfTrue,
    AssertIfFalse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    pub kind: AssertionKind,
    pub param: Arc<str>,
    pub ty: Union,
}

// ---------------------------------------------------------------------------
// MethodStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodStorage {
    pub name: Arc<str>,
    pub fqcn: Arc<str>,
    pub params: Vec<FnParam>,
    /// Type from annotation (`@return` / native type hint). `None` means unannotated.
    pub return_type: Option<Union>,
    /// Type inferred from body analysis (filled in during pass 2).
    pub inferred_return_type: Option<Union>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_constructor: bool,
    pub template_params: Vec<TemplateParam>,
    pub assertions: Vec<Assertion>,
    pub throws: Vec<Arc<str>>,
    pub deprecated: Option<Arc<str>>,
    pub is_internal: bool,
    pub is_pure: bool,
    pub location: Option<Location>,
}

impl MethodStorage {
    pub fn effective_return_type(&self) -> Option<&Union> {
        self.return_type
            .as_ref()
            .or(self.inferred_return_type.as_ref())
    }
}

// ---------------------------------------------------------------------------
// PropertyStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyStorage {
    pub name: Arc<str>,
    pub ty: Option<Union>,
    pub inferred_ty: Option<Union>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_readonly: bool,
    pub default: Option<Union>,
    pub location: Option<Location>,
}

// ---------------------------------------------------------------------------
// ConstantStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantStorage {
    pub name: Arc<str>,
    pub ty: Union,
    pub visibility: Option<Visibility>,
    #[serde(default)]
    pub is_final: bool,
    pub location: Option<Location>,
}

// ---------------------------------------------------------------------------
// ClassStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassStorage {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub parent: Option<Arc<str>>,
    pub interfaces: Vec<Arc<str>>,
    pub traits: Vec<Arc<str>>,
    pub own_methods: IndexMap<Arc<str>, Arc<MethodStorage>>,
    pub own_properties: IndexMap<Arc<str>, PropertyStorage>,
    pub own_constants: IndexMap<Arc<str>, ConstantStorage>,
    #[serde(default)]
    pub mixins: Vec<Arc<str>>,
    pub template_params: Vec<TemplateParam>,
    /// Type arguments from `@extends ParentClass<T1, T2>` — maps parent's template params to concrete types.
    pub extends_type_args: Vec<Union>,
    /// Type arguments from `@implements Interface<T1, T2>`.
    #[serde(default)]
    pub implements_type_args: Vec<(Arc<str>, Vec<Union>)>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_readonly: bool,
    /// Populated during finalization: all ancestor FQCNs (parents + interfaces, transitively).
    pub all_parents: Vec<Arc<str>>,
    pub deprecated: Option<Arc<str>>,
    pub is_internal: bool,
    pub location: Option<Location>,
    /// Type aliases declared on this class via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: std::collections::HashMap<Arc<str>, Union>,
    /// Raw import-type declarations (`(local_name, original_name, from_class)`) — resolved during finalization.
    #[serde(default)]
    pub pending_import_types: Vec<(Arc<str>, Arc<str>, Arc<str>)>,
}

impl ClassStorage {
    pub fn get_method(&self, name: &str) -> Option<&MethodStorage> {
        // PHP method names are case-insensitive; caller should pass lowercase name.
        // Only searches own_methods — inherited method resolution is done by Codebase::get_method.
        self.own_methods.get(name).map(Arc::as_ref).or_else(|| {
            self.own_methods
                .iter()
                .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_ref())
        })
    }

    pub fn get_property(&self, name: &str) -> Option<&PropertyStorage> {
        self.own_properties.get(name)
    }

    pub fn implements_or_extends(&self, fqcn: &str) -> bool {
        self.fqcn.as_ref() == fqcn || self.all_parents.iter().any(|p| p.as_ref() == fqcn)
    }
}

// ---------------------------------------------------------------------------
// InterfaceStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceStorage {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub extends: Vec<Arc<str>>,
    pub own_methods: IndexMap<Arc<str>, Arc<MethodStorage>>,
    pub own_constants: IndexMap<Arc<str>, ConstantStorage>,
    pub template_params: Vec<TemplateParam>,
    pub all_parents: Vec<Arc<str>>,
    pub location: Option<Location>,
}

// ---------------------------------------------------------------------------
// TraitStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitStorage {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub own_methods: IndexMap<Arc<str>, Arc<MethodStorage>>,
    pub own_properties: IndexMap<Arc<str>, PropertyStorage>,
    pub own_constants: IndexMap<Arc<str>, ConstantStorage>,
    pub template_params: Vec<TemplateParam>,
    /// Traits used by this trait (`use OtherTrait;` inside a trait body).
    pub traits: Vec<Arc<str>>,
    pub location: Option<Location>,
    /// `@psalm-require-extends` / `@phpstan-require-extends` — FQCNs that using classes must extend.
    #[serde(default)]
    pub require_extends: Vec<Arc<str>>,
    /// `@psalm-require-implements` / `@phpstan-require-implements` — FQCNs that using classes must implement.
    #[serde(default)]
    pub require_implements: Vec<Arc<str>>,
}

// ---------------------------------------------------------------------------
// EnumStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumCaseStorage {
    pub name: Arc<str>,
    pub value: Option<Union>,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumStorage {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub scalar_type: Option<Union>,
    pub interfaces: Vec<Arc<str>>,
    pub cases: IndexMap<Arc<str>, EnumCaseStorage>,
    pub own_methods: IndexMap<Arc<str>, Arc<MethodStorage>>,
    pub own_constants: IndexMap<Arc<str>, ConstantStorage>,
    pub location: Option<Location>,
}

// ---------------------------------------------------------------------------
// FunctionStorage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionStorage {
    pub fqn: Arc<str>,
    pub short_name: Arc<str>,
    pub params: Vec<FnParam>,
    pub return_type: Option<Union>,
    pub inferred_return_type: Option<Union>,
    pub template_params: Vec<TemplateParam>,
    pub assertions: Vec<Assertion>,
    pub throws: Vec<Arc<str>>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
    pub location: Option<Location>,
}

impl FunctionStorage {
    pub fn effective_return_type(&self) -> Option<&Union> {
        self.return_type
            .as_ref()
            .or(self.inferred_return_type.as_ref())
    }
}

// ---------------------------------------------------------------------------
// StubSlice — serializable bundle of definitions from one extension's stubs
// ---------------------------------------------------------------------------

/// A snapshot of all PHP definitions contributed by a single stub file set.
///
/// Produced by `mir-stubs-gen` at code-generation time and deserialized at
/// runtime to inject definitions into the `Codebase`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StubSlice {
    pub classes: Vec<ClassStorage>,
    pub interfaces: Vec<InterfaceStorage>,
    pub traits: Vec<TraitStorage>,
    pub enums: Vec<EnumStorage>,
    pub functions: Vec<FunctionStorage>,
    #[serde(default)]
    pub constants: Vec<(Arc<str>, Union)>,
    /// Source file this slice was collected from. `None` for bundled stub slices
    /// that were pre-computed and are not tied to a specific on-disk file.
    #[serde(default)]
    pub file: Option<Arc<str>>,
    /// Types of `@var`-annotated global variables collected from this file.
    /// Populated by `DefinitionCollector`; merged into `Codebase::global_vars`
    /// by [`crate::Codebase::inject_stub_slice`] when `file` is `Some`.
    #[serde(default)]
    pub global_vars: Vec<(Arc<str>, Union)>,
}
