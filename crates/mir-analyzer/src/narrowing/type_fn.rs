//! `is_*`/`ctype_*`/`array_is_list`/`method_exists`/`property_exists` type-check
//! narrowing, for variable, property, and static-property receivers.
use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, is_numeric_string, narrow_receiver_non_null_on_prop_match,
    resolve_prop_current_type, resolve_static_prop_current_type, set_narrowed, UnionNarrowExt,
};
use super::literals::narrow_string_to_non_empty;

pub(super) fn narrow_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    var_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = ctx.get_var(var_name);
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    set_narrowed(ctx, var_name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_from_type_fn`, for
/// `is_string($this->prop)`, `is_array($this->prop)`, `array_is_list($this->prop)`,
/// `ctype_digit($this->prop)`, `method_exists($this->prop, ...)`, etc. — the
/// whole `is_*`/`ctype_*`/type-check family previously only ever narrowed a
/// plain-variable receiver, unlike the analogous `instanceof`/null/literal-match
/// arms elsewhere in this file, which all have a property-access fallback.
pub(super) fn narrow_prop_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    // A nullable receiver makes `$obj->prop` itself evaluate to `null` (PHP 8
    // warning, not fatal), which is an extra value the property's own declared
    // type doesn't account for. That extra `null` only satisfies `is_null()`'s
    // true branch — every other `is_*`/`ctype_*` check returns false on null,
    // so it's their false branch that gains a reachable-despite-empty case
    // (same reasoning as `narrow_prop_instanceof`'s false-branch gate).
    let is_null_check = crate::util::php_ident_lowercase(fn_name) == "is_null";
    let receiver_nullable = ctx.get_var(obj_var).is_nullable();
    // Proves the property's own value isn't null (`is_true != is_null_check`,
    // same condition as mark_diverges above) — which, on a nullable receiver,
    // also proves the receiver itself wasn't null (see the comment above).
    let proved_prop_non_null = is_true != is_null_check;
    let mark_diverges = !receiver_nullable || proved_prop_non_null;
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
    narrow_receiver_non_null_on_prop_match(ctx, obj_var, proved_prop_non_null);
}

/// Static-property counterpart of `narrow_prop_from_type_fn`, for
/// `is_string(self::$prop)`, `ctype_digit(static::$prop)`, etc. Unlike the
/// instance-property case, a static property has no separate "receiver
/// variable" whose own nullability could add an extra unaccounted-for
/// `null` — `self::`/`static::` is never itself null — so mark_diverges is
/// unconditional, matching the plain-variable case.
pub(super) fn narrow_static_prop_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Core `is_*`/`ctype_*`/`array_is_list`/`method_exists`/`property_exists`
/// narrowing logic, shared between the variable-receiver
/// (`narrow_from_type_fn`) and property-receiver (`narrow_prop_from_type_fn`)
/// entry points. Returns `None` for an unrecognized function name — the
/// caller should leave the type untouched.
pub(super) fn type_fn_narrowed(
    current: &Type,
    fn_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) -> Option<Type> {
    Some(match crate::util::php_ident_lowercase(fn_name).as_str() {
        "is_string" => {
            if is_true {
                current.narrow_to_string()
            } else {
                current.filter(|t| !t.is_string())
            }
        }
        "is_int" | "is_integer" | "is_long" => {
            if is_true {
                current.narrow_to_int()
            } else {
                current.filter(|t| !t.is_int())
            }
        }
        "is_float" | "is_double" | "is_real" => {
            if is_true {
                current.narrow_to_float()
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
                    )
                })
            }
        }
        "is_bool" => {
            if is_true {
                current.narrow_to_bool()
            } else {
                current.filter(|t| !matches!(t, Atomic::TBool | Atomic::TTrue | Atomic::TFalse))
            }
        }
        "is_null" => {
            if is_true {
                current.narrow_to_null()
            } else {
                current.remove_null()
            }
        }
        "is_array" => {
            if is_true {
                current.narrow_to_array()
            } else {
                current.filter(|t| !t.is_array())
            }
        }
        "array_is_list" => {
            if is_true {
                current.narrow_to_list()
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { is_list: true, .. }
                    )
                })
            }
        }
        "is_object" => {
            if is_true {
                current.narrow_to_object()
            } else {
                current.filter(|t| !t.is_object())
            }
        }
        "is_callable" => {
            if is_true {
                current.narrow_to_callable()
            } else {
                current.filter(|t| !t.is_callable())
            }
        }
        "is_scalar" => {
            if is_true {
                current.narrow_to_scalar()
            } else {
                current.filter(|t| {
                    !t.is_string()
                        && !t.is_int()
                        && !matches!(
                            t,
                            Atomic::TFloat
                                | Atomic::TIntegralFloat
                                | Atomic::TLiteralFloat(..)
                                | Atomic::TBool
                                | Atomic::TTrue
                                | Atomic::TFalse
                                | Atomic::TScalar
                                | Atomic::TNumeric
                        )
                })
            }
        }
        "is_iterable" => {
            if is_true {
                current.narrow_to_iterable()
            } else {
                // Beyond the array atom (always excludable), a named-object atom
                // is only excludable when it's `final` (no subclass could add
                // `implements Traversable` later) AND its own hierarchy provably
                // doesn't already extend/implement Traversable — same
                // final-class soundness gate `narrow_var_to_specific_class` uses
                // for its false branch. A non-final or unresolvable class stays,
                // same conservatism as before this class-hierarchy check existed.
                current
                    .filter(|t| !atom_excluded_from_is_iterable_or_countable(t, "Traversable", db))
            }
        }
        "is_countable" => {
            if is_true {
                current.narrow_to_countable()
            } else {
                current.filter(|t| !atom_excluded_from_is_iterable_or_countable(t, "Countable", db))
            }
        }
        "is_resource" => {
            if is_true {
                current.narrow_to_resource()
            } else {
                // Exclude nothing (no resource type exists); return unchanged
                current.clone()
            }
        }
        "is_numeric" => {
            if is_true {
                // In the truthy branch: keep numeric types and string types that
                // *could* be numeric. TString / TNonEmptyString narrow to TNumericString
                // (a string proven to be numeric-valued). All int and float variants are
                // always numeric. TMixed is kept as-is.
                let mut narrowed_parts = Type::empty();
                for t in &current.types {
                    match t {
                        // All int and float variants are unconditionally numeric.
                        Atomic::TInt
                        | Atomic::TIntRange { .. }
                        | Atomic::TPositiveInt
                        | Atomic::TNonNegativeInt
                        | Atomic::TNegativeInt
                        | Atomic::TLiteralInt(_)
                        | Atomic::TFloat
                        | Atomic::TIntegralFloat
                        | Atomic::TLiteralFloat(..)
                        | Atomic::TNumeric
                        | Atomic::TNumericString => {
                            narrowed_parts.add_type(t.clone());
                        }
                        // A generic string could be numeric; narrow to numeric-string.
                        Atomic::TString | Atomic::TNonEmptyString => {
                            narrowed_parts.add_type(Atomic::TNumericString);
                        }
                        // A literal string is numeric only if it parses as a number.
                        Atomic::TLiteralString(s) if is_numeric_string(s) => {
                            narrowed_parts.add_type(t.clone());
                        }
                        // mixed/scalar could be anything; a truthy is_numeric()
                        // proves it's specifically int|float|numeric-string, so
                        // replace it rather than leaving it as mixed/scalar (matching
                        // how is_string/is_int/etc. narrow these two atoms).
                        Atomic::TScalar | Atomic::TMixed => {
                            narrowed_parts.add_type(Atomic::TInt);
                            narrowed_parts.add_type(Atomic::TFloat);
                            narrowed_parts.add_type(Atomic::TNumericString);
                        }
                        _ => {} // non-numeric types are excluded
                    }
                }
                narrowed_parts
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TInt
                            | Atomic::TIntRange { .. }
                            | Atomic::TPositiveInt
                            | Atomic::TNonNegativeInt
                            | Atomic::TNegativeInt
                            | Atomic::TFloat
                            | Atomic::TIntegralFloat
                            | Atomic::TNumeric
                            | Atomic::TNumericString
                            | Atomic::TLiteralInt(_)
                            | Atomic::TLiteralFloat(..)
                    ) && !matches!(t, Atomic::TLiteralString(s) if is_numeric_string(s.as_ref()))
                })
            }
        }
        // ctype_*() returns false on an empty string for every variant, so a
        // truthy result proves the string argument is non-empty. It says
        // nothing when the argument isn't a string (e.g. ctype_digit(65) is
        // true because 65 is ASCII 'A', unrelated to decimal digits), so
        // only TString atoms are touched — everything else passes through.
        "ctype_alpha" | "ctype_alnum" | "ctype_digit" | "ctype_lower" | "ctype_upper"
        | "ctype_punct" | "ctype_space" | "ctype_xdigit" | "ctype_print" | "ctype_graph"
        | "ctype_cntrl" => {
            if is_true {
                narrow_string_to_non_empty(current)
            } else {
                current.clone()
            }
        }
        // method_exists($obj, 'method') / property_exists($obj, 'prop') — both accept
        // object|string, so on true keep object atoms as-is (preserving the specific
        // class instead of collapsing it to bare TObject) and keep string/class-string
        // atoms as-is; TMixed is replaced with bare TObject (a usable placeholder — the
        // string alternative isn't worth widening a plain mixed into a union for).
        // TScalar excludes object entirely though, so it can only narrow to TString,
        // never TObject (bool|int|float|string can never be an object instance).
        "method_exists" | "property_exists" => {
            // A receiver that's neither object-like, string-like, mixed, nor
            // scalar (e.g. plain int/bool/array) can never be passed to
            // method_exists()/property_exists() — PHP 8 throws a TypeError for
            // such an argument regardless of which boolean the call returns,
            // so reaching EITHER branch already proves it was object|string.
            // Let the result go empty like every sibling is_*() arm (the
            // caller's set_narrowed/apply_prop_narrowed already marks
            // divergence on empty), instead of silently reverting to the
            // unnarrowed current type.
            let mut result = Type::empty();
            result.from_docblock = current.from_docblock;
            for t in &current.types {
                if t.is_object() || t.is_string() {
                    result.add_type(t.clone());
                } else if matches!(t, Atomic::TMixed) {
                    result.add_type(Atomic::TObject);
                } else if matches!(t, Atomic::TScalar) {
                    result.add_type(Atomic::TString);
                }
            }
            result
        }
        _ => return None,
    })
}

/// Whether `t` is provably excluded from `is_iterable()`/`is_countable()`'s
/// false branch: always true for the `array` atom (the one type both
/// functions are unconditionally true for), and true for a named-object atom
/// that already provably extends/implements `$interface` — such an atom can
/// never make `is_iterable()`/`is_countable()` false, so it can't survive
/// into the false branch. A `final` class that provably does NOT
/// extend/implement `$interface` is the opposite case (guaranteed to make
/// the check false) and must be kept, not excluded. A non-final or
/// unresolvable class is left alone (kept), matching this function's
/// conservative behavior before this class-hierarchy check existed
/// (stripping a non-final class here risks falsely excluding a legitimately
/// Countable/Traversable subclass).
pub(super) fn atom_excluded_from_is_iterable_or_countable(
    t: &Atomic,
    interface: &str,
    db: &dyn MirDatabase,
) -> bool {
    if t.is_array() {
        return true;
    }
    if let Atomic::TNamedObject { fqcn, .. }
    | Atomic::TSelf { fqcn }
    | Atomic::TStaticObject { fqcn }
    | Atomic::TParent { fqcn } = t
    {
        return crate::db::extends_or_implements(db, fqcn, interface);
    }
    false
}
