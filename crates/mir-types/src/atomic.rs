use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Union;

// ---------------------------------------------------------------------------
// FnParam — used inside callable/closure atomics
// ---------------------------------------------------------------------------

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
// TemplateParam — `@template T` / `@template T of Bound`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: Arc<str>,
    pub bound: Option<Union>,
}

// ---------------------------------------------------------------------------
// KeyedProperty — entry in TKeyedArray
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArrayKey {
    String(Arc<str>),
    Int(i64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyedProperty {
    pub ty: Union,
    pub optional: bool,
}

// ---------------------------------------------------------------------------
// Atomic — every distinct PHP type variant
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Atomic {
    // --- Scalars ---

    /// `string`
    TString,
    /// `"hello"` — a specific string literal
    TLiteralString(Arc<str>),
    /// `class-string` or `class-string<T>`
    TClassString(Option<Arc<str>>),
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
        fqcn: Arc<str>,
        /// Resolved generic type arguments (e.g. `Collection<int>`)
        type_params: Vec<Union>,
    },
    /// `static` — late static binding type; resolved to calling class at call site
    TStaticObject { fqcn: Arc<str> },
    /// `self` — the class in whose body the type appears
    TSelf { fqcn: Arc<str> },
    /// `parent` — the parent class
    TParent { fqcn: Arc<str> },

    // --- Callables ---

    /// `callable` or `callable(T): R`
    TCallable {
        params: Option<Vec<FnParam>>,
        return_type: Option<Box<Union>>,
    },
    /// `Closure` or `Closure(T): R` — more specific than TCallable
    TClosure {
        params: Vec<FnParam>,
        return_type: Box<Union>,
        this_type: Option<Box<Union>>,
    },

    // --- Arrays ---

    /// `array` or `array<K, V>`
    TArray {
        key: Box<Union>,
        value: Box<Union>,
    },
    /// `list<T>` — integer-keyed sequential array (keys 0, 1, 2, …)
    TList { value: Box<Union> },
    /// `non-empty-array<K, V>`
    TNonEmptyArray {
        key: Box<Union>,
        value: Box<Union>,
    },
    /// `non-empty-list<T>`
    TNonEmptyList { value: Box<Union> },
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
        name: Arc<str>,
        as_type: Box<Union>,
        /// The entity (class or function FQN) that declared this template
        defining_entity: Arc<str>,
    },
    /// `(T is string ? A : B)` — conditional type
    TConditional {
        subject: Box<Union>,
        if_true: Box<Union>,
        if_false: Box<Union>,
    },

    // --- Special object strings ---

    /// `interface-string`
    TInterfaceString,
    /// `enum-string`
    TEnumString,
    /// `trait-string`
    TTraitString,
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
            Atomic::TNever | Atomic::TVoid => false,
            Atomic::TNull | Atomic::TFalse => false,
            Atomic::TLiteralInt(0) => false,
            Atomic::TLiteralFloat(0, 0) => false,
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
            Atomic::TString | Atomic::TLiteralString(_) | Atomic::TNonEmptyString | Atomic::TNumericString => "string",
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
        }
    }
}
