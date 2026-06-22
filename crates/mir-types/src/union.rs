use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::{Arc, OnceLock};

use crate::atomic::Atomic;
use crate::symbol::Name;

/// Returns a cached empty `Arc<[Type]>` for `type_params` / `parts` fields.
/// Re-uses a single Arc allocation so all empty parameter lists share one
/// control block instead of allocating one per TNamedObject construction.
pub fn empty_type_params() -> Arc<[Type]> {
    static EMPTY: OnceLock<Arc<[Type]>> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::from([] as [Type; 0])).clone()
}

/// Convert a `Vec<Type>` to `Arc<[Type]>`, using the cached empty Arc when
/// the vec is empty to avoid an allocation for the common no-generic case.
pub fn vec_to_type_params(v: Vec<Type>) -> Arc<[Type]> {
    if v.is_empty() {
        empty_type_params()
    } else {
        Arc::from(v)
    }
}

// Most unions contain 1-2 atomics (e.g. `string|null`), so we inline two.
pub type AtomicVec = SmallVec<[Atomic; 2]>;

/// Result of classifying a type for `clone` validity (see [`Type::clone_validity`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneValidity {
    /// Every member is (or may be) an object — cloning is fine.
    Cloneable,
    /// Every member is definitely a non-object — cloning is an error.
    Invalid,
    /// Some members are non-objects, some are objects — cloning may be an error.
    PossiblyInvalid,
    /// Empty/unknown type — no diagnostic.
    Unknown,
}

// ---------------------------------------------------------------------------
// Type — the primary type carrier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Type {
    pub types: AtomicVec,
    /// The variable holding this type may not be initialized at this point.
    pub possibly_undefined: bool,
    /// This type originated from a docblock annotation rather than inference.
    pub from_docblock: bool,
}

impl Type {
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
        // `mixed|null` = `mixed` — null is already included in mixed.
        if matches!(atomic, Atomic::TMixed) {
            return Self::mixed();
        }
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
        self.types.iter().any(|t| match t {
            Atomic::TMixed => true,
            Atomic::TTemplateParam { as_type, .. } => as_type.is_mixed(),
            _ => false,
        })
    }

    pub fn is_never(&self) -> bool {
        self.types.iter().all(|t| matches!(t, Atomic::TNever)) && !self.types.is_empty()
    }

    /// Classify this type for `clone` validity. Recurses into template-param
    /// bounds (like [`Type::is_mixed`]). Callers handle `mixed` separately.
    pub fn clone_validity(&self) -> CloneValidity {
        if self.types.is_empty() {
            return CloneValidity::Unknown;
        }
        let mut has_non_object = false;
        let mut has_other = false; // object or ambiguous (callable, mixed, conditional, …)
        for t in &self.types {
            match t {
                Atomic::TTemplateParam { as_type, .. } => match as_type.clone_validity() {
                    CloneValidity::Invalid => has_non_object = true,
                    CloneValidity::PossiblyInvalid => {
                        has_non_object = true;
                        has_other = true;
                    }
                    CloneValidity::Cloneable | CloneValidity::Unknown => has_other = true,
                },
                other if other.is_definitely_non_object() => has_non_object = true,
                _ => has_other = true,
            }
        }
        match (has_non_object, has_other) {
            (true, false) => CloneValidity::Invalid,
            (true, true) => CloneValidity::PossiblyInvalid,
            _ => CloneValidity::Cloneable,
        }
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

        // Simplify trivial conditional types: (X is ? T : T) → T
        // Recursively simplify branches first so nested trivial conditionals collapse.
        let atomic = if let Atomic::TConditional {
            param_name: _,
            subject: _,
            if_true,
            if_false,
        } = &atomic
        {
            let mut simplified_true = Type::empty();
            for t in &if_true.types {
                simplified_true.add_type(t.clone());
            }
            let mut simplified_false = Type::empty();
            for t in &if_false.types {
                simplified_false.add_type(t.clone());
            }
            if simplified_true == simplified_false {
                for t in simplified_true.types {
                    self.add_type(t);
                }
                return;
            }
            atomic
        } else {
            atomic
        };

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

        // TNever is the bottom type: T | never = T.
        if matches!(atomic, Atomic::TNever) {
            if !self.types.is_empty() {
                return;
            }
        } else {
            self.types.retain(|t| !matches!(t, Atomic::TNever));
        }

        // Empty keyed array (array{}) is a subtype of any generic array or list.
        // Remove array{} if we already have a generic array<K,V> or list<V>.
        if let Atomic::TKeyedArray { properties, .. } = &atomic {
            if properties.is_empty() {
                for existing in &self.types {
                    match existing {
                        Atomic::TArray { .. }
                        | Atomic::TNonEmptyArray { .. }
                        | Atomic::TList { .. }
                        | Atomic::TNonEmptyList { .. } => {
                            return; // Don't add empty array, it's subsumed
                        }
                        _ => {}
                    }
                }
            }
        }

        // When adding a generic array or list, remove any empty keyed arrays since they're subtypes.
        let is_generic_array_or_list = matches!(
            &atomic,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
        );
        if is_generic_array_or_list {
            self.types.retain(|t| {
                if let Atomic::TKeyedArray { properties, .. } = t {
                    !properties.is_empty()
                } else {
                    true
                }
            });
        }

        self.types.push(atomic);
    }

    // --- Narrowing -----------------------------------------------------------

    /// Remove `null` from the union (e.g. after a null check).
    pub fn remove_null(&self) -> Type {
        self.filter(|t| !matches!(t, Atomic::TNull))
    }

    /// Remove `false` from the union.
    /// `TFalse` is dropped; `TBool` becomes `TTrue` since `bool - false = true`.
    pub fn remove_false(&self) -> Type {
        let mut result = self.filter(|t| !matches!(t, Atomic::TFalse | Atomic::TBool));
        if self.types.iter().any(|t| matches!(t, Atomic::TBool)) {
            result.add_type(Atomic::TTrue);
        }
        result
    }

    /// Remove both `null` and `false` from the union (core type without nullable/falsy variants).
    pub fn core_type(&self) -> Type {
        self.remove_null().remove_false()
    }

    /// Keep only truthy atomics (e.g. after `if ($x)`).
    pub fn narrow_to_truthy(&self) -> Type {
        if self.is_mixed() {
            return Type::mixed();
        }
        let mut result = Type::empty();
        result.from_docblock = self.from_docblock;
        for t in &self.types {
            match t {
                // Always-falsy — exclude entirely.
                Atomic::TLiteralInt(0)
                | Atomic::TLiteralFloat(0, 0)
                | Atomic::TNull
                | Atomic::TFalse => {}
                Atomic::TLiteralString(s) if s.as_ref() == "" || s.as_ref() == "0" => {}
                // bool contains both true (truthy) and false (falsy); truthy branch is true.
                Atomic::TBool => result.add_type(Atomic::TTrue),
                // array/list: empty ↔ falsy; truthy branch is non-empty-array/list.
                Atomic::TArray { key, value } => result.add_type(Atomic::TNonEmptyArray {
                    key: key.clone(),
                    value: value.clone(),
                }),
                Atomic::TList { value } => result.add_type(Atomic::TNonEmptyList {
                    value: value.clone(),
                }),
                // string: only "" and "0" are falsy; truthy branch is non-empty-string.
                // non-empty-string still includes "0" (which is falsy) but that is the
                // standard approximation used by Psalm and other analyzers.
                Atomic::TString => result.add_type(Atomic::TNonEmptyString),
                // numeric-string: "0" is the only falsy value; non-zero numerics are truthy.
                // No named "non-zero numeric-string" type exists; keep as-is conservatively.
                // int<0, max> only has 0 as its falsy value; truthy branch is int<1, max>.
                // (int<0, 0> is handled by the can_be_truthy() false guard below.)
                Atomic::TNonNegativeInt => result.add_type(Atomic::TPositiveInt),
                Atomic::TIntRange { min: Some(0), max } if max.is_none_or(|m| m >= 1) => {
                    let atom = if max.is_none() {
                        Atomic::TPositiveInt
                    } else {
                        Atomic::TIntRange {
                            min: Some(1),
                            max: *max,
                        }
                    };
                    result.add_type(atom);
                }
                // int<min, 0>: 0 is the only falsy value; truthy branch excludes it → int<min, -1>.
                Atomic::TIntRange { min, max: Some(0) } => {
                    let atom = match min {
                        None => Atomic::TNegativeInt,
                        Some(n) if *n <= -1 => Atomic::TIntRange {
                            min: *min,
                            max: Some(-1),
                        },
                        _ => continue, // min >= 0 with max == 0 → range is {0} — can_be_truthy() handles this
                    };
                    result.add_type(atom);
                }
                // Anything else that can never be truthy — drop.
                t if !t.can_be_truthy() => {}
                _ => result.add_type(t.clone()),
            }
        }
        result
    }

    /// Keep only falsy atomics (e.g. after `if (!$x)`).
    pub fn narrow_to_falsy(&self) -> Type {
        if self.is_mixed() {
            return Type::from_vec(vec![
                Atomic::TNull,
                Atomic::TFalse,
                Atomic::TLiteralInt(0),
                Atomic::TLiteralString("".into()),
            ]);
        }
        let mut result = Type::empty();
        result.from_docblock = self.from_docblock;
        for t in &self.types {
            match t {
                // bool: only false is falsy; falsy branch is false.
                Atomic::TBool => result.add_type(Atomic::TFalse),
                // int: only 0 is falsy.
                Atomic::TInt => result.add_type(Atomic::TLiteralInt(0)),
                // float: only 0.0 is falsy.
                Atomic::TFloat => result.add_type(Atomic::TLiteralFloat(0, 0)),
                // string: only "" and "0" are falsy.
                Atomic::TString => {
                    result.add_type(Atomic::TLiteralString("".into()));
                    result.add_type(Atomic::TLiteralString("0".into()));
                }
                // numeric-string: only "0" is a falsy numeric string.
                Atomic::TNumericString => result.add_type(Atomic::TLiteralString("0".into())),
                // non-negative-int: only 0 is falsy.
                Atomic::TNonNegativeInt => result.add_type(Atomic::TLiteralInt(0)),
                // int<0, hi>: only 0 is falsy.
                Atomic::TIntRange {
                    min: Some(0),
                    max: Some(_) | None,
                } => result.add_type(Atomic::TLiteralInt(0)),
                // int<min, 0>: only 0 is falsy.
                Atomic::TIntRange { max: Some(0), .. } => result.add_type(Atomic::TLiteralInt(0)),
                t if !t.can_be_falsy() => {} // always truthy — exclude
                _ => result.add_type(t.clone()),
            }
        }
        result
    }

    /// Narrow this type as if `$x instanceof ClassName` is true.
    ///
    /// The instanceof check guarantees the value IS an instance of `class`, so we
    /// replace any object / mixed constituents with the specific named object.  Scalar
    /// constituents are dropped (they can never satisfy instanceof).
    pub fn narrow_instanceof(&self, class: &str) -> Type {
        let narrowed_ty = Atomic::TNamedObject {
            fqcn: class.into(),
            type_params: empty_type_params(),
        };
        // If any constituent is an object-like type, the result is the specific class.
        let has_object = self.types.iter().any(|t| {
            matches!(
                t,
                Atomic::TObject | Atomic::TNamedObject { .. } | Atomic::TMixed | Atomic::TNull // null fails instanceof, but mixed/object may include null
            )
        });
        if has_object || self.is_empty() {
            Type::single(narrowed_ty)
        } else {
            // Pure scalars — instanceof is always false here, but return the class
            // defensively so callers don't see an empty union.
            Type::single(narrowed_ty)
        }
    }

    /// Narrow as if `is_string($x)` is true.
    pub fn narrow_to_string(&self) -> Type {
        self.filter(|t| t.is_string() || matches!(t, Atomic::TMixed | Atomic::TScalar))
    }

    /// Narrow as if `is_int($x)` is true.
    pub fn narrow_to_int(&self) -> Type {
        self.filter(|t| {
            t.is_int() || matches!(t, Atomic::TMixed | Atomic::TScalar | Atomic::TNumeric)
        })
    }

    /// Narrow as if `is_float($x)` is true.
    pub fn narrow_to_float(&self) -> Type {
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
    pub fn narrow_to_bool(&self) -> Type {
        self.filter(|t| {
            matches!(
                t,
                Atomic::TBool | Atomic::TTrue | Atomic::TFalse | Atomic::TMixed | Atomic::TScalar
            )
        })
    }

    /// Narrow as if `is_null($x)` is true.
    pub fn narrow_to_null(&self) -> Type {
        self.filter(|t| matches!(t, Atomic::TNull | Atomic::TMixed))
    }

    /// Narrow as if `is_array($x)` is true.
    pub fn narrow_to_array(&self) -> Type {
        self.filter(|t| t.is_array() || matches!(t, Atomic::TMixed))
    }

    /// Narrow array/list types to their non-empty variants (for `count() > 0` etc.).
    pub fn narrow_to_non_empty_collection(&self) -> Type {
        let mut out = Type::empty();
        out.from_docblock = self.from_docblock;
        for t in &self.types {
            match t {
                Atomic::TArray { key, value } => out.add_type(Atomic::TNonEmptyArray {
                    key: key.clone(),
                    value: value.clone(),
                }),
                Atomic::TList { value } => out.add_type(Atomic::TNonEmptyList {
                    value: value.clone(),
                }),
                _ => out.add_type(t.clone()),
            }
        }
        out
    }

    /// Narrow as if `array_is_list($x)` is true.
    /// Lists have sequential integer keys starting from 0, so:
    /// - `list<T>` / `non-empty-list<T>` are kept unchanged.
    /// - `array<int, T>` is narrowed to `list<T>` (could be sequential).
    /// - `non-empty-array<int, T>` is narrowed to `non-empty-list<T>`.
    /// - `mixed` becomes `list<mixed>` (array_is_list implies array).
    /// - All other types (string-keyed arrays, non-arrays) are dropped.
    pub fn narrow_to_list(&self) -> Type {
        let mut out = Type::empty();
        out.from_docblock = self.from_docblock;
        for t in &self.types {
            match t {
                Atomic::TList { .. } | Atomic::TNonEmptyList { .. } => out.add_type(t.clone()),
                Atomic::TArray { key, value } if matches!(key.types.as_slice(), [Atomic::TInt]) => {
                    out.add_type(Atomic::TList {
                        value: value.clone(),
                    });
                }
                Atomic::TNonEmptyArray { key, value }
                    if matches!(key.types.as_slice(), [Atomic::TInt]) =>
                {
                    out.add_type(Atomic::TNonEmptyList {
                        value: value.clone(),
                    });
                }
                Atomic::TMixed => out.add_type(Atomic::TList {
                    value: Box::new(Type::mixed()),
                }),
                _ => {}
            }
        }
        if out.is_empty() {
            self.filter(|t| matches!(t, Atomic::TList { .. } | Atomic::TNonEmptyList { .. }))
        } else {
            out
        }
    }

    /// Narrow as if `is_object($x)` is true. A `mixed` becomes a concrete bare
    /// `object` (rather than staying `mixed`) so downstream object-only
    /// operations — `clone`, `instanceof`, method calls — see an object type
    /// instead of reporting `Mixed*`.
    pub fn narrow_to_object(&self) -> Type {
        let mut out = Type::empty();
        for t in &self.types {
            if matches!(t, Atomic::TMixed) {
                out.add_type(Atomic::TObject);
            } else if t.is_object() {
                out.add_type(t.clone());
            }
        }
        if out.types.is_empty() {
            self.filter(|t| t.is_object())
        } else {
            out
        }
    }

    /// Narrow as if `is_callable($x)` is true.
    ///
    /// PHP accepts closures, TCallable, strings (function names), arrays
    /// (['Class', 'method'] or [$obj, 'method']), and objects with __invoke.
    /// Keep all of these; only drop atoms that are definitely not callable
    /// (scalars, null, bool, etc.).
    pub fn narrow_to_callable(&self) -> Type {
        self.filter(|t| {
            t.is_callable()
                || t.is_string()
                || t.is_array()
                || t.is_object()
                || matches!(t, Atomic::TMixed)
        })
    }

    /// Narrow as if `is_scalar($x)` is true (int | string | float | bool).
    pub fn narrow_to_scalar(&self) -> Type {
        self.filter(|t| {
            t.is_string()
                || t.is_int()
                || matches!(
                    t,
                    Atomic::TFloat
                        | Atomic::TLiteralFloat(..)
                        | Atomic::TBool
                        | Atomic::TTrue
                        | Atomic::TFalse
                        | Atomic::TScalar
                        | Atomic::TNumeric
                        | Atomic::TNumericString
                        | Atomic::TMixed
                )
        })
    }

    /// Narrow as if `is_iterable($x)` is true (array | Traversable).
    /// For simplicity, this narrows to arrays or objects (can't easily verify interfaces).
    pub fn narrow_to_iterable(&self) -> Type {
        self.filter(|t| t.is_array() || t.is_object() || matches!(t, Atomic::TMixed))
    }

    /// Narrow as if `is_countable($x)` is true (array | Countable).
    /// For simplicity, this narrows to arrays or objects (can't easily verify Countable interface).
    pub fn narrow_to_countable(&self) -> Type {
        self.filter(|t| t.is_array() || t.is_object() || matches!(t, Atomic::TMixed))
    }

    /// Narrow as if `is_resource($x)` is true.
    /// Note: No TResource atomic type exists in the type system; this is a no-op.
    /// Resources are declining in modern PHP and not actively tracked.
    pub fn narrow_to_resource(&self) -> Type {
        // No resource type in the system; just return mixed (allows any type)
        self.filter(|t| matches!(t, Atomic::TMixed))
    }

    // --- Merge (branch join) ------------------------------------------------

    /// Merge two unions at a branch join point (e.g. after if/else).
    /// The result is the union of all types in both.
    pub fn merge(a: &Type, b: &Type) -> Type {
        // Fast path: b is empty — nothing to add.
        if b.types.is_empty() {
            let mut result = a.clone();
            result.possibly_undefined = a.possibly_undefined || b.possibly_undefined;
            return result;
        }
        // Fast path: a is empty — clone b.
        if a.types.is_empty() {
            let mut result = b.clone();
            result.possibly_undefined = a.possibly_undefined || b.possibly_undefined;
            return result;
        }
        // Fast path: a is already mixed — b cannot widen it further.
        if a.types.len() == 1 && matches!(a.types[0], Atomic::TMixed) {
            let mut result = a.clone();
            result.possibly_undefined = a.possibly_undefined || b.possibly_undefined;
            return result;
        }
        // Fast path: b contains mixed — result collapses to mixed.
        if b.types.iter().any(|t| matches!(t, Atomic::TMixed)) {
            return Type {
                types: smallvec::smallvec![Atomic::TMixed],
                possibly_undefined: a.possibly_undefined || b.possibly_undefined,
                from_docblock: a.from_docblock || b.from_docblock,
            };
        }
        let mut result = a.clone();
        result.merge_with(b);
        result
    }

    /// Merge `other` into `self` in-place (avoids cloning `self`).
    pub fn merge_with(&mut self, other: &Type) {
        if self.types.iter().any(|t| matches!(t, Atomic::TMixed)) {
            self.possibly_undefined |= other.possibly_undefined;
            return;
        }
        if other.types.iter().any(|t| matches!(t, Atomic::TMixed)) {
            self.types.clear();
            self.types.push(Atomic::TMixed);
            self.possibly_undefined |= other.possibly_undefined;
            return;
        }
        for atomic in &other.types {
            self.add_type(atomic.clone());
        }
        self.possibly_undefined |= other.possibly_undefined;
    }

    /// Intersect with another union: keep only types present in `other`, widening
    /// where `self` contains `mixed` (which is compatible with everything).
    /// Used for match-arm subject narrowing.
    pub fn intersect_with(&self, other: &Type) -> Type {
        if self.is_mixed() {
            return other.clone();
        }
        if other.is_mixed() {
            return self.clone();
        }
        // Keep atomics from self that are also in other (by equality or subtype)
        let mut result = Type::empty();
        for a in &self.types {
            for b in &other.types {
                if a == b || atomic_subtype(a, b) || atomic_subtype(b, a) {
                    result.add_type(a.clone());
                    break;
                }
            }
        }
        if result.is_empty() {
            Type::never()
        } else {
            result
        }
    }

    // --- Template substitution ----------------------------------------------

    /// Replace template param references with their resolved types.
    pub fn substitute_templates(&self, bindings: &FxHashMap<Name, Type>) -> Type {
        if bindings.is_empty() {
            return self.clone();
        }
        let mut result = Type::empty();
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
                    param_name,
                    subject,
                    if_true,
                    if_false,
                } => {
                    let new_subject = subject.substitute_templates(bindings);
                    let new_if_true = if_true.substitute_templates(bindings);
                    let new_if_false = if_false.substitute_templates(bindings);

                    // If param_name names a template that is bound in this substitution,
                    // resolve the conditional immediately using the same predicate logic as
                    // `resolve_conditional_returns` for the $param form.
                    let resolved = if let Some(name) = param_name {
                        if let Some(bound) = bindings.get(name) {
                            if new_subject.types.len() == 1 {
                                resolve_conditional_branch(
                                    &new_subject.types[0],
                                    bound,
                                    &new_if_true,
                                    &new_if_false,
                                )
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(branch) = resolved {
                        for t in branch.types {
                            result.add_type(t);
                        }
                    } else {
                        result.add_type(Atomic::TConditional {
                            param_name: *param_name,
                            subject: Box::new(new_subject),
                            if_true: Box::new(new_if_true),
                            if_false: Box::new(new_if_false),
                        });
                    }
                }
                Atomic::TIntersection { parts } => {
                    result.add_type(Atomic::TIntersection {
                        parts: vec_to_type_params(
                            parts
                                .iter()
                                .map(|p| p.substitute_templates(bindings))
                                .collect(),
                        ),
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
                        if let Some(resolved) = bindings.get(fqcn) {
                            for t in &resolved.types {
                                result.add_type(t.clone());
                            }
                            continue;
                        }
                    }
                    let new_params: Vec<Type> = type_params
                        .iter()
                        .map(|p| p.substitute_templates(bindings))
                        .collect();
                    result.add_type(Atomic::TNamedObject {
                        fqcn: *fqcn,
                        type_params: vec_to_type_params(new_params),
                    });
                }
                // class-string<T> → substitute T from bindings
                Atomic::TClassString(Some(param_name)) => {
                    if let Some(resolved) = bindings.get(param_name) {
                        for r_atomic in &resolved.types {
                            let cls_name = if let Atomic::TNamedObject { fqcn, .. } = r_atomic {
                                Some(*fqcn)
                            } else {
                                None
                            };
                            result.add_type(Atomic::TClassString(cls_name));
                        }
                    } else {
                        result.add_type(atomic.clone());
                    }
                }
                _ => {
                    result.add_type(atomic.clone());
                }
            }
        }
        result
    }

    /// Resolves `TConditional` atoms whose discriminator is known at the call site.
    ///
    /// `lookup(param_name)` returns the call-site argument type for the named parameter,
    /// or `None` if the argument is not available. Handles `is null`, `is string`, and
    /// `is array` conditions; other condition types pass through unchanged.
    pub fn resolve_conditional_returns<F>(self, lookup: F) -> Type
    where
        F: Fn(&str) -> Option<Type>,
    {
        self.resolve_conditional_inner(&lookup)
    }

    fn resolve_conditional_inner<F>(self, lookup: &F) -> Type
    where
        F: Fn(&str) -> Option<Type>,
    {
        let mut result = Type::empty();
        for atomic in self.types {
            match atomic {
                Atomic::TConditional {
                    ref param_name,
                    ref subject,
                    ref if_true,
                    ref if_false,
                } => {
                    let resolved = if subject.types.len() == 1 {
                        if let Some(name) = param_name {
                            if let Some(arg_ty) = lookup(name.as_ref()) {
                                resolve_conditional_branch(
                                    &subject.types[0],
                                    &arg_ty,
                                    if_true,
                                    if_false,
                                )
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(branch) = resolved {
                        // Recursively resolve nested conditionals in the selected branch.
                        for t in branch.resolve_conditional_inner(lookup).types {
                            result.add_type(t);
                        }
                    } else {
                        // Cannot resolve at this call site: widen to the union of both branches.
                        // Recursively resolve nested conditionals in each branch.
                        for t in if_true.clone().resolve_conditional_inner(lookup).types {
                            result.add_type(t);
                        }
                        for t in if_false.clone().resolve_conditional_inner(lookup).types {
                            result.add_type(t);
                        }
                    }
                }
                other => result.add_type(other),
            }
        }
        result
    }

    // --- Subtype check -------------------------------------------------------

    /// Returns true if every atomic in `self` is a subtype of some atomic in `other`,
    /// using **only structural rules** — no `extends` / `implements` walk.
    ///
    /// Two distinct user-defined classes are never related here, even when one
    /// extends the other. Within `mir-analyzer`, when a `db` is in scope,
    /// prefer `crate::subtype::is_subtype(db, sub, sup)` which layers
    /// inheritance resolution on top of this check.
    pub fn is_subtype_structural(&self, other: &Type) -> bool {
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

    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Type {
        let mut result = Type::empty();
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
// Conditional return resolution helpers
// ---------------------------------------------------------------------------

fn is_string_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TString
            | Atomic::TNonEmptyString
            | Atomic::TLiteralString(_)
            | Atomic::TNumericString
            | Atomic::TClassString(_)
            | Atomic::TCallableString
    )
}

fn is_array_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TArray { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TKeyedArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyList { .. }
    )
}

fn is_list_atomic(a: &Atomic) -> bool {
    match a {
        Atomic::TList { .. } | Atomic::TNonEmptyList { .. } => true,
        Atomic::TKeyedArray { is_list, .. } => *is_list,
        _ => false,
    }
}

/// Resolve one branch of a conditional return type given the subject discriminant
/// atomic and the actual argument type at the call site.
///
/// Returns `Some(branch)` when the branch can be determined statically, or `None`
/// to signal that the caller should widen to the union of both branches.
fn resolve_conditional_branch(
    subject: &Atomic,
    arg_ty: &Type,
    if_true: &Type,
    if_false: &Type,
) -> Option<Type> {
    let predicate: fn(&Atomic) -> bool = match subject {
        Atomic::TNull => |a| matches!(a, Atomic::TNull),
        Atomic::TTrue => |a| matches!(a, Atomic::TTrue),
        Atomic::TFalse => |a| matches!(a, Atomic::TFalse),
        Atomic::TString => is_string_atomic,
        Atomic::TList { .. } => is_list_atomic,
        Atomic::TArray { .. } => is_array_atomic,
        _ => return None,
    };

    if arg_ty.types.is_empty() {
        return None;
    }
    let all_match = arg_ty.types.iter().all(&predicate);
    let none_match = !arg_ty.types.iter().any(predicate);
    if all_match {
        Some(if_true.clone())
    } else if none_match {
        Some(if_false.clone())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Template substitution helpers
// ---------------------------------------------------------------------------

fn substitute_in_fn_param(
    p: &crate::atomic::FnParam,
    bindings: &FxHashMap<Name, Type>,
) -> crate::atomic::FnParam {
    crate::atomic::FnParam {
        name: p.name,
        ty: p.ty.as_ref().map(|t| {
            let u = t.to_union();
            let substituted = u.substitute_templates(bindings);
            crate::compact::SimpleType::from_union(substituted)
        }),
        default: p.default.as_ref().map(|d| {
            let u = d.to_union();
            let substituted = u.substitute_templates(bindings);
            crate::compact::SimpleType::from_union(substituted)
        }),
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
        // Template param in supertype position: any value satisfies an unconstrained
        // template (as_type = mixed), or a constrained one if it satisfies the bound.
        // This handles union bounds like `T of string|list<I>|array<K, V>` where
        // I/K/V are free template params — any type satisfies them structurally.
        (_, Atomic::TTemplateParam { as_type, .. }) => {
            as_type.is_mixed() || as_type.types.iter().any(|b| atomic_subtype(sub, b))
        }

        // Scalars
        (Atomic::TLiteralInt(_), Atomic::TInt) => true,
        (Atomic::TLiteralInt(_), Atomic::TNumeric) => true,
        (Atomic::TLiteralInt(_), Atomic::TScalar) => true,
        (Atomic::TLiteralInt(n), Atomic::TPositiveInt) => *n > 0,
        (Atomic::TLiteralInt(n), Atomic::TNonNegativeInt) => *n >= 0,
        (Atomic::TLiteralInt(n), Atomic::TNegativeInt) => *n < 0,
        (Atomic::TPositiveInt, Atomic::TInt) => true,
        (Atomic::TPositiveInt, Atomic::TNonNegativeInt) => true,
        (Atomic::TPositiveInt, Atomic::TNumeric) => true,
        (Atomic::TPositiveInt, Atomic::TScalar) => true,
        (Atomic::TNegativeInt, Atomic::TInt) => true,
        (Atomic::TNegativeInt, Atomic::TNumeric) => true,
        (Atomic::TNegativeInt, Atomic::TScalar) => true,
        (Atomic::TNonNegativeInt, Atomic::TInt) => true,
        (Atomic::TNonNegativeInt, Atomic::TNumeric) => true,
        (Atomic::TNonNegativeInt, Atomic::TScalar) => true,
        (Atomic::TIntRange { .. }, Atomic::TInt) => true,
        (Atomic::TIntRange { .. }, Atomic::TNumeric) => true,
        (Atomic::TIntRange { .. }, Atomic::TScalar) => true,
        // positive-int is int<1, ∞>: subtype of int<sup_min, ∞> when sup_min <= 1
        (Atomic::TPositiveInt, Atomic::TIntRange { min, max }) => {
            max.is_none() && min.is_none_or(|m| m <= 1)
        }
        // negative-int is int<-∞, -1>: subtype of int<-∞, sup_max> when sup_max >= -1
        (Atomic::TNegativeInt, Atomic::TIntRange { min, max }) => {
            min.is_none() && max.is_none_or(|m| m >= -1)
        }
        // non-negative-int is int<0, ∞>: subtype of int<sup_min, ∞> when sup_min <= 0
        (Atomic::TNonNegativeInt, Atomic::TIntRange { min, max }) => {
            max.is_none() && min.is_none_or(|m| m <= 0)
        }
        // A bounded int range is a subtype of a named int subtype when every value fits
        (Atomic::TIntRange { min: sub_min, .. }, Atomic::TPositiveInt) => {
            sub_min.is_some_and(|lo| lo >= 1)
        }
        (Atomic::TIntRange { min: sub_min, .. }, Atomic::TNonNegativeInt) => {
            sub_min.is_some_and(|lo| lo >= 0)
        }
        (Atomic::TIntRange { max: sub_max, .. }, Atomic::TNegativeInt) => {
            sub_max.is_some_and(|hi| hi <= -1)
        }
        // int<sub_min, sub_max> <: int<sup_min, sup_max> when ranges nest
        (
            Atomic::TIntRange {
                min: sub_min,
                max: sub_max,
            },
            Atomic::TIntRange {
                min: sup_min,
                max: sup_max,
            },
        ) => {
            let lower_ok = match (sub_min, sup_min) {
                (_, None) => true,
                (None, Some(_)) => false,
                (Some(sl), Some(su)) => sl >= su,
            };
            let upper_ok = match (sub_max, sup_max) {
                (None, None) | (Some(_), None) => true,
                (None, Some(_)) => false,
                (Some(sl), Some(su)) => sl <= su,
            };
            lower_ok && upper_ok
        }

        (Atomic::TLiteralFloat(..), Atomic::TFloat) => true,
        (Atomic::TLiteralFloat(..), Atomic::TNumeric) => true,
        (Atomic::TLiteralFloat(..), Atomic::TScalar) => true,

        (Atomic::TLiteralString(s), Atomic::TString) => {
            let _ = s;
            true
        }
        (Atomic::TLiteralString(s), Atomic::TCallableString) => {
            let _ = s;
            true
        }
        (Atomic::TLiteralString(s), Atomic::TNonEmptyString) => !s.is_empty(),
        (Atomic::TLiteralString(s), Atomic::TNumericString) => s.parse::<f64>().is_ok(),
        // A literal string is type-compatible with class-string; validate_class_string_argument
        // separately checks whether the string names a real class (UndefinedClass).
        (Atomic::TLiteralString(_), Atomic::TClassString(_)) => true,
        (Atomic::TLiteralString(_), Atomic::TScalar) => true,
        (Atomic::TNonEmptyString, Atomic::TString) => true,
        (Atomic::TCallableString, Atomic::TString) => true,
        // numeric-string is always non-empty (e.g. "42", "-1", "0.5") — "" is not numeric.
        (Atomic::TNumericString, Atomic::TNonEmptyString) => true,
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
        // Bare generic property accepts parameterized value: Box accepts Box<string>.
        // The reverse is NOT true — bare Box value does not satisfy Box<string> property
        // (invariant check). Only sup being bare (empty type_params) is the wildcard.
        (
            Atomic::TNamedObject {
                fqcn: sub_fqcn,
                type_params: sub_params,
            },
            Atomic::TNamedObject {
                fqcn: sup_fqcn,
                type_params: sup_params,
            },
        ) => {
            sub_fqcn == sup_fqcn
                && (sup_params.is_empty() || type_params_compatible(sub_params, sup_params))
        }

        // Literal int widens to float in PHP
        (Atomic::TLiteralInt(_), Atomic::TFloat) => true,
        (Atomic::TPositiveInt, Atomic::TFloat) => true,
        (Atomic::TNegativeInt, Atomic::TFloat) => true,
        (Atomic::TNonNegativeInt, Atomic::TFloat) => true,
        (Atomic::TInt, Atomic::TFloat) => true,
        (Atomic::TIntRange { .. }, Atomic::TFloat) => true,

        // Literal int satisfies an int range only when the value is within bounds
        (Atomic::TLiteralInt(n), Atomic::TIntRange { min, max }) => {
            min.is_none_or(|lo| *n >= lo) && max.is_none_or(|hi| *n <= hi)
        }

        // PHP callables: string and array are valid callable values
        (Atomic::TString, Atomic::TCallable { .. }) => true,
        (Atomic::TNonEmptyString, Atomic::TCallable { .. }) => true,
        (Atomic::TLiteralString(_), Atomic::TCallable { .. }) => true,
        (Atomic::TArray { .. }, Atomic::TCallable { .. }) => true,
        (Atomic::TNonEmptyArray { .. }, Atomic::TCallable { .. }) => true,
        (Atomic::TKeyedArray { .. }, Atomic::TCallable { .. }) => true,

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
        // bare `Closure` (named object without signature) satisfies any typed Closure(): T
        (Atomic::TNamedObject { fqcn, .. }, Atomic::TClosure { .. }) => {
            fqcn.as_ref().eq_ignore_ascii_case("closure")
        }
        // `Closure` named-object satisfies `callable`
        (Atomic::TNamedObject { fqcn, .. }, Atomic::TCallable { .. }) => {
            fqcn.as_ref().eq_ignore_ascii_case("closure")
        }

        // List <: array  (list key is always int; int must satisfy the array's key type)
        (Atomic::TList { value }, Atomic::TArray { key, value: av }) => {
            Type::single(Atomic::TInt).is_subtype_structural(key) && value.is_subtype_structural(av)
        }
        (Atomic::TNonEmptyList { value }, Atomic::TArray { key, value: av }) => {
            Type::single(Atomic::TInt).is_subtype_structural(key) && value.is_subtype_structural(av)
        }
        (Atomic::TNonEmptyList { value }, Atomic::TNonEmptyArray { key, value: av }) => {
            Type::single(Atomic::TInt).is_subtype_structural(key) && value.is_subtype_structural(av)
        }
        (Atomic::TNonEmptyList { value }, Atomic::TList { value: lv }) => {
            value.is_subtype_structural(lv)
        }
        // array<int, X> is accepted where list<X> or non-empty-list<X> expected
        (Atomic::TArray { key, value: av }, Atomic::TList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_structural(lv)
        }
        (Atomic::TArray { key, value: av }, Atomic::TNonEmptyList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_structural(lv)
        }
        (Atomic::TNonEmptyArray { key, value: av }, Atomic::TList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_structural(lv)
        }
        (Atomic::TNonEmptyArray { key, value: av }, Atomic::TNonEmptyList { value: lv }) => {
            matches!(key.types.as_slice(), [Atomic::TInt | Atomic::TMixed])
                && av.is_subtype_structural(lv)
        }
        // TList <: TList value covariance
        (Atomic::TList { value: v1 }, Atomic::TList { value: v2 }) => v1.is_subtype_structural(v2),
        (Atomic::TNonEmptyArray { key: k1, value: v1 }, Atomic::TArray { key: k2, value: v2 }) => {
            k1.is_subtype_structural(k2) && v1.is_subtype_structural(v2)
        }

        // array<A, B> <: array<C, D>  iff  A <: C && B <: D
        (Atomic::TArray { key: k1, value: v1 }, Atomic::TArray { key: k2, value: v2 }) => {
            k1.is_subtype_structural(k2) && v1.is_subtype_structural(v2)
        }

        // A keyed/shape array is a subtype of array<K, V> / non-empty-array<K, V>
        // when all property KEYS are subtypes of K. Value compatibility is checked
        // structurally only for scalar types; named-object values are deferred to
        // class-hierarchy checks in return_arrays_compatible (mir-analyzer).
        // Open shapes (is_open=true) may have extra unknown keys: keep permissive.
        (
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            },
            Atomic::TArray { key, value },
        ) => {
            if *is_open {
                return true;
            }
            properties.iter().all(|(prop_key, prop)| {
                let key_atomic = match prop_key {
                    crate::atomic::ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                    crate::atomic::ArrayKey::Int(n) => Atomic::TLiteralInt(*n),
                };
                if !Type::single(key_atomic).is_subtype_structural(key) {
                    return false; // key mismatch — definitively incompatible
                }
                // Named-object values require class-hierarchy checks not available here.
                let has_named_obj = prop.ty.types.iter().any(|a| {
                    matches!(
                        a,
                        Atomic::TNamedObject { .. }
                            | Atomic::TSelf { .. }
                            | Atomic::TStaticObject { .. }
                            | Atomic::TClosure { .. }
                            | Atomic::TTemplateParam { .. }
                    )
                });
                has_named_obj || prop.ty.is_subtype_structural(value)
            })
        }
        (
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            },
            Atomic::TNonEmptyArray { key, value },
        ) => {
            if *is_open {
                return !properties.is_empty();
            }
            properties.iter().any(|(_, p)| !p.optional)
                && properties.iter().all(|(prop_key, prop)| {
                    let key_atomic = match prop_key {
                        crate::atomic::ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                        crate::atomic::ArrayKey::Int(n) => Atomic::TLiteralInt(*n),
                    };
                    if !Type::single(key_atomic).is_subtype_structural(key) {
                        return false;
                    }
                    let has_named_obj = prop.ty.types.iter().any(|a| {
                        matches!(
                            a,
                            Atomic::TNamedObject { .. }
                                | Atomic::TSelf { .. }
                                | Atomic::TStaticObject { .. }
                                | Atomic::TClosure { .. }
                                | Atomic::TTemplateParam { .. }
                        )
                    });
                    has_named_obj || prop.ty.is_subtype_structural(value)
                })
        }

        // A list-shaped keyed array (is_list=true, all int keys) is a subtype of list<X>.
        (
            Atomic::TKeyedArray {
                properties,
                is_list,
                ..
            },
            Atomic::TList { value: lv },
        ) => *is_list && properties.values().all(|p| p.ty.is_subtype_structural(lv)),
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
                && properties.values().all(|p| p.ty.is_subtype_structural(lv))
        }

        _ => false,
    }
}

/// Whether each generic type-argument in `sub` is compatible with the
/// corresponding argument in `sup`. Arguments are invariant (require structural
/// equality) with one exception: an empty array literal (`array{}`) is accepted
/// against any array/list argument, so `new Box([])` — inferred as
/// `Box<array{}>` — satisfies a declared `Box<list<T>>` for any `T`.
fn type_params_compatible(sub: &[Type], sup: &[Type]) -> bool {
    if sub.len() != sup.len() {
        return false;
    }
    sub.iter()
        .zip(sup.iter())
        .all(|(a, b)| a == b || (is_empty_array_literal(a) && is_array_like(b)))
}

/// True for a non-empty union whose atoms are all empty keyed arrays (`array{}`),
/// i.e. the type of an empty array literal `[]`.
fn is_empty_array_literal(t: &Type) -> bool {
    !t.types.is_empty()
        && t.types.iter().all(
            |atom| matches!(atom, Atomic::TKeyedArray { properties, .. } if properties.is_empty()),
        )
}

/// True for a non-empty union whose atoms are all array/list types.
fn is_array_like(t: &Type) -> bool {
    !t.types.is_empty() && t.types.iter().all(|atom| atom.is_array())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn single_is_single() {
        let u = Type::single(Atomic::TString);
        assert!(u.is_single());
        assert!(!u.is_nullable());
    }

    #[test]
    fn nullable_has_null() {
        let u = Type::nullable(Atomic::TString);
        assert!(u.is_nullable());
        assert_eq!(u.types.len(), 2);
    }

    #[test]
    fn add_type_deduplicates() {
        let mut u = Type::single(Atomic::TString);
        u.add_type(Atomic::TString);
        assert_eq!(u.types.len(), 1);
    }

    #[test]
    fn add_type_literal_subsumed_by_base() {
        let mut u = Type::single(Atomic::TInt);
        u.add_type(Atomic::TLiteralInt(42));
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TInt));
    }

    #[test]
    fn add_type_base_widens_literals() {
        let mut u = Type::single(Atomic::TLiteralInt(1));
        u.add_type(Atomic::TLiteralInt(2));
        u.add_type(Atomic::TInt);
        assert_eq!(u.types.len(), 1);
        assert!(matches!(u.types[0], Atomic::TInt));
    }

    #[test]
    fn mixed_subsumes_everything() {
        let mut u = Type::single(Atomic::TString);
        u.add_type(Atomic::TMixed);
        assert_eq!(u.types.len(), 1);
        assert!(u.is_mixed());
    }

    #[test]
    fn remove_null() {
        let u = Type::nullable(Atomic::TString);
        let narrowed = u.remove_null();
        assert!(!narrowed.is_nullable());
        assert_eq!(narrowed.types.len(), 1);
    }

    #[test]
    fn narrow_to_truthy_removes_null_false() {
        let mut u = Type::empty();
        u.add_type(Atomic::TString);
        u.add_type(Atomic::TNull);
        u.add_type(Atomic::TFalse);
        let truthy = u.narrow_to_truthy();
        assert!(!truthy.is_nullable());
        assert!(!truthy.contains(|t| matches!(t, Atomic::TFalse)));
    }

    #[test]
    fn merge_combines_types() {
        let a = Type::single(Atomic::TString);
        let b = Type::single(Atomic::TInt);
        let merged = Type::merge(&a, &b);
        assert_eq!(merged.types.len(), 2);
    }

    #[test]
    fn subtype_literal_int_under_int() {
        let sub = Type::single(Atomic::TLiteralInt(5));
        let sup = Type::single(Atomic::TInt);
        assert!(sub.is_subtype_structural(&sup));
    }

    #[test]
    fn subtype_never_is_bottom() {
        let never = Type::never();
        let string = Type::single(Atomic::TString);
        assert!(never.is_subtype_structural(&string));
    }

    #[test]
    fn subtype_everything_under_mixed() {
        let string = Type::single(Atomic::TString);
        let mixed = Type::mixed();
        assert!(string.is_subtype_structural(&mixed));
    }

    #[test]
    fn template_substitution() {
        let mut bindings = FxHashMap::default();
        bindings.insert(Name::new("T"), Type::single(Atomic::TString));

        let tmpl = Type::single(Atomic::TTemplateParam {
            name: Name::new("T"),
            as_type: Box::new(Type::mixed()),
            defining_entity: Name::new("MyClass"),
        });

        let resolved = tmpl.substitute_templates(&bindings);
        assert_eq!(resolved.types.len(), 1);
        assert!(matches!(resolved.types[0], Atomic::TString));
    }

    #[test]
    fn intersection_is_object() {
        let parts = vec![
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("Iterator"),
                type_params: empty_type_params(),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("Countable"),
                type_params: empty_type_params(),
            }),
        ];
        let atomic = Atomic::TIntersection {
            parts: vec_to_type_params(parts),
        };
        assert!(atomic.is_object());
        assert!(!atomic.can_be_falsy());
        assert!(atomic.can_be_truthy());
    }

    #[test]
    fn intersection_display_two_parts() {
        let parts = vec![
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("Iterator"),
                type_params: empty_type_params(),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("Countable"),
                type_params: empty_type_params(),
            }),
        ];
        let u = Type::single(Atomic::TIntersection {
            parts: vec_to_type_params(parts),
        });
        assert_eq!(format!("{u}"), "Iterator&Countable");
    }

    #[test]
    fn intersection_display_three_parts() {
        let parts = vec![
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("A"),
                type_params: empty_type_params(),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("B"),
                type_params: empty_type_params(),
            }),
            Type::single(Atomic::TNamedObject {
                fqcn: Name::new("C"),
                type_params: empty_type_params(),
            }),
        ];
        let u = Type::single(Atomic::TIntersection {
            parts: vec_to_type_params(parts),
        });
        assert_eq!(format!("{u}"), "A&B&C");
    }

    #[test]
    fn intersection_in_nullable_union_display() {
        let intersection = Atomic::TIntersection {
            parts: vec_to_type_params(vec![
                Type::single(Atomic::TNamedObject {
                    fqcn: Name::new("Iterator"),
                    type_params: empty_type_params(),
                }),
                Type::single(Atomic::TNamedObject {
                    fqcn: Name::new("Countable"),
                    type_params: empty_type_params(),
                }),
            ]),
        };
        let mut u = Type::single(intersection);
        u.add_type(Atomic::TNull);
        assert!(u.is_nullable());
        assert!(u.contains(|t| matches!(t, Atomic::TIntersection { .. })));
    }

    // --- substitute_templates coverage for previously-missing arms ----------

    fn t_param(name: &str) -> Type {
        Type::single(Atomic::TTemplateParam {
            name: Name::new(name),
            as_type: Box::new(Type::mixed()),
            defining_entity: Name::new("Fn"),
        })
    }

    fn bindings_t_string() -> FxHashMap<Name, Type> {
        let mut b = FxHashMap::default();
        b.insert(Name::new("T"), Type::single(Atomic::TString));
        b
    }

    #[test]
    fn substitute_non_empty_array_key_and_value() {
        let ty = Type::single(Atomic::TNonEmptyArray {
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
        let ty = Type::single(Atomic::TNonEmptyList {
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
        let ty = Type::single(Atomic::TKeyedArray {
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
        let ty = Type::single(Atomic::TCallable {
            params: Some(vec![FnParam {
                name: Name::new("x"),
                ty: Some(crate::compact::SimpleType::from_union(t_param("T"))),
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
        let param_union = param_ty.to_union();
        assert!(matches!(param_union.types[0], Atomic::TString));
        let ret = return_type.as_ref().unwrap();
        assert!(matches!(ret.types[0], Atomic::TString));
    }

    #[test]
    fn substitute_callable_bare_no_panic() {
        // callable with no params/return — must not panic and must pass through unchanged
        let ty = Type::single(Atomic::TCallable {
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
        let ty = Type::single(Atomic::TClosure {
            params: vec![FnParam {
                name: Name::new("a"),
                ty: Some(crate::compact::SimpleType::from_union(t_param("T"))),
                default: Some(crate::compact::SimpleType::from_union(t_param("T"))),
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
        let ty_union = p.ty.as_ref().unwrap().to_union();
        let default_union = p.default.as_ref().unwrap().to_union();
        assert!(matches!(ty_union.types[0], Atomic::TString));
        assert!(matches!(default_union.types[0], Atomic::TString));
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
        let ty = Type::single(Atomic::TConditional {
            param_name: None,
            subject: Box::new(t_param("T")),
            if_true: Box::new(t_param("T")),
            if_false: Box::new(Type::single(Atomic::TInt)),
        });
        let result = ty.substitute_templates(&bindings_t_string());
        let Atomic::TConditional {
            param_name: _,
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
    fn resolve_conditional_is_null_non_null_arg() {
        let ty = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TNull)),
            if_true: Box::new(Type::single(Atomic::TInt)),
            if_false: Box::new(Type::single(Atomic::TString)),
        });
        let result = ty.resolve_conditional_returns(|name| {
            if name == "x" {
                Some(Type::single(Atomic::TString)) // definitely not null
            } else {
                None
            }
        });
        assert!(result.types.len() == 1);
        assert!(matches!(result.types[0], Atomic::TString));
    }

    #[test]
    fn resolve_conditional_is_null_null_arg() {
        let ty = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TNull)),
            if_true: Box::new(Type::single(Atomic::TInt)),
            if_false: Box::new(Type::single(Atomic::TString)),
        });
        let result = ty.resolve_conditional_returns(|name| {
            if name == "x" {
                Some(Type::single(Atomic::TNull)) // definitely null
            } else {
                None
            }
        });
        assert!(result.types.len() == 1);
        assert!(matches!(result.types[0], Atomic::TInt));
    }

    #[test]
    fn resolve_conditional_is_null_nullable_arg_widens_to_branch_union() {
        let mut nullable_str = Type::single(Atomic::TString);
        nullable_str.add_type(Atomic::TNull);
        let ty = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TNull)),
            if_true: Box::new(Type::single(Atomic::TInt)),
            if_false: Box::new(Type::single(Atomic::TString)),
        });
        let result = ty.resolve_conditional_returns(|name| {
            if name == "x" {
                Some(nullable_str.clone())
            } else {
                None
            }
        });
        // uncertain discriminator → widen to if_true | if_false
        assert_eq!(result.types.len(), 2);
        assert!(result.types.iter().any(|t| matches!(t, Atomic::TInt)));
        assert!(result.types.iter().any(|t| matches!(t, Atomic::TString)));
    }

    #[test]
    fn resolve_conditional_nested_widens_inner_branch() {
        // ($x is null ? int : ($x is string ? string : float))
        // When $x is unknown, should widen to int|string|float (no TConditional remaining).
        let inner = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TString)),
            if_true: Box::new(Type::single(Atomic::TString)),
            if_false: Box::new(Type::single(Atomic::TFloat)),
        });
        let ty = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TNull)),
            if_true: Box::new(Type::single(Atomic::TInt)),
            if_false: Box::new(inner),
        });
        // unknown arg → widen both outer branches, inner conditional must also be widened
        let result = ty.resolve_conditional_returns(|_| None);
        assert!(
            result
                .types
                .iter()
                .all(|t| !matches!(t, Atomic::TConditional { .. })),
            "no TConditional should survive: {:?}",
            result.types
        );
        assert!(result.types.iter().any(|t| matches!(t, Atomic::TInt)));
        assert!(result.types.iter().any(|t| matches!(t, Atomic::TString)));
        assert!(result.types.iter().any(|t| matches!(t, Atomic::TFloat)));
    }

    #[test]
    fn resolve_conditional_nested_resolves_inner_branch() {
        // ($x is null ? int : ($x is string ? string : float))
        // When $x is definitely not null but unknown string-or-not → resolves outer to inner,
        // then inner must also be resolved.
        let inner = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TString)),
            if_true: Box::new(Type::single(Atomic::TString)),
            if_false: Box::new(Type::single(Atomic::TFloat)),
        });
        let ty = Type::single(Atomic::TConditional {
            param_name: Some(Name::new("x")),
            subject: Box::new(Type::single(Atomic::TNull)),
            if_true: Box::new(Type::single(Atomic::TInt)),
            if_false: Box::new(inner),
        });
        // $x = string → outer: not null → if_false (inner); inner: is string → if_true = string
        let result = ty.resolve_conditional_returns(|name| {
            if name == "x" {
                Some(Type::single(Atomic::TString))
            } else {
                None
            }
        });
        assert!(
            result
                .types
                .iter()
                .all(|t| !matches!(t, Atomic::TConditional { .. })),
            "no TConditional should survive: {:?}",
            result.types
        );
        assert_eq!(result.types.len(), 1);
        assert!(matches!(result.types[0], Atomic::TString));
    }

    #[test]
    fn substitute_intersection_parts() {
        let ty = Type::single(Atomic::TIntersection {
            parts: vec_to_type_params(vec![
                Type::single(Atomic::TNamedObject {
                    fqcn: Name::new("Countable"),
                    type_params: empty_type_params(),
                }),
                t_param("T"),
            ]),
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
        let ty = Type::single(Atomic::TInt);
        let result = ty.substitute_templates(&bindings_t_string());
        assert!(matches!(result.types[0], Atomic::TInt));
    }
}
