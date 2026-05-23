use std::sync::Arc;

use indexmap::IndexMap;
use mir_types::{Symbol, Union};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Interned common types for deduplication
// ---------------------------------------------------------------------------

/// Interned Union types for common parameter/property types.
/// Deduplicates allocations when thousands of parameters share types like `string`, `int`, etc.
mod interned_types {
    use super::*;
    use std::sync::OnceLock;

    fn intern_string() -> Arc<Union> {
        Arc::new(Union::string())
    }

    fn intern_int() -> Arc<Union> {
        Arc::new(Union::int())
    }

    fn intern_float() -> Arc<Union> {
        Arc::new(Union::float())
    }

    fn intern_bool() -> Arc<Union> {
        Arc::new(Union::bool())
    }

    fn intern_mixed() -> Arc<Union> {
        Arc::new(Union::mixed())
    }

    fn intern_null() -> Arc<Union> {
        Arc::new(Union::null())
    }

    fn intern_void() -> Arc<Union> {
        Arc::new(Union::void())
    }

    static STRING: OnceLock<Arc<Union>> = OnceLock::new();
    static INT: OnceLock<Arc<Union>> = OnceLock::new();
    static FLOAT: OnceLock<Arc<Union>> = OnceLock::new();
    static BOOL: OnceLock<Arc<Union>> = OnceLock::new();
    static MIXED: OnceLock<Arc<Union>> = OnceLock::new();
    static NULL: OnceLock<Arc<Union>> = OnceLock::new();
    static VOID: OnceLock<Arc<Union>> = OnceLock::new();

    pub fn string() -> Arc<Union> {
        STRING.get_or_init(intern_string).clone()
    }

    pub fn int() -> Arc<Union> {
        INT.get_or_init(intern_int).clone()
    }

    pub fn float() -> Arc<Union> {
        FLOAT.get_or_init(intern_float).clone()
    }

    pub fn bool() -> Arc<Union> {
        BOOL.get_or_init(intern_bool).clone()
    }

    pub fn mixed() -> Arc<Union> {
        MIXED.get_or_init(intern_mixed).clone()
    }

    pub fn null() -> Arc<Union> {
        NULL.get_or_init(intern_null).clone()
    }

    pub fn void() -> Arc<Union> {
        VOID.get_or_init(intern_void).clone()
    }

    /// Global content-keyed `Arc<Union>` interner. Any structurally-identical
    /// Union is shared as a single Arc across the session.
    ///
    /// Why: PHP codebases re-declare a small set of type shapes thousands of
    /// times — `string|null` return types, `int` params, `array<string, mixed>`
    /// property types. Without interning, each declaration allocates its own
    /// `Arc<Union>` plus the inline `SmallVec<[Atomic; 2]>` and any boxed
    /// `Atomic` payloads. With interning, only the first occurrence allocates.
    ///
    /// Trade-off: every `intern_or_wrap` call hashes + does one DashMap lookup.
    /// Hashing a `Union` is cheap (SmallVec, small atomics) — measured cost is
    /// well below the alloc-savings benefit on real workloads.
    static GLOBAL_UNION_INTERN: std::sync::OnceLock<dashmap::DashMap<Union, Arc<Union>>> =
        std::sync::OnceLock::new();

    fn global_intern_table() -> &'static dashmap::DashMap<Union, Arc<Union>> {
        GLOBAL_UNION_INTERN.get_or_init(dashmap::DashMap::default)
    }

    /// Try to intern a Union if it matches a common type, otherwise wrap in Arc.
    pub fn intern_or_wrap(union: Union) -> Arc<Union> {
        // Fast path 1: single-atomic scalar — covered by `OnceLock` constants.
        // Avoids any DashMap traffic for the most common case.
        if union.types.len() == 1 && !union.possibly_undefined && !union.from_docblock {
            match &union.types[0] {
                mir_types::Atomic::TString => return string(),
                mir_types::Atomic::TInt => return int(),
                mir_types::Atomic::TFloat => return float(),
                mir_types::Atomic::TBool => return bool(),
                mir_types::Atomic::TMixed => return mixed(),
                mir_types::Atomic::TNull => return null(),
                mir_types::Atomic::TVoid => return void(),
                _ => {}
            }
        }
        // Fast path 2: empty Union — also a common case (e.g. unresolved
        // return type). Don't pollute the intern table with these.
        if union.types.is_empty() {
            return Arc::new(union);
        }
        // Global path: dedup against any previously-seen identical Union.
        let table = global_intern_table();
        if let Some(existing) = table.get(&union) {
            return Arc::clone(existing.value());
        }
        let arc = Arc::new(union.clone());
        // `insert` semantics: if a parallel thread beat us, its Arc wins.
        // The lookup-before-insert race is benign — both Arcs are content-
        // equal — but we still want to share the canonical one going forward.
        match table.entry(union) {
            dashmap::mapref::entry::Entry::Occupied(o) => Arc::clone(o.get()),
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(Arc::clone(&arc));
                arc
            }
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: Symbol,
    pub bound: Option<Union>,
    /// The entity (class or function FQN) that declared this template param.
    pub defining_entity: Symbol,
    pub variance: mir_types::Variance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnParam {
    pub name: Symbol,
    /// Parameter type. Stored as `Option<Arc<Union>>` to enable deduplication of
    /// common types across parameters. Many parameters share types like `string`,
    /// `int`, `bool`, etc., so interning via Arc saves allocations.
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub ty: Option<Arc<Union>>,
    /// Whether this parameter has a default value. During analysis, defaults are
    /// never used for their value — only for marking parameters as optional.
    pub has_default: bool,
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
}

impl std::hash::Hash for FnParam {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.has_default.hash(state);
        self.is_variadic.hash(state);
        self.is_byref.hash(state);
        self.is_optional.hash(state);
        // Hash the type value (not the Arc pointer) so that two FnParams with
        // equal types (PartialEq) always produce the same hash, even when they
        // are backed by different Arc allocations.
        self.ty.as_deref().hash(state);
    }
}

// Serde helpers to transparently convert between Option<Union> and Option<Arc<Union>>
fn deserialize_param_type<'de, D>(deserializer: D) -> Result<Option<Arc<Union>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Union>::deserialize(deserializer).map(|opt| opt.map(interned_types::intern_or_wrap))
}

fn serialize_param_type<S>(value: &Option<Arc<Union>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let opt = value.as_ref().map(|arc| (**arc).clone());
    opt.serialize(serializer)
}

fn deserialize_return_type<'de, D>(deserializer: D) -> Result<Option<Arc<Union>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Union>::deserialize(deserializer).map(|opt| opt.map(interned_types::intern_or_wrap))
}

fn serialize_return_type<S>(value: &Option<Arc<Union>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let opt = value.as_ref().map(|arc| (**arc).clone());
    opt.serialize(serializer)
}

fn deserialize_params<'de, D>(deserializer: D) -> Result<Arc<[FnParam]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<FnParam>::deserialize(deserializer).map(|v| Arc::from(v.into_boxed_slice()))
}

fn default_imports() -> Arc<FxHashMap<Symbol, Symbol>> {
    Arc::new(FxHashMap::default())
}

/// Deserialize imports map. Supports both new (Symbol-keyed) and legacy
/// (String-keyed) on-disk formats — older `cache.bin` files have plain
/// `HashMap<String, String>`. Either way, we intern at load time so the
/// in-memory representation is always `Arc<FxHashMap<Symbol, Symbol>>`.
fn deserialize_imports<'de, D>(deserializer: D) -> Result<Arc<FxHashMap<Symbol, Symbol>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = FxHashMap::<String, String>::deserialize(deserializer)?;
    let mut out: FxHashMap<Symbol, Symbol> =
        FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());
    for (k, v) in raw {
        out.insert(Symbol::new(&k), Symbol::new(&v));
    }
    Ok(Arc::new(out))
}

/// Serialize imports as the legacy `HashMap<String, String>` shape so disk
/// caches written by this version remain compatible with readers that haven't
/// been recompiled yet (and vice-versa).
fn serialize_imports<S>(
    value: &Arc<FxHashMap<Symbol, Symbol>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(value.len()))?;
    for (k, v) in value.iter() {
        map.serialize_entry(k.as_str(), v.as_str())?;
    }
    map.end()
}

fn serialize_params<S>(value: &Arc<[FnParam]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_ref().serialize(serializer)
}

/// Helper to wrap Option<Union> in interned Arc<Union>.
pub fn wrap_param_type(ty: Option<Union>) -> Option<Arc<Union>> {
    ty.map(interned_types::intern_or_wrap)
}

/// Helper to wrap return type Option<Union> in interned Arc<Union>.
pub fn wrap_return_type(ty: Option<Union>) -> Option<Arc<Union>> {
    ty.map(interned_types::intern_or_wrap)
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
    #[serde(
        deserialize_with = "deserialize_params",
        serialize_with = "serialize_params"
    )]
    pub params: Arc<[FnParam]>,
    /// Type from annotation (`@return` / native type hint). `None` means unannotated.
    /// Stored as `Option<Arc<Union>>` to enable deduplication of common return types
    /// (e.g., `void`, `string`, `mixed`, `bool`) across thousands of methods.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub return_type: Option<Arc<Union>>,
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
    /// Plain-text description from the docblock (text before `@tag` lines).
    /// Used for hover info.
    #[serde(default)]
    pub docstring: Option<Arc<str>>,
    /// True for methods added via `@method` docblock annotations. Virtual
    /// methods must not be required as concrete interface implementations.
    #[serde(default)]
    pub is_virtual: bool,
}

impl MethodStorage {
    pub fn effective_return_type(&self) -> Option<&Union> {
        self.return_type
            .as_deref()
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
    pub deprecated: Option<Arc<str>>,
    pub is_internal: bool,
    pub location: Option<Location>,
    /// Per-`use` statement locations for each used trait: `(fqcn, location)` in
    /// declaration order, parallel to `traits`.  Absent from older serialized
    /// slices; defaults to empty.
    #[serde(default)]
    pub trait_use_locations: Vec<(Arc<str>, Location)>,
    /// Type aliases declared on this class via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Union>,
    /// Raw import-type declarations (`(local_name, original_name, from_class)`) — resolved during finalization.
    #[serde(default)]
    pub pending_import_types: Vec<(Arc<str>, Arc<str>, Arc<str>)>,
}

impl ClassStorage {
    pub fn get_method(&self, name: &str) -> Option<&MethodStorage> {
        // PHP method names are case-insensitive; caller should pass lowercase name.
        // Only searches own_methods — inherited method resolution is done by
        // `db::lookup_method_in_chain`.
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
    #[serde(
        deserialize_with = "deserialize_params",
        serialize_with = "serialize_params"
    )]
    pub params: Arc<[FnParam]>,
    /// Type from annotation (`@return` / native type hint). `None` means unannotated.
    /// Stored as `Option<Arc<Union>>` to enable deduplication of common return types.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub return_type: Option<Arc<Union>>,
    pub inferred_return_type: Option<Union>,
    pub template_params: Vec<TemplateParam>,
    pub assertions: Vec<Assertion>,
    pub throws: Vec<Arc<str>>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
    pub location: Option<Location>,
    /// Plain-text description from the docblock (text before `@tag` lines).
    /// Used for hover info.
    #[serde(default)]
    pub docstring: Option<Arc<str>>,
}

impl FunctionStorage {
    pub fn effective_return_type(&self) -> Option<&Union> {
        self.return_type
            .as_deref()
            .or(self.inferred_return_type.as_ref())
    }
}

// ---------------------------------------------------------------------------
// StubSlice — serializable bundle of definitions from one extension's stubs
// ---------------------------------------------------------------------------

/// A snapshot of all PHP definitions contributed by a single stub file set.
///
/// Produced by `mir-stubs-gen` at code-generation time and deserialized at
/// runtime to ingest definitions into the salsa db via
/// `MirDatabase::ingest_stub_slice`.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StubSlice {
    pub classes: Vec<Arc<ClassStorage>>,
    pub interfaces: Vec<Arc<InterfaceStorage>>,
    pub traits: Vec<Arc<TraitStorage>>,
    pub enums: Vec<Arc<EnumStorage>>,
    pub functions: Vec<Arc<FunctionStorage>>,
    #[serde(default)]
    pub constants: Vec<(Arc<str>, Union)>,
    /// Source file this slice was collected from. `None` for bundled stub slices
    /// that were pre-computed and are not tied to a specific on-disk file.
    #[serde(default)]
    pub file: Option<Arc<str>>,
    /// Types of `@var`-annotated global variables collected from this file.
    /// Populated by `DefinitionCollector`; ingested into the salsa db's
    /// `global_vars` table by `ingest_stub_slice` when `file` is `Some`.
    #[serde(default)]
    pub global_vars: Vec<(Arc<str>, Union)>,
    /// The first namespace declared in this file (e.g. `"App\\Service"`).
    /// Populated by `DefinitionCollector`; ingested into the salsa db's
    /// `file_namespaces` table by `ingest_stub_slice` when `file` is `Some`.
    #[serde(default)]
    pub namespace: Option<Arc<str>>,
    /// `use` alias map for this file: alias → FQCN.
    ///
    /// Stored as `Arc<FxHashMap<Symbol, Symbol>>` so that `file_imports()`
    /// returns a cheap Arc clone instead of deep-cloning the map on every
    /// `resolve_name_via_db` call (which fires once per symbol reference in
    /// Pass 2). `Symbol` keys/values shrink each entry from ~108 bytes
    /// (two `String` headers + two heap allocs averaging ~30 chars) to
    /// 16 bytes (two `Ustr` u64 handles); the global ustr interner holds
    /// one copy of each unique alias / FQCN string for the whole session.
    #[serde(
        deserialize_with = "deserialize_imports",
        serialize_with = "serialize_imports"
    )]
    #[serde(default = "default_imports")]
    pub imports: Arc<FxHashMap<Symbol, Symbol>>,
    /// Set to `true` after `deduplicate_params_in_slice` has run on this slice.
    /// `ingest_stub_slice` skips the clone+re-dedup when this flag is set.
    #[serde(skip)]
    pub is_deduped: bool,
}

// ---------------------------------------------------------------------------
// Param list deduplication
// ---------------------------------------------------------------------------

use std::sync::Mutex;

type ParamCache = Mutex<FxHashMap<Vec<FnParam>, Arc<[FnParam]>>>;

/// Global cache of canonical Arc<[FnParam]> instances for deduplication.
/// Shared across all StubSlices to deduplicate vendor code with millions of
/// methods that often have identical parameter lists.
static PARAM_DEDUP_CACHE: std::sync::OnceLock<ParamCache> = std::sync::OnceLock::new();

/// Deduplicate parameter lists across all methods and functions in a StubSlice.
/// Many PHP framework methods share identical parameter lists (e.g., thousands
/// of `(string $arg, array $opts)` signatures). This function groups identical
/// param lists globally (across all slices processed so far) and replaces them
/// with Arc<[FnParam]> pointers to shared allocations.
///
/// Expected memory savings: 100–150 MiB on cold start (vendor collection).
pub fn deduplicate_params_in_slice(slice: &mut StubSlice) {
    let cache: &ParamCache = PARAM_DEDUP_CACHE.get_or_init(|| Mutex::new(FxHashMap::default()));
    let mut canonical_params = cache.lock().unwrap();

    let mut deduplicate = |params: &mut Arc<[FnParam]>| {
        if let Some(existing) = canonical_params.get(params.as_ref()) {
            *params = existing.clone();
        } else {
            canonical_params.insert(params.as_ref().to_vec(), params.clone());
        }
    };

    // Deduplicate method params in all classes
    for cls in &mut slice.classes {
        for method in Arc::make_mut(cls).own_methods.values_mut() {
            deduplicate(&mut Arc::make_mut(method).params);
        }
    }

    // Deduplicate method params in all interfaces
    for iface in &mut slice.interfaces {
        for method in Arc::make_mut(iface).own_methods.values_mut() {
            deduplicate(&mut Arc::make_mut(method).params);
        }
    }

    // Deduplicate method params in all traits
    for tr in &mut slice.traits {
        for method in Arc::make_mut(tr).own_methods.values_mut() {
            deduplicate(&mut Arc::make_mut(method).params);
        }
    }

    // Deduplicate method params in all enums
    for en in &mut slice.enums {
        for method in Arc::make_mut(en).own_methods.values_mut() {
            deduplicate(&mut Arc::make_mut(method).params);
        }
    }

    // Deduplicate function params
    for func in &mut slice.functions {
        deduplicate(&mut Arc::make_mut(func).params);
    }
    slice.is_deduped = true;
}
