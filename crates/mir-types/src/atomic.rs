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
    /// `numeric-string`
    TNumericString,

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

    /// `float`
    TFloat,
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
        params: Option<Vec<FnParam>>,
        return_type: Option<Box<Type>>,
    },
    /// `Closure` or `Closure(T): R` — more specific than TCallable
    TClosure {
        params: Vec<FnParam>,
        return_type: Box<Type>,
        this_type: Option<Box<Type>>,
    },

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
        properties: IndexMap<ArrayKey, KeyedProperty>,
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
    /// `($param is TypeName ? A : B)` — conditional type
    TConditional {
        /// The parameter name being tested (without `$`), e.g. `"classOrInterface"`.
        /// `None` for conditionals that were not parsed from a `$param is` form.
        param_name: Option<Name>,
        subject: Box<Type>,
        if_true: Box<Type>,
        if_false: Box<Type>,
    },

    // --- Special object strings ---
    /// `interface-string`
    TInterfaceString,
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
            | Atomic::TFloat
            | Atomic::TNumeric
            | Atomic::TScalar
            | Atomic::TMixed
            | Atomic::TString
            | Atomic::TNonEmptyString
            | Atomic::TArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TNonEmptyList { .. } => true,

            Atomic::TLiteralString(s) => s.as_ref().is_empty() || s.as_ref() == "0",

            Atomic::TKeyedArray { properties, .. } => properties.is_empty(),

            _ => false,
        }
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
                | Atomic::TFloat
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
                | Atomic::TNumericString
                | Atomic::TInterfaceString
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
                | Atomic::TNamedObject { .. }
                | Atomic::TStaticObject { .. }
                | Atomic::TSelf { .. }
                | Atomic::TParent { .. }
                | Atomic::TIntersection { .. }
        )
    }

    /// Whether this atomic is a callable variant.
    pub fn is_callable(&self) -> bool {
        matches!(self, Atomic::TCallable { .. } | Atomic::TClosure { .. })
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
            | Atomic::TNumericString => "string",
            Atomic::TCallableString => "callable-string",
            Atomic::TClassString(_) => "class-string",
            Atomic::TInt | Atomic::TLiteralInt(_) | Atomic::TIntRange { .. } => "int",
            Atomic::TPositiveInt => "positive-int",
            Atomic::TNegativeInt => "negative-int",
            Atomic::TNonNegativeInt => "non-negative-int",
            Atomic::TFloat | Atomic::TLiteralFloat(..) => "float",
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
            Atomic::TConditional { .. } => "conditional-type",
            Atomic::TInterfaceString => "interface-string",
            Atomic::TEnumString => "enum-string",
            Atomic::TTraitString => "trait-string",
            Atomic::TLiteralEnumCase { .. } => "enum-case",
            Atomic::TIntersection { .. } => "intersection",
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
    TNumericString,
    TInt,
    TLiteralInt,
    TIntRange,
    TPositiveInt,
    TNegativeInt,
    TNonNegativeInt,
    TFloat,
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
    TConditional,
    TInterfaceString,
    TEnumString,
    TTraitString,
    TLiteralEnumCase,
    TIntersection,
}

impl Hash for Atomic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use AtomicTag as T;
        match self {
            // --- tag-only variants ---
            Atomic::TString => (T::TString as u8).hash(state),
            Atomic::TCallableString => (T::TCallableString as u8).hash(state),
            Atomic::TNonEmptyString => (T::TNonEmptyString as u8).hash(state),
            Atomic::TNumericString => (T::TNumericString as u8).hash(state),
            Atomic::TInt => (T::TInt as u8).hash(state),
            Atomic::TPositiveInt => (T::TPositiveInt as u8).hash(state),
            Atomic::TNegativeInt => (T::TNegativeInt as u8).hash(state),
            Atomic::TNonNegativeInt => (T::TNonNegativeInt as u8).hash(state),
            Atomic::TFloat => (T::TFloat as u8).hash(state),
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
            Atomic::TInterfaceString => (T::TInterfaceString as u8).hash(state),
            Atomic::TEnumString => (T::TEnumString as u8).hash(state),
            Atomic::TTraitString => (T::TTraitString as u8).hash(state),

            // --- variants with fields ---
            Atomic::TLiteralString(s) => {
                (T::TLiteralString as u8).hash(state);
                s.hash(state);
            }
            Atomic::TClassString(opt) => {
                (T::TClassString as u8).hash(state);
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
            } => {
                (T::TCallable as u8).hash(state);
                params.hash(state);
                return_type.hash(state);
            }
            Atomic::TClosure {
                params,
                return_type,
                this_type,
            } => {
                (T::TClosure as u8).hash(state);
                params.hash(state);
                return_type.hash(state);
                this_type.hash(state);
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
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                (T::TKeyedArray as u8).hash(state);
                // Sort by key before hashing so the hash is order-independent,
                // consistent with PartialEq which uses IndexMap value equality.
                let mut pairs: Vec<_> = properties.iter().collect();
                pairs.sort_by_key(|(a, _)| *a);
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash(state);
                }
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
            Atomic::TConditional {
                param_name,
                subject,
                if_true,
                if_false,
            } => {
                (T::TConditional as u8).hash(state);
                param_name.hash(state);
                subject.hash(state);
                if_true.hash(state);
                if_false.hash(state);
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
