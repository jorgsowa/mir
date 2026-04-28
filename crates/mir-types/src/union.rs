use std::sync::Arc;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::atomic::Atomic;

// Most unions contain 1-2 atomics (e.g. `string|null`), so we inline two.
pub type AtomicVec = SmallVec<[Atomic; 2]>;

// ---------------------------------------------------------------------------
// Union — the primary type carrier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Union {
    pub types: AtomicVec,
    /// The variable holding this type may not be initialized at this point.
    pub possibly_undefined: bool,
    /// This union originated from a docblock annotation rather than inference.
    pub from_docblock: bool,
}

impl Union {
    // --- Constructors -------------------------------------------------------

    pub fn empty() -> Self {
        Self {
            types: SmallVec::new(),
            possibly_undefined: false,
            from_docblock: false,
        }
    }

    pub fn single(atomic: Atomic) -> Self {
        let mut types = SmallVec::new();
        types.push(atomic);
        Self {
            types,
            possibly_undefined: false,
            from_docblock: false,
        }
    }

    pub fn mixed() -> Self {
        Self::single(Atomic::TMixed)
    }

    pub fn void() -> Self {
        Self::single(Atomic::TVoid)
    }

    pub fn never() -> Self {
        Self::single(Atomic::TNever)
    }

    pub fn null() -> Self {
        Self::single(Atomic::TNull)
    }

    pub fn bool() -> Self {
        Self::single(Atomic::TBool)
    }

    pub fn int() -> Self {
        Self::single(Atomic::TInt)
    }

    pub fn float() -> Self {
        Self::single(Atomic::TFloat)
    }

    pub fn string() -> Self {
        Self::single(Atomic::TString)
    }

    /// `T|null`
    pub fn nullable(atomic: Atomic) -> Self {
        let mut types = SmallVec::new();
        types.push(atomic);
        types.push(Atomic::TNull);
        Self {
            types,
            possibly_undefined: false,
            from_docblock: false,
        }
    }

    /// Build a union from multiple atomics, de-duplicating on the fly.
    pub fn from_vec(atomics: Vec<Atomic>) -> Self {
        let mut u = Self::empty();
        for a in atomics {
            u.add_type(a);
        }
        u
    }

    // --- Introspection -------------------------------------------------------

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn is_single(&self) -> bool {
        self.types.len() == 1
    }

    pub fn is_nullable(&self) -> bool {
        self.types.iter().any(|t| matches!(t, Atomic::TNull))
    }

    pub fn is_mixed(&self) -> bool {
        self.types.iter().any(|t| matches!(t, Atomic::TMixed))
    }

    pub fn is_never(&self) -> bool {
        self.types.iter().all(|t| matches!(t, Atomic::TNever)) && !self.types.is_empty()
    }

    pub fn is_void(&self) -> bool {
        self.is_single() && matches!(self.types[0], Atomic::TVoid)
    }

    pub fn can_be_falsy(&self) -> bool {
        self.types.iter().any(|t| t.can_be_falsy())
    }

    pub fn can_be_truthy(&self) -> bool {
        self.types.iter().any(|t| t.can_be_truthy())
    }

    pub fn contains<F: Fn(&Atomic) -> bool>(&self, f: F) -> bool {
        self.types.iter().any(f)
    }

    pub fn has_named_object(&self, fqcn: &str) -> bool {
        self.types.iter().any(|t| match t {
            Atomic::TNamedObject { fqcn: f, .. } => f.as_ref() == fqcn,
            _ => false,
        })
    }

    // --- Mutation ------------------------------------------------------------

    /// Add an atomic to this union, skipping duplicates.
    /// Subsumption rules: anything ⊆ TMixed; TLiteralInt ⊆ TInt; etc.
    pub fn add_type(&mut self, atomic: Atomic) {
        // If we already have TMixed, nothing to add.
        if self.types.iter().any(|t| matches!(t, Atomic::TMixed)) {
            return;
        }

        // Adding TMixed subsumes everything.
        if matches!(atomic, Atomic::TMixed) {
            self.types.clear();
            self.types.push(Atomic::TMixed);
            return;
        }

        // Avoid exact duplicates.
        if self.types.contains(&atomic) {
            return;
        }

        // TLiteralInt(n) is subsumed by TInt.
        if let Atomic::TLiteralInt(_) = &atomic {
            if self.types.iter().any(|t| matches!(t, Atomic::TInt)) {
                return;
            }
        }
        // TLiteralString(s) is subsumed by TString.
        if let Atomic::TLiteralString(_) = &atomic {
            if self.types.iter().any(|t| matches!(t, Atomic::TString)) {
                return;
            }
        }
        // TTrue / TFalse are subsumed by TBool.
        if matches!(atomic, Atomic::TTrue | Atomic::TFalse)
            && self.types.iter().any(|t| matches!(t, Atomic::TBool))
        {
            return;
        }
        // Adding TInt widens away all TLiteralInt variants.
        if matches!(atomic, Atomic::TInt) {
            self.types.retain(|t| !matches!(t, Atomic::TLiteralInt(_)));
        }
        // Adding TString widens away all TLiteralString variants.
        if matches!(atomic, Atomic::TString) {
            self.types
                .retain(|t| !matches!(t, Atomic::TLiteralString(_)));
        }
        // Adding TBool widens away TTrue/TFalse.
        if matches!(atomic, Atomic::TBool) {
            self.types
                .retain(|t| !matches!(t, Atomic::TTrue | Atomic::TFalse));
        }

        self.types.push(atomic);
    }

    // --- Narrowing -----------------------------------------------------------

    /// Remove `null` from the union (e.g. after a null check).
    pub fn remove_null(&self) -> Union {
        self.filter(|t| !matches!(t, Atomic::TNull))
    }

    /// Remove `false` from the union.
    pub fn remove_false(&self) -> Union {
        self.filter(|t| !matches!(t, Atomic::TFalse | Atomic::TBool))
    }

    /// Keep only truthy atomics (e.g. after `if ($x)`).
    pub fn narrow_to_truthy(&self) -> Union {
        if self.is_mixed() {
            return Union::mixed();
        }
        let narrowed = self.filter(|t| t.can_be_truthy());
        // Remove specific falsy literals from string/int
        narrowed.filter(|t| match t {
            Atomic::TLiteralInt(0) => false,
            Atomic::TLiteralString(s) if s.as_ref() == "" || s.as_ref() == "0" => false,
            Atomic::TLiteralFloat(0, 0) => false,
            _ => true,
        })
    }

    /// Keep only falsy atomics (e.g. after `if (!$x)`).
    pub fn narrow_to_falsy(&self) -> Union {
        if self.is_mixed() {
            return Union::from_vec(vec![
                Atomic::TNull,
                Atomic::TFalse,
                Atomic::TLiteralInt(0),
                Atomic::TLiteralString("".into()),
            ]);
        }
        self.filter(|t| t.can_be_falsy())
    }

    /// Narrow this type as if `$x instanceof ClassName` is true.
    ///
    /// The instanceof check guarantees the value IS an instance of `class`, so we
    /// replace any object / mixed constituents with the specific named object.  Scalar
    /// constituents are dropped (they can never satisfy instanceof).
    pub fn narrow_instanceof(&self, class: &str) -> Union {
        let narrowed_ty = Atomic::TNamedObject {
            fqcn: class.into(),
            type_params: vec![],
        };
        // If any constituent is an object-like type, the result is the specific class.
        let has_object = self.types.iter().any(|t| {
            matches!(
                t,
                Atomic::TObject | Atomic::TNamedObject { .. } | Atomic::TMixed | Atomic::TNull // null fails instanceof, but mixed/object may include null
            )
        });
        if has_object || self.is_empty() {
            Union::single(narrowed_ty)
        } else {
            // Pure scalars — instanceof is always false here, but return the class
            // defensively so callers don't see an empty union.
            Union::single(narrowed_ty)
        }
    }

    /// Narrow as if `is_string($x)` is true.
    pub fn narrow_to_string(&self) -> Union {
        self.filter(|t| t.is_string() || matches!(t, Atomic::TMixed | Atomic::TScalar))
    }

    /// Narrow as if `is_int($x)` is true.
    pub fn narrow_to_int(&self) -> Union {
        self.filter(|t| {
            t.is_int() || matches!(t, Atomic::TMixed | Atomic::TScalar | Atomic::TNumeric)
        })
    }

    /// Narrow as if `is_float($x)` is true.
    pub fn narrow_to_float(&self) -> Union {
        self.filter(|t| {
            matches!(
                t,
                Atomic::TFloat
                    | Atomic::TLiteralFloat(..)
                    | Atomic::TMixed
                    | Atomic::TScalar
                    | Atomic::TNumeric
            )
        })
    }

    /// Narrow as if `is_bool($x)` is true.
    pub fn narrow_to_bool(&self) -> Union {
        self.filter(|t| {
            matches!(
                t,
                Atomic::TBool | Atomic::TTrue | Atomic::TFalse | Atomic::TMixed | Atomic::TScalar
            )
        })
    }

    /// Narrow as if `is_null($x)` is true.
    pub fn narrow_to_null(&self) -> Union {
        self.filter(|t| matches!(t, Atomic::TNull | Atomic::TMixed))
    }

    /// Narrow as if `is_array($x)` is true.
    pub fn narrow_to_array(&self) -> Union {
        self.filter(|t| t.is_array() || matches!(t, Atomic::TMixed))
    }

    /// Narrow as if `is_object($x)` is true.
    pub fn narrow_to_object(&self) -> Union {
        self.filter(|t| t.is_object() || matches!(t, Atomic::TMixed))
    }

    /// Narrow as if `is_callable($x)` is true.
    pub fn narrow_to_callable(&self) -> Union {
        self.filter(|t| t.is_callable() || matches!(t, Atomic::TMixed))
    }

    // --- Merge (branch join) ------------------------------------------------

    /// Merge two unions at a branch join point (e.g. after if/else).
    /// The result is the union of all types in both.
    pub fn merge(a: &Union, b: &Union) -> Union {
        let mut result = a.clone();
        for atomic in &b.types {
            result.add_type(atomic.clone());
        }
        result.possibly_undefined = a.possibly_undefined || b.possibly_undefined;
        result
    }

    /// Intersect with another union: keep only types present in `other`, widening
    /// where `self` contains `mixed` (which is compatible with everything).
    /// Used for match-arm subject narrowing.
    pub fn intersect_with(&self, other: &Union) -> Union {
        if self.is_mixed() {
            return other.clone();
        }
        if other.is_mixed() {
            return self.clone();
        }
        // Keep atomics from self that are also in other (by equality or subtype)
        let mut result = Union::empty();
        for a in &self.types {
            for b in &other.types {
                if a == b || atomic_subtype(a, b) || atomic_subtype(b, a) {
                    result.add_type(a.clone());
                    break;
                }
            }
        }
        // If nothing matched, fall back to other (conservative)
        if result.is_empty() {
            other.clone()
        } else {
            result
        }
    }

    // --- Template substitution ----------------------------------------------

    /// Replace template param references with their resolved types.
    pub fn substitute_templates(
        &self,
        bindings: &std::collections::HashMap<Arc<str>, Union>,
    ) -> Union {
        if bindings.is_empty() {
            return self.clone();
        }
        let mut result = Union::empty();
        result.possibly_undefined = self.possibly_undefined;
        result.from_docblock = self.from_docblock;
        for atomic in &self.types {
            match atomic {
                Atomic::TTemplateParam { name, .. } => {
                    if let Some(resolved) = bindings.get(name) {
                        for t in &resolved.types {
                            result.add_type(t.clone());
                        }
                    } else {
                        result.add_type(atomic.clone());
                    }
                }
                Atomic::TArray { key, value } => {
                    result.add_type(Atomic::TArray {
                        key: Box::new(key.substitute_templates(bindings)),
                        value: Box::new(value.substitute_templates(bindings)),
                    });
                }
                Atomic::TList { value } => {
                    result.add_type(Atomic::TList {
                        value: Box::new(value.substitute_templates(bindings)),
                    });
                }
                Atomic::TNonEmptyArray { key, value } => {
                    result.add_type(Atomic::TNonEmptyArray {
                        key: Box::new(key.substitute_templates(bindings)),
                        value: Box::new(value.substitute_templates(bindings)),
                    });
                }
                Atomic::TNonEmptyList { value } => {
                    result.add_type(Atomic::TNonEmptyList {
                        value: Box::new(value.substitute_templates(bindings)),
                    });
                }
                Atomic::TKeyedArray {
                    properties,
                    is_open,
                    is_list,
                } => {
                    use crate::atomic::KeyedProperty;
                    let new_props = properties
                        .iter()
                        .map(|(k, prop)| {
                            (
                                k.clone(),
                                KeyedProperty {
                                    ty: prop.ty.substitute_templates(bindings),
                                    optional: prop.optional,
                                },
                            )
                        })
                        .collect();
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                }
                Atomic::TCallable {
                    params,
                    return_type,
                } => {
                    result.add_type(Atomic::TCallable {
                        params: params.as_ref().map(|ps| {
                            ps.iter()
                                .map(|p| substitute_in_fn_param(p, bindings))
                                .collect()
                        }),
                        return_type: return_type
                            .as_ref()
                            .map(|r| Box::new(r.substitute_templates(bindings))),
                    });
                }
                Atomic::TClosure {
                    params,
                    return_type,
                    this_type,
                } => {
                    result.add_type(Atomic::TClosure {
                        params: params
                            .iter()
                            .map(|p| substitute_in_fn_param(p, bindings))
                            .collect(),
                        return_type: Box::new(return_type.substitute_templates(bindings)),
                        this_type: this_type
                            .as_ref()
                            .map(|t| Box::new(t.substitute_templates(bindings))),
                    });
                }
                Atomic::TConditional {
                    subject,
                    if_true,
                    if_false,
                } => {
                    result.add_type(Atomic::TConditional {
                        subject: Box::new(subject.substitute_templates(bindings)),
                        if_true: Box::new(if_true.substitute_templates(bindings)),
                        if_false: Box::new(if_false.substitute_templates(bindings)),
                    });
                }
                Atomic::TIntersection { parts } => {
                    result.add_type(Atomic::TIntersection {
                        parts: parts
                            .iter()
                            .map(|p| p.substitute_templates(bindings))
                            .collect(),
                    });
                }
                Atomic::TNamedObject { fqcn, type_params } => {
                    // TODO: the docblock parser emits TNamedObject { fqcn: "T" } for bare @return T
                    // annotations instead of TTemplateParam, because it lacks template context at
                    // parse time. This block works around that by treating bare unqualified names
                    // as template param references when they appear in the binding map. Proper fix:
                    // make the docblock parser template-aware so it emits TTemplateParam directly.
                    // See issue #26 for context.
                    if type_params.is_empty() && !fqcn.contains('\\') {
                        if let Some(resolved) = bindings.get(fqcn.as_ref()) {
                            for t in &resolved.types {
                                result.add_type(t.clone());
                            }
                            continue;
                        }
                    }
                    let new_params = type_params
                        .iter()
                        .map(|p| p.substitute_templates(bindings))
                        .collect();
                    result.add_type(Atomic::TNamedObject {
                        fqcn: fqcn.clone(),
                        type_params: new_params,
                    });
                }
                _ => {
                    result.add_type(atomic.clone());
                }
            }
        }
        result
    }

    // --- Subtype check -------------------------------------------------------

    /// Returns true if every atomic in `self` is a subtype of some atomic in `other`.
    /// Does not require a Codebase (no inheritance check); use the codebase-aware
    /// version in mir-analyzer for full checks.
    pub fn is_subtype_of_simple(&self, other: &Union) -> bool {
        if other.is_mixed() {
            return true;
        }
        if self.is_never() {
            return true; // never <: everything
        }
        self.types
            .iter()
            .all(|a| other.types.iter().any(|b| atomic_subtype(a, b)))
    }

    // --- Utilities ----------------------------------------------------------

    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Union {
        let mut result = Union::empty();
        result.possibly_undefined = self.possibly_undefined;
        result.from_docblock = self.from_docblock;
        for atomic in &self.types {
            if f(atomic) {
                result.types.push(atomic.clone());
            }
        }
        result
    }

    /// Mark this union as possibly-undefined and return it.
    pub fn possibly_undefined(mut self) -> Self {
        self.possibly_undefined = true;
        self
    }

    /// Mark this union as coming from a docblock annotation.
    pub fn from_docblock(mut self) -> Self {
        self.from_docblock = true;
        self
    }
}

// ---------------------------------------------------------------------------
// Template substitution helpers
// ---------------------------------------------------------------------------

fn substitute_in_fn_param(
    p: &crate::atomic::FnParam,
    bindings: &std::collections::HashMap<Arc<str>, Union>,
) -> crate::atomic::FnParam {
    crate::atomic::FnParam {
        name: p.name.clone(),
        ty: p.ty.as_ref().map(|t| t.substitute_templates(bindings)),
        default: p.default.as_ref().map(|d| d.substitute_templates(bindings)),
        is_variadic: p.is_variadic,
        is_byref: p.is_byref,
        is_optional: p.is_optional,
    }
}

// ---------------------------------------------------------------------------
// Atomic subtype (no codebase — structural check only)
// ---------------------------------------------------------------------------

fn atomic_subtype(sub: &Atomic, sup: &Atomic) -> bool {
    if sub == sup {
        return true;
    }
    match (sub, sup) {
        // Bottom type
        (Atomic::TNever, _) => true,
        // Top types — anything goes in both directions for mixed
        (_, Atomic::TMixed) => true,
        (Atomic::TMixed, _) => true,

        // Scalars
        (Atomic::TLiteralInt(_), Atomic::TInt) => true,
        (Atomic::TLiteralInt(_), Atomic::TNumeric) => true,
        (Atomic::TLiteralInt(_), Atomic::TScalar) => true,
        (Atomic::TLiteralInt(n), Atomic::TPositiveInt) => *n > 0,
        (Atomic::TLiteralInt(n), Atomic::TNonNegativeInt) => *n >= 0,
        (Atomic::TLiteralInt(n), Atomic::TNegativeInt) => *n < 0,
        (Atomic::TPositiveInt, Atomic::TInt) => true,
        (Atomic::TPositiveInt, Atomic::TNonNegativeInt) => true,
        (Atomic::TNegativeInt, Atomic::TInt) => true,
        (Atomic::TNonNegativeInt, Atomic::TInt) => true,
        (Atomic::TIntRange { .. }, Atomic::TInt) => true,

        (Atomic::TLiteralFloat(..), Atomic::TFloat) => true,
        (Atomic::TLiteralFloat(..), Atomic::TNumeric) => true,
        (Atomic::TLiteralFloat(..), Atomic::TScalar) => true,

        (Atomic::TLiteralString(s), Atomic::TString) => {
            let _ = s;
            true
        }
        (Atomic::TLiteralString(s), Atomic::TNonEmptyString) => !s.is_empty(),
        (Atomic::TLiteralString(_), Atomic::TScalar) => true,
        (Atomic::TNonEmptyString, Atomic::TString) => true,
        (Atomic::TNumericString, Atomic::TString) => true,
        (Atomic::TClassString(_), Atomic::TString) => true,
        (Atomic::TInterfaceString, Atomic::TString) => true,
        (Atomic::TEnumString, Atomic::TString) => true,
        (Atomic::TTraitString, Atomic::TString) => true,

        (Atomic::TTrue, Atomic::TBool) => true,
        (Atomic::TFalse, Atomic::TBool) => true,

        (Atomic::TInt, Atomic::TNumeric) => true,
        (Atomic::TFloat, Atomic::TNumeric) => true,
        (Atomic::TNumericString, Atomic::TNumeric) => true,

        (Atomic::TInt, Atomic::TScalar) => true,
        (Atomic::TFloat, Atomic::TScalar) => true,
        (Atomic::TString, Atomic::TScalar) => true,
        (Atomic::TBool, Atomic::TScalar) => true,
        (Atomic::TNumeric, Atomic::TScalar) => true,
        (Atomic::TTrue, Atomic::TScalar) => true,
        (Atomic::TFalse, Atomic::TScalar) => true,

        // Object hierarchy (structural, no codebase)
        (Atomic::TNamedObject { .. }, Atomic::TObject) => true,
        (Atomic::TStaticObject { .. }, Atomic::TObject) => true,
        (Atomic::TSelf { .. }, Atomic::TObject) => true,
        // self(X) and static(X) satisfy TNamedObject(X) with same FQCN
        (Atomic::TSelf { fqcn: a }, Atomic::TNamedObject { fqcn: b, .. }) => a == b,
        (Atomic::TStaticObject { fqcn: a }, Atomic::TNamedObject { fqcn: b, .. }) => a == b,
        // TNamedObject(X) satisfies self(X) / static(X) with same FQCN
        (Atomic::TNamedObject { fqcn: a, .. }, Atomic::TSelf { fqcn: b }) => a == b,
        (Atomic::TNamedObject { fqcn: a, .. }, Atomic::TStaticObject { fqcn: b }) => a == b,

        // Literal int widens to float in PHP
        (Atomic::TLiteralInt(_), Atomic::TFloat) => true,
        (Atomic::TPositiveInt, Atomic::TFloat) => true,
        (Atomic::TInt, Atomic::TFloat) => true,

        // Literal int satisfies int ranges
        (Atomic::TLiteralInt(_), Atomic::TIntRange { .. }) => true,

        // PHP callables: string and array are valid callable values
        (Atomic::TString, Atomic::TCallable { .. }) => true,
        (Atomic::TNonEmptyString, Atomic::TCallable { .. }) => true,
        (Atomic::TLiteralString(_), Atomic::TCallable { .. }) => true,
        (Atomic::TArray { .. }, Atomic::TCallable { .. }) => true,
        (Atomic::TNonEmptyArray { .. }, Atomic::TCallable { .. }) => true,

        // Closure <: callable, typed Closure <: Closure
        (Atomic::TClosure { .. }, Atomic::TCallable { .. }) => true,
        // callable <: Closure: callable is wider but not flagged at default error level
        (Atomic::TCallable { .. }, Atomic::TClosure { .. }) => true,
        // Any TClosure satisfies another TClosure (structural compatibility simplified)
        (Atomic::TClosure { .. }, Atomic::TClosure { .. }) => true,
        // callable <: callable (trivial)
        (Atomic::TCallable { .. }, Atomic::TCallable { .. }) => true,
        // TClosure satisfies `Closure` named object or `object`
        (Atomic::TClosure { .. }, Atomic::TNamedObject { fqcn, .. }) => {
            fqcn.as_ref().eq_ignore_ascii_case("closure")
        }
        (Atomic::TClosure { .. }, Atomic::TObject) => true,

        // List <: array
        (Atomic::TList { value }, Atomic::TArray { key, value: av }) => {
            // list key is always int
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && value.is_subtype_of_simple(av)
        }
        (Atomic::TNonEmptyList { value }, Atomic::TList { value: lv }) => {
            value.is_subtype_of_simple(lv)
        }
        // array<int, X> is accepted where list<X> or non-empty-list<X> expected
        (Atomic::TArray { key, value: av }, Atomic::TList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_of_simple(lv)
        }
        (Atomic::TArray { key, value: av }, Atomic::TNonEmptyList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_of_simple(lv)
        }
        (Atomic::TNonEmptyArray { key, value: av }, Atomic::TList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_of_simple(lv)
        }
        (Atomic::TNonEmptyArray { key, value: av }, Atomic::TNonEmptyList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_of_simple(lv)
        }
        // TList <: TList value covariance
        (Atomic::TList { value: v1 }, Atomic::TList { value: v2 }) => v1.is_subtype_of_simple(v2),
        (Atomic::TNonEmptyArray { key: k1, value: v1 }, Atomic::TArray { key: k2, value: v2 }) => {
            k1.is_subtype_of_simple(k2) && v1.is_subtype_of_simple(v2)
        }

        // array<A, B> <: array<C, D>  iff  A <: C && B <: D
        (Atomic::TArray { key: k1, value: v1 }, Atomic::TArray { key: k2, value: v2 }) => {
            k1.is_subtype_of_simple(k2) && v1.is_subtype_of_simple(v2)
        }

        // A keyed/shape array (array{...} or array{}) is a subtype of any generic array.
        (Atomic::TKeyedArray { .. }, Atomic::TArray { .. }) => true,

        // A list-shaped keyed array (is_list=true, all int keys) is a subtype of list<X>.
        (
            Atomic::TKeyedArray {
                properties,
                is_list,
                ..
            },
            Atomic::TList { value: lv },
        ) => *is_list && properties.values().all(|p| p.ty.is_subtype_of_simple(lv)),
        (
            Atomic::TKeyedArray {
                properties,
                is_list,
                ..
            },
            Atomic::TNonEmptyList { value: lv },
        ) => {
            *is_list
                && !properties.is_empty()
                && properties.values().all(|p| p.ty.is_subtype_of_simple(lv))
        }

        // A template parameter T acts as a wildcard — any type satisfies it.
        (_, Atomic::TTemplateParam { .. }) => true,

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_is_single() {
        let u = Union::single(Atomic::TString);
        assert!(u.is_single());
        assert!(!u.is_nullable());
    }

    #[test]
    fn nullable_has_null() {
        let u = Union::nullable(Atomic::TString);
        assert!(u.is_nullable());
        assert_eq!(u.types.len(), 2);
    }

    #[test]
    fn add_type_deduplicates() {
        let mut u = Union::single(Atomic::TString);
        u.add_type(Atomic::TString);
        assert_eq!(u.types.len(), 1);
    }

    #[test]
    fn add_type_literal_subsumed_by_base() {
        let mut u = Union::single(Atomic::TInt);
        u.add_type(Atomic::TLiteralInt(42));
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TInt));
    }

    #[test]
    fn add_type_base_widens_literals() {
        let mut u = Union::single(Atomic::TLiteralInt(1));
        u.add_type(Atomic::TLiteralInt(2));
        u.add_type(Atomic::TInt);
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TInt));
    }

    #[test]
    fn mixed_subsumes_everything() {
        let mut u = Union::single(Atomic::TString);
        u.add_type(Atomic::TMixed);
        assert_eq!(u.types.len(), 1);
        assert!(u.is_mixed());
    }

    #[test]
    fn remove_null() {
        let u = Union::nullable(Atomic::TString);
        let narrowed = u.remove_null();
        assert!(!narrowed.is_nullable());
        assert_eq!(narrowed.types.len(), 1);
    }

    #[test]
    fn narrow_to_truthy_removes_null_false() {
        let mut u = Union::empty();
        u.add_type(Atomic::TString);
        u.add_type(Atomic::TNull);
        u.add_type(Atomic::TFalse);
        let truthy = u.narrow_to_truthy();
        assert!(!truthy.is_nullable());
        assert!(!truthy.contains(|t| matches!(t, Atomic::TFalse)));
    }

    #[test]
    fn merge_combines_types() {
        let a = Union::single(Atomic::TString);
        let b = Union::single(Atomic::TInt);
        let merged = Union::merge(&a, &b);
        assert_eq!(merged.types.len(), 2);
    }

    #[test]
    fn subtype_literal_int_under_int() {
        let sub = Union::single(Atomic::TLiteralInt(5));
        let sup = Union::single(Atomic::TInt);
        assert!(sub.is_subtype_of_simple(&sup));
    }

    #[test]
    fn subtype_never_is_bottom() {
        let never = Union::never();
        let string = Union::single(Atomic::TString);
        assert!(never.is_subtype_of_simple(&string));
    }

    #[test]
    fn subtype_everything_under_mixed() {
        let string = Union::single(Atomic::TString);
        let mixed = Union::mixed();
        assert!(string.is_subtype_of_simple(&mixed));
    }

    #[test]
    fn template_substitution() {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert(Arc::from("T"), Union::single(Atomic::TString));

        let tmpl = Union::single(Atomic::TTemplateParam {
            name: Arc::from("T"),
            as_type: Box::new(Union::mixed()),
            defining_entity: Arc::from("MyClass"),
        });

        let resolved = tmpl.substitute_templates(&bindings);
        assert_eq!(resolved.types.len(), 1);
        assert!(matches!(resolved.types[0], Atomic::TString));
    }

    #[test]
    fn intersection_is_object() {
        let parts = vec![
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("Iterator"),
                type_params: vec![],
            }),
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("Countable"),
                type_params: vec![],
            }),
        ];
        let atomic = Atomic::TIntersection { parts };
        assert!(atomic.is_object());
        assert!(!atomic.can_be_falsy());
        assert!(atomic.can_be_truthy());
    }

    #[test]
    fn intersection_display_two_parts() {
        let parts = vec![
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("Iterator"),
                type_params: vec![],
            }),
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("Countable"),
                type_params: vec![],
            }),
        ];
        let u = Union::single(Atomic::TIntersection { parts });
        assert_eq!(format!("{u}"), "Iterator&Countable");
    }

    #[test]
    fn intersection_display_three_parts() {
        let parts = vec![
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("A"),
                type_params: vec![],
            }),
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("B"),
                type_params: vec![],
            }),
            Union::single(Atomic::TNamedObject {
                fqcn: Arc::from("C"),
                type_params: vec![],
            }),
        ];
        let u = Union::single(Atomic::TIntersection { parts });
        assert_eq!(format!("{u}"), "A&B&C");
    }

    #[test]
    fn intersection_in_nullable_union_display() {
        let intersection = Atomic::TIntersection {
            parts: vec![
                Union::single(Atomic::TNamedObject {
                    fqcn: Arc::from("Iterator"),
                    type_params: vec![],
                }),
                Union::single(Atomic::TNamedObject {
                    fqcn: Arc::from("Countable"),
                    type_params: vec![],
                }),
            ],
        };
        let mut u = Union::single(intersection);
        u.add_type(Atomic::TNull);
        assert!(u.is_nullable());
        assert!(u.contains(|t| matches!(t, Atomic::TIntersection { .. })));
    }

    // --- substitute_templates coverage for previously-missing arms ----------

    fn t_param(name: &str) -> Union {
        Union::single(Atomic::TTemplateParam {
            name: Arc::from(name),
            as_type: Box::new(Union::mixed()),
            defining_entity: Arc::from("Fn"),
        })
    }

    fn bindings_t_string() -> std::collections::HashMap<Arc<str>, Union> {
        let mut b = std::collections::HashMap::new();
        b.insert(Arc::from("T"), Union::single(Atomic::TString));
        b
    }

    #[test]
    fn substitute_non_empty_array_key_and_value() {
        let ty = Union::single(Atomic::TNonEmptyArray {
            key: Box::new(t_param("T")),
            value: Box::new(t_param("T")),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        assert_eq!(result.types.len(), 1);
        let Atomic::TNonEmptyArray { key, value } = &result.types[0] else {
            panic!("expected TNonEmptyArray");
        };
        assert!(matches!(key.types[0], Atomic::TString));
        assert!(matches!(value.types[0], Atomic::TString));
    }

    #[test]
    fn substitute_non_empty_list_value() {
        let ty = Union::single(Atomic::TNonEmptyList {
            value: Box::new(t_param("T")),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TNonEmptyList { value } = &result.types[0] else {
            panic!("expected TNonEmptyList");
        };
        assert!(matches!(value.types[0], Atomic::TString));
    }

    #[test]
    fn substitute_keyed_array_property_types() {
        use crate::atomic::{ArrayKey, KeyedProperty};
        use indexmap::IndexMap;
        let mut props = IndexMap::new();
        props.insert(
            ArrayKey::String(Arc::from("name")),
            KeyedProperty {
                ty: t_param("T"),
                optional: false,
            },
        );
        props.insert(
            ArrayKey::String(Arc::from("tag")),
            KeyedProperty {
                ty: t_param("T"),
                optional: true,
            },
        );
        let ty = Union::single(Atomic::TKeyedArray {
            properties: props,
            is_open: true,
            is_list: false,
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } = &result.types[0]
        else {
            panic!("expected TKeyedArray");
        };
        assert!(is_open);
        assert!(!is_list);
        assert!(matches!(
            properties[&ArrayKey::String(Arc::from("name"))].ty.types[0],
            Atomic::TString
        ));
        assert!(properties[&ArrayKey::String(Arc::from("tag"))].optional);
        assert!(matches!(
            properties[&ArrayKey::String(Arc::from("tag"))].ty.types[0],
            Atomic::TString
        ));
    }

    #[test]
    fn substitute_callable_params_and_return() {
        use crate::atomic::FnParam;
        let ty = Union::single(Atomic::TCallable {
            params: Some(vec![FnParam {
                name: Arc::from("x"),
                ty: Some(t_param("T")),
                default: None,
                is_variadic: false,
                is_byref: false,
                is_optional: false,
            }]),
            return_type: Some(Box::new(t_param("T"))),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TCallable {
            params,
            return_type,
        } = &result.types[0]
        else {
            panic!("expected TCallable");
        };
        let param_ty = params.as_ref().unwrap()[0].ty.as_ref().unwrap();
        assert!(matches!(param_ty.types[0], Atomic::TString));
        let ret = return_type.as_ref().unwrap();
        assert!(matches!(ret.types[0], Atomic::TString));
    }

    #[test]
    fn substitute_callable_bare_no_panic() {
        // callable with no params/return — must not panic and must pass through unchanged
        let ty = Union::single(Atomic::TCallable {
            params: None,
            return_type: None,
        });
        let result = ty.substitute_templates(&bindings_t_string());
        assert!(matches!(
            result.types[0],
            Atomic::TCallable {
                params: None,
                return_type: None
            }
        ));
    }

    #[test]
    fn substitute_closure_params_return_and_this() {
        use crate::atomic::FnParam;
        let ty = Union::single(Atomic::TClosure {
            params: vec![FnParam {
                name: Arc::from("a"),
                ty: Some(t_param("T")),
                default: Some(t_param("T")),
                is_variadic: true,
                is_byref: true,
                is_optional: true,
            }],
            return_type: Box::new(t_param("T")),
            this_type: Some(Box::new(t_param("T"))),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TClosure {
            params,
            return_type,
            this_type,
        } = &result.types[0]
        else {
            panic!("expected TClosure");
        };
        let p = &params[0];
        assert!(matches!(p.ty.as_ref().unwrap().types[0], Atomic::TString));
        assert!(matches!(
            p.default.as_ref().unwrap().types[0],
            Atomic::TString
        ));
        // flags preserved
        assert!(p.is_variadic);
        assert!(p.is_byref);
        assert!(p.is_optional);
        assert!(matches!(return_type.types[0], Atomic::TString));
        assert!(matches!(
            this_type.as_ref().unwrap().types[0],
            Atomic::TString
        ));
    }

    #[test]
    fn substitute_conditional_all_branches() {
        let ty = Union::single(Atomic::TConditional {
            subject: Box::new(t_param("T")),
            if_true: Box::new(t_param("T")),
            if_false: Box::new(Union::single(Atomic::TInt)),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TConditional {
            subject,
            if_true,
            if_false,
        } = &result.types[0]
        else {
            panic!("expected TConditional");
        };
        assert!(matches!(subject.types[0], Atomic::TString));
        assert!(matches!(if_true.types[0], Atomic::TString));
        assert!(matches!(if_false.types[0], Atomic::TInt));
    }

    #[test]
    fn substitute_intersection_parts() {
        let ty = Union::single(Atomic::TIntersection {
            parts: vec![
                Union::single(Atomic::TNamedObject {
                    fqcn: Arc::from("Countable"),
                    type_params: vec![],
                }),
                t_param("T"),
            ],
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TIntersection { parts } = &result.types[0] else {
            panic!("expected TIntersection");
        };
        assert_eq!(parts.len(), 2);
        assert!(matches!(parts[0].types[0], Atomic::TNamedObject { .. }));
        assert!(matches!(parts[1].types[0], Atomic::TString));
    }

    #[test]
    fn substitute_no_template_params_identity() {
        let ty = Union::single(Atomic::TInt);
        let result = ty.substitute_templates(&bindings_t_string());
        assert!(matches!(result.types[0], Atomic::TInt));
    }
}
