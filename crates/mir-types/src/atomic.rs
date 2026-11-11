use std::hash::Hash;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::symbol::Name;
use crate::Type;

// ---------------------------------------------------------------------------
// FnParam — used inside callable/closure atomics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnParam {
    pub name: Name,
    /// Parameter type stored as SimpleType for compact representation.
    /// Most params are simple scalars (string, int, etc.) and fit inline.
    pub ty: Option<crate::compact::SimpleType>,
    /// `@param-out` / `@psalm-param-out` writeback type. When `Some`, the
    /// variable passed for this by-ref param is set to this type after the call
    /// instead of the declared in-type (`ty`). Preserved through `TClosure` so
    /// first-class callables honour out-param annotations.
    #[serde(default)]
    pub out_ty: Option<crate::compact::SimpleType>,
    /// Default value stored as SimpleType. Usually None or a simple scalar.
    pub default: Option<crate::compact::SimpleType>,
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
}

// ---------------------------------------------------------------------------
// Variance — covariance / contravariance for template type parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Variance {
    /// Default: exact type match required (no `@template-covariant` / `@template-contravariant`).
    #[default]
    Invariant,
    /// `@template-covariant T` — `C<Sub>` is assignable to `C<Super>`.
    Covariant,
    /// `@template-contravariant T` — `C<Super>` is assignable to `C<Sub>`.
    Contravariant,
}

// ---------------------------------------------------------------------------
// TemplateParam — `@template T` / `@template T of Bound`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: Name,
    pub bound: Option<Type>,
    pub variance: Variance,
}

// ---------------------------------------------------------------------------
// KeyedProperty — entry in TKeyedArray
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArrayKey {
    String(Arc<str>),
    Int(i64),
}

impl PartialOrd for ArrayKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArrayKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ArrayKey::Int(a), ArrayKey::Int(b)) => a.cmp(b),
            (ArrayKey::String(a), ArrayKey::String(b)) => a.cmp(b),
            // Int < String
            (ArrayKey::Int(_), ArrayKey::String(_)) => std::cmp::Ordering::Less,
            (ArrayKey::String(_), ArrayKey::Int(_)) => std::cmp::Ordering::Greater,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyedProperty {
    pub ty: Type,
    pub optional: bool,
}

type KeyedMap = IndexMap<ArrayKey, KeyedProperty, rustc_hash::FxBuildHasher>;

/// Entries of an `array{...}` shape, in declaration order. Remembers its
/// order-independent hash, since a shared shape is hashed again whenever a
/// type holding it is interned; any mutable access forgets it.
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyedProperties {
    map: KeyedMap,
    #[serde(skip)]
    hash: std::sync::OnceLock<u64>,
}

impl KeyedProperties {
    pub fn with_capacity_and_hasher(n: usize, hasher: rustc_hash::FxBuildHasher) -> Self {
        KeyedMap::with_capacity_and_hasher(n, hasher).into()
    }

    /// Hash of the entries, independent of their order (equality ignores it too).
    pub fn entries_hash(&self) -> u64 {
        *self.hash.get_or_init(|| {
            self.map.iter().fold(0u64, |combined, (k, v)| {
                let mut entry_hasher = rustc_hash::FxHasher::default();
                k.hash(&mut entry_hasher);
                v.hash(&mut entry_hasher);
                combined.wrapping_add(std::hash::Hasher::finish(&entry_hasher))
            })
        })
    }
}

impl From<KeyedMap> for KeyedProperties {
    fn from(map: KeyedMap) -> Self {
        KeyedProperties {
            map,
            hash: std::sync::OnceLock::new(),
        }
    }
}

impl Clone for KeyedProperties {
    fn clone(&self) -> Self {
        KeyedProperties {
            map: self.map.clone(),
            hash: self.hash.clone(),
        }
    }
}

impl std::fmt::Debug for KeyedProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}

impl PartialEq for KeyedProperties {
    fn eq(&self, other: &Self) -> bool {
        // Remembered hashes that differ settle it without visiting the entries.
        if let (Some(a), Some(b)) = (self.hash.get(), other.hash.get()) {
            if a != b {
                return false;
            }
        }
        self.map == other.map
    }
}

impl Eq for KeyedProperties {}

impl std::ops::Deref for KeyedProperties {
    type Target = KeyedMap;
    fn deref(&self) -> &KeyedMap {
        &self.map
    }
}

impl std::ops::DerefMut for KeyedProperties {
    fn deref_mut(&mut self) -> &mut KeyedMap {
        self.hash = std::sync::OnceLock::new();
        &mut self.map
    }
}

impl FromIterator<(ArrayKey, KeyedProperty)> for KeyedProperties {
    fn from_iter<I: IntoIterator<Item = (ArrayKey, KeyedProperty)>>(iter: I) -> Self {
        KeyedMap::from_iter(iter).into()
    }
}

impl Extend<(ArrayKey, KeyedProperty)> for KeyedProperties {
    fn extend<I: IntoIterator<Item = (ArrayKey, KeyedProperty)>>(&mut self, iter: I) {
        std::ops::DerefMut::deref_mut(self).extend(iter);
    }
}

impl IntoIterator for KeyedProperties {
    type Item = (ArrayKey, KeyedProperty);
    type IntoIter = indexmap::map::IntoIter<ArrayKey, KeyedProperty>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a> IntoIterator for &'a KeyedProperties {
    type Item = (&'a ArrayKey, &'a KeyedProperty);
    type IntoIter = indexmap::map::Iter<'a, ArrayKey, KeyedProperty>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a> IntoIterator for &'a mut KeyedProperties {
    type Item = (&'a ArrayKey, &'a mut KeyedProperty);
    type IntoIter = indexmap::map::IterMut<'a, ArrayKey, KeyedProperty>;
    fn into_iter(self) -> Self::IntoIter {
        std::ops::DerefMut::deref_mut(self).iter_mut()
    }
}

// ---------------------------------------------------------------------------
// Boxed variant payloads — keep `Atomic` (and `Type`, which inlines two) small
// ---------------------------------------------------------------------------

/// Payload of [`Atomic::TClosure`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClosureData {
    pub params: Box<[FnParam]>,
    pub return_type: Type,
    pub this_type: Option<Type>,
    /// `pure-Closure(...)`, or a closure literal whose body is checked as pure.
    pub is_pure: bool,
    /// A `static` closure literal, which has no `$this` to mutate.
    pub is_static: bool,
    /// A first-class callable of a named function or method, so surplus arguments are reported.
    #[serde(default)]
    pub rejects_surplus_args: bool,
}

/// Payload of [`Atomic::TConditional`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConditionalData {
    /// The parameter name being tested (without `$`), e.g. `"classOrInterface"`.
    /// `None` for conditionals that were not parsed from a `$param is` form.
    pub param_name: Option<Name>,
    pub subject: Type,
    pub if_true: Type,
    pub if_false: Type,
}

/// What a [`Atomic::TRefinedInt`] guarantees about its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntRefinement {
    /// `literal-int` — an int written in the source, not read at runtime.
    Literal,
    /// `int-mask-of<Cls::PATTERN>` over the constants of a class other than the declaring one.
    MaskOf(Name),
}

/// What a [`Atomic::TRefinedString`] guarantees about its characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StringRefinement {
    Lowercase,
    Uppercase,
    /// `literal-string` — a value written in the source, not read at runtime.
    Literal,
}

impl StringRefinement {
    pub fn accepts(self, s: &str) -> bool {
        match self {
            StringRefinement::Lowercase => !s.bytes().any(|b| b.is_ascii_uppercase()),
            StringRefinement::Uppercase => !s.bytes().any(|b| b.is_ascii_lowercase()),
            StringRefinement::Literal => true,
        }
    }
}

// ---------------------------------------------------------------------------
// Atomic — every distinct PHP type variant
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Atomic {
    // --- Scalars ---
    /// `string`
    TString,
    /// `"hello"` — a specific string literal
    TLiteralString(Arc<str>),
    /// `callable-string` — a string containing a callable name
    TCallableString,
    /// `class-string` or `class-string<T>`
    TClassString(Option<Name>),
    /// `non-empty-string`
    TNonEmptyString,
    /// `non-falsy-string` — neither `""` nor `"0"`
    TNonFalsyString,
    /// `numeric-string`
    TNumericString,
    /// `lowercase-string`, `uppercase-string`, `literal-string` and their `non-empty-` forms.
    TRefinedString {
        refinement: StringRefinement,
        non_empty: bool,
    },

    /// `int`
    TInt,
    /// `42` — a specific integer literal
    TLiteralInt(i64),
    /// `int<min, max>` — bounded integer range
    TIntRange { min: Option<i64>, max: Option<i64> },
    /// `positive-int`
    TPositiveInt,
    /// `negative-int`
    TNegativeInt,
    /// `non-negative-int`
    TNonNegativeInt,
    /// `literal-int` or an unresolved `int-mask-of<Cls::*>`.
    TRefinedInt(IntRefinement),

    /// `float`
    TFloat,
    /// `float` that is always integral (whole-number value, no fractional part).
    /// Returned by `floor()`, `ceil()`, and zero-precision `round()`. Can be passed to `int`
    /// parameters without `ImplicitFloatToIntCast` because the conversion is lossless.
    TIntegralFloat,
    /// `3.14` — a specific float literal
    TLiteralFloat(i64, i64), // stored as (int_bits, frac_bits) to be PartialEq+Hash-friendly
    // We use ordered_float or just store as ordered pair for equality purposes.
    /// `bool`
    TBool,
    /// `true`
    TTrue,
    /// `false`
    TFalse,

    /// `null`
    TNull,

    // --- Bottom / top ---
    /// `void` — return-only; can't be used as a value
    TVoid,
    /// `never` — function that never returns (throws or infinite loop)
    TNever,
    /// `mixed` — top type; accepts anything
    TMixed,
    /// `scalar` — int | float | string | bool
    TScalar,
    /// `numeric` — int | float | numeric-string
    TNumeric,

    // --- Objects ---
    /// `object` — any object
    TObject,
    /// `ClassName` / `ClassName<T1, T2>` — specific named class/interface
    TNamedObject {
        fqcn: Name,
        /// Resolved generic type arguments (e.g. `Collection<int>`)
        type_params: Arc<[Type]>,
    },
    /// `static` — late static binding type; resolved to calling class at call site
    TStaticObject { fqcn: Name },
    /// `self` — the class in whose body the type appears
    TSelf { fqcn: Name },
    /// `parent` — the parent class
    TParent { fqcn: Name },

    // --- Callables ---
    /// `callable` or `callable(T): R`
    TCallable {
        /// Thin box: an inline `Vec` or boxed slice here, next to `is_pure`,
        /// would make this the largest variant and grow every `Atomic` (and
        /// `Type` inlines two of them).
        params: Option<Box<Vec<FnParam>>>,
        return_type: Option<Box<Type>>,
        /// `pure-callable`.
        is_pure: bool,
    },
    /// `Closure` or `Closure(T): R` — more specific than TCallable.
    /// Payload boxed to keep `Atomic` at 32 bytes (see [`ClosureData`]).
    TClosure { data: Box<ClosureData> },

    // --- Arrays ---
    /// `array` or `array<K, V>`
    TArray { key: Box<Type>, value: Box<Type> },
    /// `list<T>` — integer-keyed sequential array (keys 0, 1, 2, …)
    TList { value: Box<Type> },
    /// `non-empty-array<K, V>`
    TNonEmptyArray { key: Box<Type>, value: Box<Type> },
    /// `non-empty-list<T>`
    TNonEmptyList { value: Box<Type> },
    /// `array{key: T, ...}` — shape / keyed array
    TKeyedArray {
        /// Shared so cloning a shape copies a pointer and equal clones compare by address.
        properties: Arc<KeyedProperties>,
        /// If true, additional keys beyond the declared ones may exist
        is_open: bool,
        /// If true, the shape represents a list (integer keys only)
        is_list: bool,
    },

    // --- Generics / meta-types ---
    /// `T` — a template type parameter
    TTemplateParam {
        name: Name,
        as_type: Box<Type>,
        /// The entity (class or function FQN) that declared this template
        defining_entity: Name,
    },
    /// `key-of<T>` — preserved when the operand cannot yet be reduced.
    TKeyOf { target: Box<Type> },
    /// `value-of<T>` — preserved when the operand cannot yet be reduced.
    TValueOf { target: Box<Type> },
    /// `($param is TypeName ? A : B)` — conditional type.
    /// Payload boxed to keep `Atomic` at 32 bytes (see [`ConditionalData`]).
    TConditional { data: Box<ConditionalData> },

    // --- Special object strings ---
    /// `interface-string` or `interface-string<T>`
    TInterfaceString(Option<Name>),
    /// `enum-string`
    TEnumString,
    /// `trait-string`
    TTraitString,

    // --- Enum cases ---
    /// `EnumName::CaseName` — a specific enum case literal
    TLiteralEnumCase { enum_fqcn: Name, case_name: Name },

    // --- Intersection ---
    /// `A&B&C` — PHP 8.1+ pure intersection type
    TIntersection { parts: Arc<[Type]> },

    /// `object{prop: T, ...}` — an object with these public properties.
    TObjectShape { properties: Arc<KeyedProperties> },

    /// `resource` — a legacy resource handle (open or closed); docblock-only, never a native type.
    TResource,
}

impl Atomic {
    /// Whether this atomic type can ever evaluate to a falsy value.
    pub fn can_be_falsy(&self) -> bool {
        match self {
            Atomic::TNull
            | Atomic::TFalse
            | Atomic::TBool
            | Atomic::TNever
            | Atomic::TLiteralInt(0)
            | Atomic::TLiteralFloat(0, 0)
            | Atomic::TInt
            | Atomic::TRefinedInt(_)
            | Atomic::TFloat
            | Atomic::TNumeric
            | Atomic::TScalar
            | Atomic::TMixed
            | Atomic::TString
            | Atomic::TNonEmptyString  // "0" is both non-empty and falsy in PHP
            | Atomic::TNumericString   // "0" is a valid numeric-string and is falsy
            | Atomic::TRefinedString { .. }
            | Atomic::TKeyOf { .. }
            | Atomic::TValueOf { .. }
            | Atomic::TArray { .. }
            | Atomic::TList { .. } => true,
            // Non-empty collections always have at least one element — never falsy in PHP.
            // TNonEmptyArray and TNonEmptyList are intentionally excluded here.

            Atomic::TLiteralString(s) => s.as_ref().is_empty() || s.as_ref() == "0",

            Atomic::TKeyedArray { properties, .. } => properties.values().all(|p| p.optional),

            // An int range can be falsy (== 0) when 0 is within the bounds.
            Atomic::TIntRange { min, max } => {
                min.is_none_or(|lo| lo <= 0) && max.is_none_or(|hi| hi >= 0)
            }
            // non-negative-int includes 0
            Atomic::TNonNegativeInt => true,

            _ => false,
        }
    }

    /// A template parameter or a `key-of`/`value-of` over one: its concrete type is unknown here.
    pub fn is_deferred(&self) -> bool {
        matches!(
            self,
            Atomic::TTemplateParam { .. } | Atomic::TKeyOf { .. } | Atomic::TValueOf { .. }
        )
    }

    /// Whether this atomic type can ever evaluate to a truthy value.
    pub fn can_be_truthy(&self) -> bool {
        match self {
            Atomic::TNever
            | Atomic::TVoid
            | Atomic::TNull
            | Atomic::TFalse
            | Atomic::TLiteralInt(0)
            | Atomic::TLiteralFloat(0, 0) => false,
            Atomic::TLiteralString(s) if s.as_ref() == "" || s.as_ref() == "0" => false,
            // int<0, 0> always equals zero — never truthy.
            Atomic::TIntRange {
                min: Some(0),
                max: Some(0),
            } => false,
            // array{} — a closed empty keyed array — is always falsy.
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } if !is_open && properties.is_empty() => false,
            _ => true,
        }
    }

    /// Whether this atomic represents a numeric type (int, float, or numeric-string).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Atomic::TInt
                | Atomic::TLiteralInt(_)
                | Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNegativeInt
                | Atomic::TNonNegativeInt
                | Atomic::TRefinedInt(_)
                | Atomic::TFloat
                | Atomic::TIntegralFloat
                | Atomic::TLiteralFloat(..)
                | Atomic::TNumeric
                | Atomic::TNumericString
        )
    }

    /// Whether this atomic is an integer variant.
    pub fn is_int(&self) -> bool {
        matches!(
            self,
            Atomic::TInt
                | Atomic::TLiteralInt(_)
                | Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNegativeInt
                | Atomic::TNonNegativeInt
                | Atomic::TRefinedInt(_)
        )
    }

    /// Whether this atomic is a string variant.
    pub fn is_string(&self) -> bool {
        matches!(
            self,
            Atomic::TString
                | Atomic::TLiteralString(_)
                | Atomic::TCallableString
                | Atomic::TClassString(_)
                | Atomic::TNonEmptyString
                | Atomic::TNonFalsyString
                | Atomic::TRefinedString { .. }
                | Atomic::TNumericString
                | Atomic::TInterfaceString(_)
                | Atomic::TEnumString
                | Atomic::TTraitString
        )
    }

    /// Whether this atomic is an array variant.
    pub fn is_array(&self) -> bool {
        matches!(
            self,
            Atomic::TArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    }

    /// Whether this atomic is an object variant.
    pub fn is_object(&self) -> bool {
        matches!(
            self,
            Atomic::TObject
                | Atomic::TObjectShape { .. }
                | Atomic::TNamedObject { .. }
                | Atomic::TStaticObject { .. }
                | Atomic::TSelf { .. }
                | Atomic::TParent { .. }
                | Atomic::TIntersection { .. }
        )
    }

    /// Whether this atomic can never be an object instance — so `clone`-ing it
    /// is invalid. Conservative: ambiguous atomics (`mixed`, `callable`,
    /// `never`, conditionals, template params, enum cases, intersections,
    /// object types) all return `false` so they never trigger a false positive.
    pub fn is_definitely_non_object(&self) -> bool {
        matches!(
            self,
            Atomic::TString
                | Atomic::TLiteralString(_)
                | Atomic::TCallableString
                | Atomic::TClassString(_)
                | Atomic::TNonEmptyString
                | Atomic::TNonFalsyString
                | Atomic::TRefinedString { .. }
                | Atomic::TNumericString
                | Atomic::TInterfaceString(_)
                | Atomic::TEnumString
                | Atomic::TTraitString
                | Atomic::TInt
                | Atomic::TLiteralInt(_)
                | Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNegativeInt
                | Atomic::TNonNegativeInt
                | Atomic::TRefinedInt(_)
                | Atomic::TFloat
                | Atomic::TIntegralFloat
                | Atomic::TLiteralFloat(..)
                | Atomic::TBool
                | Atomic::TTrue
                | Atomic::TFalse
                | Atomic::TNull
                | Atomic::TVoid
                | Atomic::TArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
                | Atomic::TScalar
                | Atomic::TNumeric
        )
    }

    /// Whether this atomic is a callable variant.
    ///
    /// `TCallableString` is included: it denotes a string *known* to be a
    /// valid callable, so `!is_callable()` must remove it from the type.
    pub fn is_callable(&self) -> bool {
        matches!(
            self,
            Atomic::TCallable { .. } | Atomic::TClosure { .. } | Atomic::TCallableString
        )
    }

    /// Returns the FQCN if this is a named object type.
    pub fn named_object_fqcn(&self) -> Option<&str> {
        match self {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TSelf { fqcn }
            | Atomic::TParent { fqcn } => Some(fqcn.as_ref()),
            _ => None,
        }
    }

    /// A human-readable name for this type (used in error messages).
    pub fn type_name(&self) -> &'static str {
        match self {
            Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TNonEmptyString
            | Atomic::TNonFalsyString
            | Atomic::TRefinedString { .. }
            | Atomic::TNumericString => "string",
            Atomic::TCallableString => "callable-string",
            Atomic::TClassString(_) => "class-string",
            Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TRefinedInt(_) => "int",
            Atomic::TPositiveInt => "positive-int",
            Atomic::TNegativeInt => "negative-int",
            Atomic::TNonNegativeInt => "non-negative-int",
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => "float",
            Atomic::TBool => "bool",
            Atomic::TTrue => "true",
            Atomic::TFalse => "false",
            Atomic::TNull => "null",
            Atomic::TVoid => "void",
            Atomic::TNever => "never",
            Atomic::TMixed => "mixed",
            Atomic::TScalar => "scalar",
            Atomic::TNumeric => "numeric",
            Atomic::TObject => "object",
            Atomic::TResource => "resource",
            Atomic::TNamedObject { .. } => "object",
            Atomic::TStaticObject { .. } => "static",
            Atomic::TSelf { .. } => "self",
            Atomic::TParent { .. } => "parent",
            Atomic::TCallable { .. } => "callable",
            Atomic::TClosure { .. } => "Closure",
            Atomic::TArray { .. } => "array",
            Atomic::TList { .. } => "list",
            Atomic::TNonEmptyArray { .. } => "non-empty-array",
            Atomic::TNonEmptyList { .. } => "non-empty-list",
            Atomic::TKeyedArray { .. } => "array",
            Atomic::TTemplateParam { .. } => "template-param",
            Atomic::TKeyOf { .. } => "key-of",
            Atomic::TValueOf { .. } => "value-of",
            Atomic::TConditional { .. } => "conditional-type",
            Atomic::TInterfaceString(_) => "interface-string",
            Atomic::TEnumString => "enum-string",
            Atomic::TTraitString => "trait-string",
            Atomic::TLiteralEnumCase { .. } => "enum-case",
            Atomic::TIntersection { .. } => "intersection",
            Atomic::TObjectShape { .. } => "object",
        }
    }
}

// ---------------------------------------------------------------------------
// Hash impls
// ---------------------------------------------------------------------------

impl Hash for FnParam {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ty.hash(state);
        self.out_ty.hash(state);
        self.default.hash(state);
        self.is_variadic.hash(state);
        self.is_byref.hash(state);
        self.is_optional.hash(state);
    }
}

/// Discriminant tags for `Atomic` used in the manual `Hash` impl below.
#[allow(non_camel_case_types, clippy::enum_variant_names)]
#[repr(u8)]
enum AtomicTag {
    TString = 0,
    TLiteralString,
    TCallableString,
    TClassString,
    TNonEmptyString,
    TNonFalsyString,
    TNumericString,
    TRefinedString,
    TInt,
    TLiteralInt,
    TIntRange,
    TPositiveInt,
    TNegativeInt,
    TNonNegativeInt,
    TRefinedInt,
    TFloat,
    TIntegralFloat,
    TLiteralFloat,
    TBool,
    TTrue,
    TFalse,
    TNull,
    TVoid,
    TNever,
    TMixed,
    TScalar,
    TNumeric,
    TObject,
    TNamedObject,
    TStaticObject,
    TSelf,
    TParent,
    TCallable,
    TClosure,
    TArray,
    TList,
    TNonEmptyArray,
    TNonEmptyList,
    TKeyedArray,
    TTemplateParam,
    TKeyOf,
    TValueOf,
    TConditional,
    TInterfaceString,
    TEnumString,
    TTraitString,
    TLiteralEnumCase,
    TIntersection,
    TObjectShape,
    TResource,
}

impl Hash for Atomic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use AtomicTag as T;
        match self {
            // --- tag-only variants ---
            Atomic::TString => (T::TString as u8).hash(state),
            Atomic::TCallableString => (T::TCallableString as u8).hash(state),
            Atomic::TNonEmptyString => (T::TNonEmptyString as u8).hash(state),
            Atomic::TNonFalsyString => (T::TNonFalsyString as u8).hash(state),
            Atomic::TNumericString => (T::TNumericString as u8).hash(state),
            Atomic::TRefinedString {
                refinement,
                non_empty,
            } => {
                (T::TRefinedString as u8).hash(state);
                refinement.hash(state);
                non_empty.hash(state);
            }
            Atomic::TInt => (T::TInt as u8).hash(state),
            Atomic::TPositiveInt => (T::TPositiveInt as u8).hash(state),
            Atomic::TNegativeInt => (T::TNegativeInt as u8).hash(state),
            Atomic::TNonNegativeInt => (T::TNonNegativeInt as u8).hash(state),
            Atomic::TRefinedInt(refinement) => {
                (T::TRefinedInt as u8).hash(state);
                refinement.hash(state);
            }
            Atomic::TFloat => (T::TFloat as u8).hash(state),
            Atomic::TIntegralFloat => (T::TIntegralFloat as u8).hash(state),
            Atomic::TBool => (T::TBool as u8).hash(state),
            Atomic::TTrue => (T::TTrue as u8).hash(state),
            Atomic::TFalse => (T::TFalse as u8).hash(state),
            Atomic::TNull => (T::TNull as u8).hash(state),
            Atomic::TVoid => (T::TVoid as u8).hash(state),
            Atomic::TNever => (T::TNever as u8).hash(state),
            Atomic::TMixed => (T::TMixed as u8).hash(state),
            Atomic::TScalar => (T::TScalar as u8).hash(state),
            Atomic::TNumeric => (T::TNumeric as u8).hash(state),
            Atomic::TObject => (T::TObject as u8).hash(state),
            Atomic::TEnumString => (T::TEnumString as u8).hash(state),
            Atomic::TTraitString => (T::TTraitString as u8).hash(state),
            Atomic::TResource => (T::TResource as u8).hash(state),

            // --- variants with fields ---
            Atomic::TLiteralString(s) => {
                (T::TLiteralString as u8).hash(state);
                s.hash(state);
            }
            Atomic::TClassString(opt) => {
                (T::TClassString as u8).hash(state);
                opt.hash(state);
            }
            Atomic::TInterfaceString(opt) => {
                (T::TInterfaceString as u8).hash(state);
                opt.hash(state);
            }
            Atomic::TLiteralInt(n) => {
                (T::TLiteralInt as u8).hash(state);
                n.hash(state);
            }
            Atomic::TIntRange { min, max } => {
                (T::TIntRange as u8).hash(state);
                min.hash(state);
                max.hash(state);
            }
            Atomic::TLiteralFloat(int_bits, frac_bits) => {
                (T::TLiteralFloat as u8).hash(state);
                int_bits.hash(state);
                frac_bits.hash(state);
            }
            Atomic::TNamedObject { fqcn, type_params } => {
                (T::TNamedObject as u8).hash(state);
                fqcn.hash(state);
                type_params.hash(state);
            }
            Atomic::TStaticObject { fqcn } => {
                (T::TStaticObject as u8).hash(state);
                fqcn.hash(state);
            }
            Atomic::TSelf { fqcn } => {
                (T::TSelf as u8).hash(state);
                fqcn.hash(state);
            }
            Atomic::TParent { fqcn } => {
                (T::TParent as u8).hash(state);
                fqcn.hash(state);
            }
            Atomic::TCallable {
                params,
                return_type,
                is_pure,
            } => {
                (T::TCallable as u8).hash(state);
                params.hash(state);
                return_type.hash(state);
                is_pure.hash(state);
            }
            Atomic::TClosure { data } => {
                (T::TClosure as u8).hash(state);
                data.hash(state);
            }
            Atomic::TArray { key, value } => {
                (T::TArray as u8).hash(state);
                key.hash(state);
                value.hash(state);
            }
            Atomic::TList { value } => {
                (T::TList as u8).hash(state);
                value.hash(state);
            }
            Atomic::TNonEmptyArray { key, value } => {
                (T::TNonEmptyArray as u8).hash(state);
                key.hash(state);
                value.hash(state);
            }
            Atomic::TNonEmptyList { value } => {
                (T::TNonEmptyList as u8).hash(state);
                value.hash(state);
            }
            Atomic::TObjectShape { properties } => {
                (T::TObjectShape as u8).hash(state);
                properties.entries_hash().hash(state);
                properties.len().hash(state);
            }
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                (T::TKeyedArray as u8).hash(state);
                properties.entries_hash().hash(state);
                properties.len().hash(state);
                is_open.hash(state);
                is_list.hash(state);
            }
            Atomic::TTemplateParam {
                name,
                as_type,
                defining_entity,
            } => {
                (T::TTemplateParam as u8).hash(state);
                name.hash(state);
                as_type.hash(state);
                defining_entity.hash(state);
            }
            Atomic::TKeyOf { target } => {
                (T::TKeyOf as u8).hash(state);
                target.hash(state);
            }
            Atomic::TValueOf { target } => {
                (T::TValueOf as u8).hash(state);
                target.hash(state);
            }
            Atomic::TConditional { data } => {
                (T::TConditional as u8).hash(state);
                data.hash(state);
            }
            Atomic::TLiteralEnumCase {
                enum_fqcn,
                case_name,
            } => {
                (T::TLiteralEnumCase as u8).hash(state);
                enum_fqcn.hash(state);
                case_name.hash(state);
            }
            Atomic::TIntersection { parts } => {
                (T::TIntersection as u8).hash(state);
                parts.hash(state);
            }
        }
    }
}
