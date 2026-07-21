use std::sync::Arc;

/// Insertion-ordered member map keyed by lowercased member name.
/// FxHash instead of SipHash: member lookup is one of the hottest analyzer
/// operations and the keys are short trusted identifiers.
pub type MemberMap<V> = indexmap::IndexMap<std::sync::Arc<str>, V, rustc_hash::FxBuildHasher>;
use mir_types::{Location, Name, Type};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Interned common types for deduplication
// ---------------------------------------------------------------------------

/// Interned Type types for common parameter/property types.
/// Deduplicates allocations when thousands of parameters share types like `string`, `int`, etc.
mod interned_types {
    use super::*;
    use std::sync::OnceLock;

    fn intern_string() -> Arc<Type> {
        Arc::new(Type::string())
    }

    fn intern_int() -> Arc<Type> {
        Arc::new(Type::int())
    }

    fn intern_float() -> Arc<Type> {
        Arc::new(Type::float())
    }

    fn intern_bool() -> Arc<Type> {
        Arc::new(Type::bool())
    }

    fn intern_mixed() -> Arc<Type> {
        Arc::new(Type::mixed())
    }

    fn intern_null() -> Arc<Type> {
        Arc::new(Type::null())
    }

    fn intern_void() -> Arc<Type> {
        Arc::new(Type::void())
    }

    static STRING: OnceLock<Arc<Type>> = OnceLock::new();
    static INT: OnceLock<Arc<Type>> = OnceLock::new();
    static FLOAT: OnceLock<Arc<Type>> = OnceLock::new();
    static BOOL: OnceLock<Arc<Type>> = OnceLock::new();
    static MIXED: OnceLock<Arc<Type>> = OnceLock::new();
    static NULL: OnceLock<Arc<Type>> = OnceLock::new();
    static VOID: OnceLock<Arc<Type>> = OnceLock::new();

    pub fn string() -> Arc<Type> {
        STRING.get_or_init(intern_string).clone()
    }

    pub fn int() -> Arc<Type> {
        INT.get_or_init(intern_int).clone()
    }

    pub fn float() -> Arc<Type> {
        FLOAT.get_or_init(intern_float).clone()
    }

    pub fn bool() -> Arc<Type> {
        BOOL.get_or_init(intern_bool).clone()
    }

    pub fn mixed() -> Arc<Type> {
        MIXED.get_or_init(intern_mixed).clone()
    }

    pub fn null() -> Arc<Type> {
        NULL.get_or_init(intern_null).clone()
    }

    pub fn void() -> Arc<Type> {
        VOID.get_or_init(intern_void).clone()
    }

    /// Global content-keyed `Arc<Type>` interner. Any structurally-identical
    /// Type is shared as a single Arc across the session.
    ///
    /// Why: PHP codebases re-declare a small set of type shapes thousands of
    /// times — `string|null` return types, `int` params, `array<string, mixed>`
    /// property types. Without interning, each declaration allocates its own
    /// `Arc<Type>` plus the inline `SmallVec<[Atomic; 2]>` and any boxed
    /// `Atomic` payloads. With interning, only the first occurrence allocates.
    ///
    /// Trade-off: every `intern_or_wrap` call hashes + does one DashMap lookup.
    /// Hashing a `Type` is cheap (SmallVec, small atomics) — measured cost is
    /// well below the alloc-savings benefit on real workloads.
    type InternTable = dashmap::DashMap<Type, Arc<Type>, rustc_hash::FxBuildHasher>;

    static GLOBAL_UNION_INTERN: std::sync::OnceLock<InternTable> = std::sync::OnceLock::new();

    fn global_intern_table() -> &'static InternTable {
        GLOBAL_UNION_INTERN.get_or_init(|| dashmap::DashMap::with_hasher(Default::default()))
    }

    /// Try to intern a Type if it matches a common type, otherwise wrap in Arc.
    pub fn intern_or_wrap(union: Type) -> Arc<Type> {
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
        // Fast path 2: empty Type — also a common case (e.g. unresolved
        // return type). Don't pollute the intern table with these.
        if union.types.is_empty() {
            return Arc::new(union);
        }
        // Global path: dedup against any previously-seen identical Type.
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

fn serialize_template_bound<S>(value: &Option<Arc<Type>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_deref().serialize(serializer)
}

fn deserialize_template_bound<'de, D>(deserializer: D) -> Result<Option<Arc<Type>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Type>::deserialize(deserializer).map(|opt| opt.map(interned_types::intern_or_wrap))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: Name,
    /// Declared upper bound, e.g. `@template T of Traversable`.
    /// Stored as `Option<Arc<Type>>` so common bounds (e.g. `object`, `mixed`)
    /// are deduplicated across all template params via the global intern table.
    #[serde(
        serialize_with = "serialize_template_bound",
        deserialize_with = "deserialize_template_bound"
    )]
    pub bound: Option<Arc<Type>>,
    /// Default type used when nothing binds this template param, e.g.
    /// `@template T = string`. Falls back to `mixed` when absent, same as
    /// before this field existed.
    #[serde(
        default,
        serialize_with = "serialize_template_bound",
        deserialize_with = "deserialize_template_bound"
    )]
    pub default: Option<Arc<Type>>,
    /// The entity (class or function FQN) that declared this template param.
    pub defining_entity: Name,
    pub variance: mir_types::Variance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredParam {
    pub name: Name,
    /// Parameter type. Stored as `Option<Arc<Type>>` to enable deduplication of
    /// common types across parameters. Many parameters share types like `string`,
    /// `int`, `bool`, etc., so interning via Arc saves allocations.
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub ty: Option<Arc<Type>>,
    /// Out-type declared via `@param-out` / `@psalm-param-out`. When set, this
    /// type is written back to the caller's argument variable after the call
    /// instead of (or in addition to) the declared in-type.
    #[serde(
        default,
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub out_ty: Option<Arc<Type>>,
    /// Whether this parameter has a default value. During analysis, defaults are
    /// never used for their value — only for marking parameters as optional.
    pub has_default: bool,
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
}

impl std::hash::Hash for DeclaredParam {
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
        self.out_ty.as_deref().hash(state);
    }
}

// Serde helpers to transparently convert between Option<Type> and Option<Arc<Type>>
fn deserialize_param_type<'de, D>(deserializer: D) -> Result<Option<Arc<Type>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Type>::deserialize(deserializer).map(|opt| opt.map(interned_types::intern_or_wrap))
}

fn serialize_param_type<S>(value: &Option<Arc<Type>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let opt = value.as_ref().map(|arc| (**arc).clone());
    opt.serialize(serializer)
}

fn deserialize_return_type<'de, D>(deserializer: D) -> Result<Option<Arc<Type>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Type>::deserialize(deserializer).map(|opt| opt.map(interned_types::intern_or_wrap))
}

fn serialize_return_type<S>(value: &Option<Arc<Type>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let opt = value.as_ref().map(|arc| (**arc).clone());
    opt.serialize(serializer)
}

fn deserialize_params<'de, D>(deserializer: D) -> Result<Arc<[DeclaredParam]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<DeclaredParam>::deserialize(deserializer).map(|v| Arc::from(v.into_boxed_slice()))
}

fn default_imports() -> Arc<FxHashMap<Name, Name>> {
    Arc::new(FxHashMap::default())
}

/// Deserialize imports map. Supports both new (Name-keyed) and legacy
/// (String-keyed) on-disk formats — older `cache.bin` files have plain
/// `HashMap<String, String>`. Either way, we intern at load time so the
/// in-memory representation is always `Arc<FxHashMap<Name, Name>>`.
fn deserialize_imports<'de, D>(deserializer: D) -> Result<Arc<FxHashMap<Name, Name>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = FxHashMap::<String, String>::deserialize(deserializer)?;
    let mut out: FxHashMap<Name, Name> =
        FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());
    for (k, v) in raw {
        out.insert(Name::new(&k), Name::new(&v));
    }
    Ok(Arc::new(out))
}

/// Serialize imports as the legacy `HashMap<String, String>` shape so disk
/// caches written by this version remain compatible with readers that haven't
/// been recompiled yet (and vice-versa).
fn serialize_imports<S>(
    value: &Arc<FxHashMap<Name, Name>>,
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

fn serialize_params<S>(value: &Arc<[DeclaredParam]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_ref().serialize(serializer)
}

/// Helper to wrap Option<Type> in interned Arc<Type>.
pub fn wrap_param_type(ty: Option<Type>) -> Option<Arc<Type>> {
    ty.map(interned_types::intern_or_wrap)
}

/// Helper to wrap return type Option<Type> in interned Arc<Type>.
pub fn wrap_return_type(ty: Option<Type>) -> Option<Arc<Type>> {
    ty.map(interned_types::intern_or_wrap)
}

/// Helper to wrap a `PropertyDef` type field (`ty`/`inferred_ty`/`default`) in
/// an interned `Arc<Type>`, deduplicating common property types via the global
/// pool. See [`PropertyDef`].
pub fn wrap_property_type(ty: Option<Type>) -> Option<Arc<Type>> {
    ty.map(interned_types::intern_or_wrap)
}

/// Helper to wrap a `TemplateParam.bound` in an interned `Arc<Type>`.
pub fn wrap_template_bound(ty: Option<Type>) -> Option<Arc<Type>> {
    ty.map(interned_types::intern_or_wrap)
}

/// Wrap a variable type in an interned `Arc<Type>`. Use instead of
/// `Arc::new(ty)` at `FlowState::set_var` and parameter-init sites so that
/// common scalars (string, int, bool, null, mixed) share a static Arc rather
/// than allocating a fresh one per assignment.
pub fn wrap_var_type(ty: Type) -> Arc<Type> {
    interned_types::intern_or_wrap(ty)
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
    pub ty: Type,
    /// True for the `!Type` negated form (`@psalm-assert !null $x`): the
    /// parameter is asserted to NOT be `ty`, rather than to BE it.
    #[serde(default)]
    pub negated: bool,
}

// ---------------------------------------------------------------------------
// MethodDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDef {
    pub name: Arc<str>,
    pub fqcn: Arc<str>,
    #[serde(
        deserialize_with = "deserialize_params",
        serialize_with = "serialize_params"
    )]
    pub params: Arc<[DeclaredParam]>,
    /// Type from annotation (`@return` / native type hint). `None` means unannotated.
    /// Stored as `Option<Arc<Type>>` to enable deduplication of common return types
    /// (e.g., `void`, `string`, `mixed`, `bool`) across thousands of methods.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub return_type: Option<Arc<Type>>,
    /// Type inferred from body analysis. Stored as `Option<Arc<Type>>` (8 B) rather
    /// than inline `Option<Type>` (176 B, no niche) — inference is now demand-driven
    /// via salsa (`inferred_*_return_type_demand`), so this field is a rarely/never
    /// populated fallback; shrinking it saves ~168 B on every MethodDef.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub inferred_return_type: Option<Arc<Type>>,
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
    /// `@no-named-arguments` — callers must not use named argument syntax.
    #[serde(default)]
    pub no_named_arguments: bool,
    /// True when the method has the `#[Override]` PHP attribute.
    #[serde(default)]
    pub is_override: bool,
    pub location: Option<Location>,
    /// Plain-text description from the docblock (text before `@tag` lines).
    /// Used for hover info.
    #[serde(default)]
    pub docstring: Option<Arc<str>>,
    /// True for methods added via `@method` docblock annotations. Virtual
    /// methods must not be required as concrete interface implementations.
    #[serde(default)]
    pub is_virtual: bool,
    /// Parameters declared as taint sinks via `@taint-sink <kind> $param`.
    /// Each entry is `(param_name_without_dollar, sink_kind_string)`.
    #[serde(default)]
    pub taint_sink_params: Vec<(Arc<str>, Arc<str>)>,
    /// `@if-this-is Type` — the resolved constraint a receiver's type must
    /// satisfy for this method to be callable. `None` when absent.
    #[serde(default)]
    pub if_this_is: Option<Arc<Type>>,
    /// `@psalm-self-out Type` / `@phpstan-self-out Type` — the receiver's type
    /// after this call returns (e.g. a fluent builder that narrows `$this` as
    /// it's configured). `None` when absent.
    #[serde(default)]
    pub self_out: Option<Arc<Type>>,
    /// True when the method has `@inheritDoc` / `{@inheritDoc}` in its docblock.
    /// The analyzer inherits the parent's return type, param types, throws, and
    /// template params when this method has none of its own.
    #[serde(default)]
    pub is_inherit_doc: bool,
    /// `@psalm-mutation-free` / `@phpstan-mutation-free` — this method must not
    /// assign to `$this` properties (same constraint as `@psalm-immutable` on the
    /// class, but scoped to this single method).
    #[serde(default)]
    pub is_mutation_free: bool,
    /// `@psalm-external-mutation-free` — this method must not mutate any objects
    /// passed as arguments, but is allowed to modify `$this`.
    #[serde(default)]
    pub is_external_mutation_free: bool,
    /// Method names referenced via `@dataProvider name` / `#[DataProvider('name')]`
    /// (PHPUnit) — treated as used by dead-code analysis, since PHPUnit invokes
    /// them by name through reflection rather than a direct call site.
    #[serde(default)]
    pub data_provider_targets: Vec<Arc<str>>,
}

impl MethodDef {
    pub fn effective_return_type(&self) -> Option<&Type> {
        self.return_type
            .as_deref()
            .or(self.inferred_return_type.as_deref())
    }
}

// ---------------------------------------------------------------------------
// PropertyDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: Arc<str>,
    /// Declared/inferred/default types. Stored as `Option<Arc<Type>>` (8 B)
    /// rather than inline `Option<Type>` (176 B, no niche) and interned via the
    /// global pool on construction/deserialization — common property types
    /// (`string`, `int`, a shared class type) dedup to one allocation. Mirrors
    /// `DeclaredParam::ty`. On-disk format is unchanged (the serde helpers (de)serialize
    /// the inner `Type` transparently).
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub ty: Option<Arc<Type>>,
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub inferred_ty: Option<Arc<Type>>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_readonly: bool,
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub default: Option<Arc<Type>>,
    pub location: Option<Location>,
    /// `@deprecated` docblock annotation, if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
    /// True when the property declares a PHP native type hint (`public int $x`).
    /// A property typed only via a `@var` docblock (or untyped entirely) is
    /// `false`: PHP gives such a property an implicit `null` default, so it is
    /// never "uninitialized" (no MissingConstructor) and accepts `null` on
    /// assignment regardless of the advisory docblock type.
    #[serde(default)]
    pub has_native_type: bool,
    /// True when this entry was synthesised from a `@property` / `@property-read` /
    /// `@property-write` docblock tag rather than a real PHP property declaration.
    /// Such entries describe magic properties accessible via `__get`/`__set` and
    /// do **not** participate in PHP's inheritance visibility rules.
    #[serde(default)]
    pub from_docblock: bool,
    /// True when `readonly` comes from a native PHP keyword (`readonly` modifier or
    /// `readonly class`). False when only a `@readonly` docblock annotation is present.
    /// Distinguishes PHP-enforced read-only from advisory documentation.
    #[serde(default)]
    pub has_native_readonly: bool,
    /// The PHP native type hint alone, with any `@var` docblock refinement stripped —
    /// `None` when `has_native_type` is false. `ty` mixes in the docblock type when
    /// present, which makes it unsuitable for checking PHP's redeclared-property
    /// type invariance rule: that rule is enforced by the runtime purely on the
    /// native hint, never on the (unenforced) docblock annotation.
    #[serde(default)]
    #[serde(
        deserialize_with = "deserialize_param_type",
        serialize_with = "serialize_param_type"
    )]
    pub native_ty: Option<Arc<Type>>,
}

// ---------------------------------------------------------------------------
// ConstantDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: Arc<str>,
    pub ty: Type,
    pub visibility: Option<Visibility>,
    #[serde(default)]
    pub is_final: bool,
    pub location: Option<Location>,
    /// `@deprecated` docblock annotation, if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
}

// ---------------------------------------------------------------------------
// ClassDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDef {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub parent: Option<Arc<str>>,
    pub interfaces: Vec<Arc<str>>,
    pub traits: Vec<Arc<str>>,
    pub own_methods: MemberMap<Arc<MethodDef>>,
    pub own_properties: MemberMap<PropertyDef>,
    pub own_constants: MemberMap<ConstantDef>,
    #[serde(default)]
    pub mixins: Vec<Arc<str>>,
    pub template_params: Vec<TemplateParam>,
    /// Type arguments from `@extends ParentClass<T1, T2>` — maps parent's template params to concrete types.
    pub extends_type_args: Vec<Type>,
    /// Type arguments from `@implements Interface<T1, T2>`.
    #[serde(default)]
    pub implements_type_args: Vec<(Arc<str>, Vec<Type>)>,
    /// Type arguments from `@use TraitName<T1, T2>`, keyed by the used
    /// trait's FQCN — a class's `use` clause (unlike `@extends`) may name
    /// several traits at once.
    #[serde(default)]
    pub trait_use_type_args: Vec<(Arc<str>, Vec<Type>)>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_readonly: bool,
    pub deprecated: Option<Arc<str>>,
    pub is_internal: bool,
    /// Set when the class carries `@psalm-immutable` — non-constructor methods must not
    /// assign to `$this` properties.
    #[serde(default)]
    pub is_immutable: bool,
    /// Attribute target flags if this class has `#[Attribute]` annotation.
    /// `None` = not an attribute class. The value is a bitmask of PHP's
    /// `Attribute::TARGET_*` constants (e.g. `Attribute::TARGET_CLASS = 1`).
    #[serde(default)]
    pub attribute_flags: Option<i64>,
    pub location: Option<Location>,
    /// Per-`use` statement locations for each used trait: `(fqcn, location)` in
    /// declaration order, parallel to `traits`.  Absent from older serialized
    /// slices; defaults to empty.
    #[serde(default)]
    pub trait_use_locations: Vec<(Arc<str>, Location)>,
    /// Type aliases declared on this class via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Type>,
    /// Raw import-type declarations (`(local_name, original_name, from_class)`) — resolved during finalization.
    #[serde(default)]
    pub pending_import_types: Vec<(Arc<str>, Arc<str>, Arc<str>)>,
    /// Trait precedence exclusions from `insteadof` declarations in this class's `use` blocks.
    /// Maps method_name_lowercase → list of trait FQCNs whose version of the method is excluded.
    /// E.g. `use A, B { B::hello insteadof A; }` stores `"hello" → ["A"]`.
    #[serde(default)]
    pub trait_insteadof: MemberMap<Vec<Arc<str>>>,
    /// Trait method aliases from `as` declarations in this class's `use` blocks.
    /// Maps new_name_lowercase → (optional_trait_fqcn, original_method_name_lowercase, visibility_override, alias_cased).
    /// `alias_cased` is the alias name preserving the original PHP casing (for error messages / case checks).
    /// Visibility is `None` when the `as` clause only renames without changing visibility.
    /// E.g. `use Base { __construct as __constructBase; }` stores
    ///   `"__constructbase" → (None, "__construct", None, "__constructBase")`.
    /// E.g. `use T { foo as private traitFoo; }` stores
    ///   `"traitfoo" → (None, "foo", Some(Private), "traitFoo")`.
    #[serde(default)]
    #[allow(clippy::type_complexity)]
    pub trait_aliases:
        FxHashMap<Arc<str>, (Option<Arc<str>>, Arc<str>, Option<Visibility>, Arc<str>)>,
}

impl ClassDef {
    pub fn get_method(&self, name: &str) -> Option<&MethodDef> {
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

    pub fn get_property(&self, name: &str) -> Option<&PropertyDef> {
        self.own_properties.get(name)
    }
}

// ---------------------------------------------------------------------------
// InterfaceDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceDef {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub extends: Vec<Arc<str>>,
    pub own_methods: MemberMap<Arc<MethodDef>>,
    pub own_constants: MemberMap<ConstantDef>,
    pub template_params: Vec<TemplateParam>,
    pub location: Option<Location>,
    /// `@deprecated` docblock annotation, if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
    /// Properties declared via `@property*` docblock annotations on the interface.
    #[serde(default)]
    pub own_properties: MemberMap<PropertyDef>,
    /// `@seal-properties` / `@psalm-seal-properties` — disallows undeclared property access.
    #[serde(default)]
    pub seal_properties: bool,
    /// Type arguments from `@extends BaseIface<T1, T2>` docblock lines, keyed by the
    /// extended interface's FQCN — an interface's native `extends` list (unlike a
    /// class's single parent) may name several base interfaces at once.
    #[serde(default)]
    pub extends_type_args: Vec<(Arc<str>, Vec<Type>)>,
    /// Type aliases declared on this interface via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Type>,
}

// ---------------------------------------------------------------------------
// TraitDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDef {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub own_methods: MemberMap<Arc<MethodDef>>,
    pub own_properties: MemberMap<PropertyDef>,
    pub own_constants: MemberMap<ConstantDef>,
    pub template_params: Vec<TemplateParam>,
    /// Traits used by this trait (`use OtherTrait;` inside a trait body).
    pub traits: Vec<Arc<str>>,
    pub location: Option<Location>,
    /// Per-`use` statement locations for each used trait: `(fqcn, location)` in
    /// declaration order, parallel to `traits`. Mirrors `ClassDef`/`EnumDef`'s
    /// field of the same name. Absent from older serialized slices; defaults
    /// to empty.
    #[serde(default)]
    pub trait_use_locations: Vec<(Arc<str>, Location)>,
    /// Type arguments from `@use OtherTrait<T1, T2>` (a trait may itself
    /// `use` a generic trait).
    #[serde(default)]
    pub trait_use_type_args: Vec<(Arc<str>, Vec<Type>)>,
    /// `@psalm-require-extends` / `@phpstan-require-extends` — FQCNs that using classes must extend.
    #[serde(default)]
    pub require_extends: Vec<Arc<str>>,
    /// `@psalm-require-implements` / `@phpstan-require-implements` — FQCNs that using classes must implement.
    #[serde(default)]
    pub require_implements: Vec<Arc<str>>,
    /// `@deprecated` docblock annotation, if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
    /// Type aliases declared on this trait via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Type>,
}

// ---------------------------------------------------------------------------
// EnumDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumCaseDef {
    pub name: Arc<str>,
    pub value: Option<Type>,
    pub location: Option<Location>,
    /// `@deprecated` docblock annotation, if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub fqcn: Arc<str>,
    pub short_name: Arc<str>,
    pub scalar_type: Option<Type>,
    pub interfaces: Vec<Arc<str>>,
    /// Type arguments from `@implements Interface<T1, T2>`.
    #[serde(default)]
    pub implements_type_args: Vec<(Arc<str>, Vec<Type>)>,
    pub cases: MemberMap<EnumCaseDef>,
    pub own_methods: MemberMap<Arc<MethodDef>>,
    pub own_constants: MemberMap<ConstantDef>,
    /// `use SomeTrait;` declarations. PHP enums may use traits (for methods),
    /// just never carry instance properties from them.
    #[serde(default)]
    pub traits: Vec<Arc<str>>,
    #[serde(default)]
    pub trait_use_locations: Vec<(Arc<str>, Location)>,
    /// Type arguments from `@use SomeTrait<T1, T2>`.
    #[serde(default)]
    pub trait_use_type_args: Vec<(Arc<str>, Vec<Type>)>,
    pub location: Option<Location>,
    /// `@deprecated` docblock annotation (or `#[Deprecated]` attribute), if present.
    #[serde(default)]
    pub deprecated: Option<Arc<str>>,
    /// Type aliases declared on this enum via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Type>,
    /// Properties declared via `@property*` docblock annotations on the enum.
    #[serde(default)]
    pub own_properties: MemberMap<PropertyDef>,
}

// ---------------------------------------------------------------------------
// FunctionDef
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub fqn: Arc<str>,
    pub short_name: Arc<str>,
    #[serde(
        deserialize_with = "deserialize_params",
        serialize_with = "serialize_params"
    )]
    pub params: Arc<[DeclaredParam]>,
    /// Type from annotation (`@return` / native type hint). `None` means unannotated.
    /// Stored as `Option<Arc<Type>>` to enable deduplication of common return types.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub return_type: Option<Arc<Type>>,
    /// See `MethodDef::inferred_return_type` — `Option<Arc<Type>>` (8 B) for the
    /// same demand-driven-inference reason.
    #[serde(
        deserialize_with = "deserialize_return_type",
        serialize_with = "serialize_return_type"
    )]
    pub inferred_return_type: Option<Arc<Type>>,
    pub template_params: Vec<TemplateParam>,
    pub assertions: Vec<Assertion>,
    pub throws: Vec<Arc<str>>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
    /// `@no-named-arguments` — callers must not use named argument syntax.
    #[serde(default)]
    pub no_named_arguments: bool,
    pub location: Option<Location>,
    /// Plain-text description from the docblock (text before `@tag` lines).
    /// Used for hover info.
    #[serde(default)]
    pub docstring: Option<Arc<str>>,
    /// Parameters declared as taint sinks via `@taint-sink <kind> $param`.
    /// Each entry is `(param_name_without_dollar, sink_kind_string)`.
    #[serde(default)]
    pub taint_sink_params: Vec<(Arc<str>, Arc<str>)>,
    /// Type aliases declared on this function via `@psalm-type` / `@phpstan-type`.
    #[serde(default)]
    pub type_aliases: FxHashMap<Arc<str>, Type>,
}

impl FunctionDef {
    pub fn effective_return_type(&self) -> Option<&Type> {
        self.return_type
            .as_deref()
            .or(self.inferred_return_type.as_deref())
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
    pub classes: Vec<Arc<ClassDef>>,
    pub interfaces: Vec<Arc<InterfaceDef>>,
    pub traits: Vec<Arc<TraitDef>>,
    pub enums: Vec<Arc<EnumDef>>,
    pub functions: Vec<Arc<FunctionDef>>,
    #[serde(default)]
    pub constants: Vec<(Arc<str>, Type)>,
    /// Source file this slice was collected from. `None` for bundled stub slices
    /// that were pre-computed and are not tied to a specific on-disk file.
    #[serde(default)]
    pub file: Option<Arc<str>>,
    /// Types of `@var`-annotated global variables collected from this file.
    /// Populated by `DefinitionCollector`; ingested into the salsa db's
    /// `global_vars` table by `ingest_stub_slice` when `file` is `Some`.
    #[serde(default)]
    pub global_vars: Vec<(Arc<str>, Type)>,
    /// The first namespace declared in this file (e.g. `"App\\Service"`).
    /// Populated by `DefinitionCollector`; ingested into the salsa db's
    /// `file_namespaces` table by `ingest_stub_slice` when `file` is `Some`.
    #[serde(default)]
    pub namespace: Option<Arc<str>>,
    /// `use` alias map for this file: alias → FQCN.
    ///
    /// Stored as `Arc<FxHashMap<Name, Name>>` so that `file_imports()`
    /// returns a cheap Arc clone instead of deep-cloning the map on every
    /// `resolve_name` call (which fires once per symbol reference in
    /// Pass 2). `Name` keys/values shrink each entry from ~108 bytes
    /// (two `String` headers + two heap allocs averaging ~30 chars) to
    /// 16 bytes (two `Ustr` u64 handles); the global ustr interner holds
    /// one copy of each unique alias / FQCN string for the whole session.
    #[serde(
        deserialize_with = "deserialize_imports",
        serialize_with = "serialize_imports"
    )]
    #[serde(default = "default_imports")]
    pub imports: Arc<FxHashMap<Name, Name>>,
    /// Subset of `imports` containing only `use` items that import a
    /// class/interface/trait/enum (`UseKind::Normal`) — excludes `use
    /// function`/`use const` aliases. Class-name resolution consults this
    /// instead of `imports` so a function/constant import can't shadow a
    /// same-named class reference (`use function Foo\bar;` must not make an
    /// unrelated `bar` type hint resolve to `Foo\bar`).
    #[serde(
        deserialize_with = "deserialize_imports",
        serialize_with = "serialize_imports"
    )]
    #[serde(default = "default_imports")]
    pub class_imports: Arc<FxHashMap<Name, Name>>,
    /// Set to `true` after `deduplicate_params_in_slice` has run on this slice.
    /// `ingest_stub_slice` skips the clone+re-dedup when this flag is set.
    #[serde(skip)]
    pub is_deduped: bool,
}

// ---------------------------------------------------------------------------
// Param list deduplication
// ---------------------------------------------------------------------------

use std::sync::Mutex;

type ParamCache = Mutex<FxHashMap<Vec<DeclaredParam>, Arc<[DeclaredParam]>>>;

/// Global cache of canonical Arc<[DeclaredParam]> instances for deduplication.
/// Shared across all StubSlices to deduplicate vendor code with millions of
/// methods that often have identical parameter lists.
static PARAM_DEDUP_CACHE: std::sync::OnceLock<ParamCache> = std::sync::OnceLock::new();

/// Deduplicate parameter lists across all methods and functions in a StubSlice.
/// Many PHP framework methods share identical parameter lists (e.g., thousands
/// of `(string $arg, array $opts)` signatures). This function groups identical
/// param lists globally (across all slices processed so far) and replaces them
/// with Arc<[DeclaredParam]> pointers to shared allocations.
///
/// Expected memory savings: 100–150 MiB on cold start (vendor collection).
pub fn deduplicate_params_in_slice(slice: &mut StubSlice) {
    let cache: &ParamCache = PARAM_DEDUP_CACHE.get_or_init(|| Mutex::new(FxHashMap::default()));
    let mut canonical_params = cache.lock().unwrap();

    let mut deduplicate = |params: &mut Arc<[DeclaredParam]>| {
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
