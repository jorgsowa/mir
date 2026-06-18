use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::atomic::FnParam;
use mir_types::{Atomic, Type};

use crate::expr::ExpressionAnalyzer;

/// Simple param info for arity checking (works with both codebase and types FnParam)
#[derive(Clone)]
pub(crate) struct ParamInfo {
    pub(crate) is_optional: bool,
    pub(crate) is_variadic: bool,
}

/// Extract callable parameter list for arity checking from a union when it can be determined statically:
/// - TClosure: return params directly
/// - TLiteralString: resolve to function only if from documented type annotation (issue #5)
/// - TIntersection: check parts for callable/closure types
/// - Everything else: None (param list is unknown at compile time)
pub(crate) fn extract_callable_params(
    union: &Type,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Vec<ParamInfo>> {
    // If the union contains a bare callable (unknown arity), we cannot determine
    // arity statically — bail out to avoid false positives from sibling TClosure members.
    if union
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TCallable { params: None, .. }))
    {
        return None;
    }

    for atomic in &union.types {
        match atomic {
            Atomic::TClosure { params, .. } => {
                return Some(
                    params
                        .iter()
                        .map(|p| ParamInfo {
                            is_optional: p.is_optional,
                            is_variadic: p.is_variadic,
                        })
                        .collect(),
                );
            }
            Atomic::TLiteralString(fn_name) => {
                if fn_name.is_empty() {
                    continue;
                }

                // Try to resolve the function name. Only return params if found (don't fail for unknown strings).
                // This allows arity checking for both documented callables and literal function names in code.
                let here = crate::db::Fqcn::from_str(ea.db, fn_name.as_ref());
                if let Some(f) = crate::db::find_function(ea.db, here) {
                    return Some(
                        f.params
                            .iter()
                            .map(|p| ParamInfo {
                                is_optional: p.is_optional,
                                is_variadic: p.is_variadic,
                            })
                            .collect(),
                    );
                }
            }
            Atomic::TIntersection { parts } => {
                for part in parts.iter() {
                    if let Some(params) = extract_callable_params(part, ea) {
                        return Some(params);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a union type is valid for use as a callable.
///
/// Returns false only for types that are clearly NOT callable at runtime:
/// - TList<T>, TNonEmptyList<T> — sequential arrays, never callable
/// - TArray, TNonEmptyArray — general arrays, not valid callables
/// - TKeyedArray marked as is_list — known to be a numeric list, not callable
///
/// Returns true (safe fallback) for:
/// - TClosure, TCallable, TString, TLiteralString, TNull
/// - TKeyedArray NOT marked as is_list (could be [$obj, 'method'] form)
/// - Unknown/other types
pub(crate) fn is_valid_callable_type(union: &Type) -> bool {
    for atomic in &union.types {
        match atomic {
            Atomic::TClosure { .. }
            | Atomic::TCallable { .. }
            | Atomic::TString
            | Atomic::TNonEmptyString
            | Atomic::TLiteralString(_)
            | Atomic::TNull => {
                return true;
            }
            Atomic::TKeyedArray { is_list, .. } => {
                // A numeric-keyed list is only callable in the `[$obj, 'method']`
                // / `['Class', 'method']` 2-element form.
                if *is_list {
                    return is_callable_array_pair(union);
                }
                // Otherwise it could be [obj, 'method'] form, accept it
                return true;
            }
            Atomic::TList { .. }
            | Atomic::TNonEmptyList { .. }
            | Atomic::TArray { .. }
            | Atomic::TNonEmptyArray { .. } => {
                return false;
            }
            _ => {
                continue;
            }
        }
    }
    true
}

/// True if `arg` is the array-callable pair form `[$obj, 'method']` /
/// `['Class', 'method']` — a 2-element shape (keys 0 and 1) whose first element
/// is an object / class-string / string and whose second element is a string.
/// PHP accepts this anywhere a `callable` is expected, including the `is_list`
/// shape produced by an `[$this, 'm']` literal.
pub(crate) fn is_callable_array_pair(arg: &Type) -> bool {
    arg.types.iter().any(|a| {
        let Atomic::TKeyedArray { properties, .. } = a else {
            return false;
        };
        if properties.len() != 2 {
            return false;
        }
        let first = properties.get(&mir_types::atomic::ArrayKey::Int(0));
        let second = properties.get(&mir_types::atomic::ArrayKey::Int(1));
        let (Some(first), Some(second)) = (first, second) else {
            return false;
        };
        // The first element must be an object (`[$this, 'm']`) or a
        // class-string. A plain/literal string is NOT accepted here: a literal
        // like `["one", "two"]` is only callable if "one" names a real class,
        // which this db-less predicate can't verify — leave that to the regular
        // checks so a non-class string pair is still rejected.
        let first_ok = first.ty.contains(|t| {
            matches!(
                t,
                Atomic::TNamedObject { .. }
                    | Atomic::TObject
                    | Atomic::TSelf { .. }
                    | Atomic::TStaticObject { .. }
                    | Atomic::TClassString(_)
                    | Atomic::TMixed
            )
        });
        let second_ok = second.ty.contains(|t| {
            matches!(
                t,
                Atomic::TString | Atomic::TNonEmptyString | Atomic::TLiteralString(_)
            )
        });
        first_ok && second_ok
    })
}

/// Validate array_map callback: arity must match the number of arrays passed.
/// array_map(callback, array1, array2, ...) → callback receives one element from each array.
pub(crate) fn check_array_map_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    arg_spans: &[Span],
) {
    if arg_types.is_empty() || arg_spans.is_empty() {
        return;
    }

    let callback_ty = &arg_types[0];
    let callback_span = arg_spans[0];

    if !is_valid_callable_type(callback_ty) {
        ea.emit(
            IssueKind::InvalidArgument {
                param: "callback".to_string(),
                fn_name: "array_map".to_string(),
                expected: "callable".to_string(),
                actual: callback_ty.to_string(),
            },
            Severity::Error,
            callback_span,
        );
        return;
    }

    if arg_types.len() > 1 {
        validate_callback_arity(ea, callback_ty, callback_span, arg_types.len() - 1);
    }
}

/// Generic callback arity validation for any function.
/// Emits TooFewArguments or TooManyArguments if the callback doesn't match expected arity.
fn validate_callback_arity(
    ea: &mut ExpressionAnalyzer<'_>,
    callback_ty: &Type,
    callback_span: Span,
    expected_arity: usize,
) {
    if let Some(params) = extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let has_variadic = params.iter().any(|p| p.is_variadic);
        let max_params = params.len();

        if required_count > expected_arity {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooFewArguments {
                    fn_name,
                    expected: required_count,
                    actual: expected_arity,
                },
                Severity::Error,
                callback_span,
            );
        } else if !has_variadic && max_params < expected_arity {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooManyArguments {
                    fn_name,
                    expected: max_params,
                    actual: expected_arity,
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

// PHP array_filter mode constants
const ARRAY_FILTER_USE_BOTH: i64 = 1; // pass value and key to callback
const ARRAY_FILTER_USE_KEY: i64 = 2; // pass only key to callback

/// Validate array_filter callback.
/// Expected arity depends on mode (arg_types[2]):
/// - ARRAY_FILTER_USE_BOTH (1): 2 args (value, key)
/// - ARRAY_FILTER_USE_KEY (2): 1 arg (key)
/// - else/missing: 1 arg (value)
pub(crate) fn check_array_filter_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    arg_spans: &[Span],
) {
    if arg_types.len() < 2 || arg_spans.len() < 2 {
        return;
    }

    let callback_ty = &arg_types[1];
    let callback_span = arg_spans[1];

    if !is_valid_callable_type(callback_ty) {
        ea.emit(
            IssueKind::InvalidArgument {
                param: "callback".to_string(),
                fn_name: "array_filter".to_string(),
                expected: "callable".to_string(),
                actual: callback_ty.to_string(),
            },
            Severity::Error,
            callback_span,
        );
        return;
    }

    let expected_arity = if arg_types.len() > 2 {
        match arg_types[2].types.first() {
            Some(Atomic::TLiteralInt(ARRAY_FILTER_USE_BOTH)) => 2,
            Some(Atomic::TLiteralInt(ARRAY_FILTER_USE_KEY)) => 1,
            _ => 1,
        }
    } else {
        1
    };

    if let Some(params) = extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let has_variadic = params.iter().any(|p| p.is_variadic);
        let max_params = params.len();

        if required_count > expected_arity || (!has_variadic && max_params < expected_arity) {
            let actual_count = if has_variadic {
                required_count
            } else {
                max_params
            };
            let expected_plural = if expected_arity == 1 { "" } else { "s" };
            let actual_plural = if actual_count == 1 { "" } else { "s" };
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: "array_filter".to_string(),
                    expected: format!(
                        "callable accepting {} argument{}",
                        expected_arity, expected_plural
                    ),
                    actual: format!(
                        "callable accepting {} argument{}",
                        actual_count, actual_plural
                    ),
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Extract the return type a callable union resolves to, if statically known:
/// - `TClosure` / `TCallable` carry their return type directly.
/// - `TLiteralString` is resolved as a function name and its declared return is used.
/// - `TIntersection` is searched part by part.
///
/// Returns `None` when the callback's return type cannot be determined (bare
/// `callable`, unknown string, `null`, …) so callers can fall back to the
/// generic stub return type rather than inventing a wrong element type.
fn callable_return_type(union: &Type, ea: &ExpressionAnalyzer<'_>) -> Option<Type> {
    for atomic in &union.types {
        match atomic {
            Atomic::TClosure { return_type, .. } => return Some((**return_type).clone()),
            Atomic::TCallable {
                return_type: Some(rt),
                ..
            } => return Some((**rt).clone()),
            Atomic::TLiteralString(fn_name) if !fn_name.is_empty() => {
                let here = crate::db::Fqcn::from_str(ea.db, fn_name.as_ref());
                if let Some(f) = crate::db::find_function(ea.db, here) {
                    if let Some(rt) = &f.return_type {
                        return Some((**rt).clone());
                    }
                }
            }
            Atomic::TIntersection { parts } => {
                for part in parts.iter() {
                    if let Some(rt) = callable_return_type(part, ea) {
                        return Some(rt);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether every member of `ty` is a statically non-empty collection, so a
/// `count()` over it is guaranteed `>= 1`. Conservative: any member that could
/// be empty (or a `Countable` object of unknown size) yields `false`.
fn is_non_empty_collection(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty.types.iter().all(|a| match a {
            Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. } => true,
            // A keyed array (shape) is non-empty iff it declares a required key.
            Atomic::TKeyedArray { properties, .. } => properties.values().any(|p| !p.optional),
            _ => false,
        })
}

/// Result type of `count($value)` / `sizeof($value)`: the integer count is
/// always `>= 0`, and `>= 1` when the argument is a statically non-empty
/// collection. Modeling this as `int<0, max>` / `int<1, max>` is the faithful
/// type and feeds range-aware arithmetic at use sites.
pub(crate) fn count_return_type(arg_types: &[Type]) -> Option<Type> {
    // Fast path: single sealed keyed-array shape with an exact known count.
    if let Some(ty) = arg_types.first() {
        if ty.types.len() == 1 {
            if let Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } = &ty.types[0]
            {
                if !is_open && properties.values().all(|p| !p.optional) {
                    return Some(Type::single(Atomic::TLiteralInt(properties.len() as i64)));
                }
            }
        }
    }
    let min = match arg_types.first() {
        Some(t) if is_non_empty_collection(t) => 1,
        _ => 0,
    };
    Some(Type::single(Atomic::TIntRange {
        min: Some(min),
        max: None,
    }))
}

/// Result type of `strlen($s)` / `mb_strlen($s)`: `int<0, max>` normally,
/// `int<1, max>` when the argument is statically non-empty, or a literal
/// `int<n, n>` when the argument is a known literal string.
pub(crate) fn strlen_return_type(arg_types: &[Type]) -> Type {
    // Fast path: single literal string with a statically known length.
    if let Some(ty) = arg_types.first() {
        if ty.types.len() == 1 {
            if let Atomic::TLiteralString(s) = &ty.types[0] {
                return Type::single(Atomic::TLiteralInt(s.len() as i64));
            }
        }
    }
    let min = match arg_types.first() {
        Some(t) if is_non_empty_string(t) => 1,
        _ => 0,
    };
    Type::single(Atomic::TIntRange {
        min: Some(min),
        max: None,
    })
}

fn is_non_empty_string(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TNonEmptyString
                    | Atomic::TClassString(_)
                    | Atomic::TInterfaceString
                    | Atomic::TEnumString
                    | Atomic::TTraitString
            ) || matches!(a, Atomic::TLiteralString(s) if !s.is_empty())
        })
}

/// Infer the return type of `abs($num)`.
///
/// PHP semantics: abs always returns a non-negative value with the same type
/// (int → int, float → float). When the argument is a known int subtype or
/// range, we can tighten the result accordingly.
pub(crate) fn abs_return_type(arg_types: &[Type]) -> Option<Type> {
    let arg = arg_types.first()?;
    // Only engage when the argument is purely integer (no float components).
    // A float argument returns float per PHP semantics; we leave that to the stub.
    let all_int = arg.types.iter().all(|a| {
        matches!(
            a,
            Atomic::TInt
                | Atomic::TLiteralInt(_)
                | Atomic::TPositiveInt
                | Atomic::TNonNegativeInt
                | Atomic::TNegativeInt
                | Atomic::TIntRange { .. }
        )
    });
    if !all_int || arg.types.is_empty() {
        return None;
    }

    let mut result = Type::empty();
    for a in &arg.types {
        let atom = match a {
            // Already non-negative — abs is identity.
            Atomic::TPositiveInt | Atomic::TNonNegativeInt => a.clone(),
            // Any int → non-negative-int.
            Atomic::TInt => Atomic::TNonNegativeInt,
            // Literal fold: use checked_neg to avoid i64::MIN overflow.
            Atomic::TLiteralInt(n) => {
                let abs = if *n >= 0 {
                    *n
                } else {
                    n.checked_neg().unwrap_or(i64::MAX)
                };
                Atomic::TLiteralInt(abs)
            }
            // negative-int is int<-∞, -1>: abs gives int<1, ∞> = positive-int.
            Atomic::TNegativeInt => Atomic::TPositiveInt,
            Atomic::TIntRange { min, max } => {
                let (lo, hi) = (*min, *max);
                let lo_is_nn = lo.is_some_and(|m| m >= 0);
                let hi_is_np = hi.is_some_and(|m| m <= 0);
                // Safe abs of a bound: None → None, saturate at i64::MAX on overflow.
                let abs_bound = |v: Option<i64>| {
                    v.map(|n| {
                        if n >= 0 {
                            n
                        } else {
                            n.checked_neg().unwrap_or(i64::MAX)
                        }
                    })
                };
                if lo_is_nn {
                    // Range is entirely non-negative: abs is identity.
                    a.clone()
                } else if hi_is_np {
                    // Range is entirely non-positive: abs flips and negates.
                    Atomic::TIntRange {
                        min: abs_bound(hi),
                        max: abs_bound(lo),
                    }
                } else {
                    // Mixed range: result is int<0, max(|lo|, hi)>.
                    let new_max = match (abs_bound(lo), hi) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        _ => None, // unbounded in at least one direction
                    };
                    Atomic::TIntRange {
                        min: Some(0),
                        max: new_max,
                    }
                }
            }
            _ => return None,
        };
        result.add_type(atom);
    }
    Some(result)
}

/// Infer the return type of `intdiv($num1, $num2)`.
///
/// PHP semantics: intdiv returns the integer quotient (truncated toward zero).
/// When the dividend is non-negative and the divisor is positive, the result
/// is also non-negative. If both bounds are known we can narrow further.
pub(crate) fn intdiv_return_type(arg_types: &[Type]) -> Option<Type> {
    let (num1_ty, num2_ty) = (arg_types.first()?, arg_types.get(1)?);

    // Extract bounds from the first argument.
    let (n1_min, n1_max) = int_type_bounds(num1_ty)?;
    // Extract bounds from the divisor — only engage when divisor is positive.
    let (n2_min, _n2_max) = int_type_bounds(num2_ty)?;

    // Only infer when dividend is non-negative and divisor is strictly positive
    // to keep the logic simple and avoid dealing with rounding toward zero for
    // negative operands. Mixed-sign or zero-divisor cases fall through to `int`.
    let dividend_nn = n1_min.is_some_and(|m| m >= 0);
    let divisor_pos = n2_min.is_some_and(|m| m > 0);
    if !dividend_nn || !divisor_pos {
        return None;
    }

    // Result is in [0, n1_max / n2_min] when both are known; [0, ∞) otherwise.
    let new_max = match (n1_max, n2_min) {
        (Some(hi), Some(lo)) => hi.checked_div(lo),
        _ => None,
    };
    let atom = match (Some(0i64), new_max) {
        (Some(0), None) => Atomic::TNonNegativeInt,
        (Some(1), None) => Atomic::TPositiveInt,
        (min, max) => Atomic::TIntRange { min, max },
    };
    Some(Type::single(atom))
}

/// Extract (min, max) int bounds from a type that consists entirely of int subtypes.
/// Returns `None` when the type contains non-integer components.
fn int_type_bounds(ty: &Type) -> Option<(Option<i64>, Option<i64>)> {
    if ty.types.is_empty() {
        return None;
    }
    let mut min: Option<i64> = Some(i64::MAX);
    let mut max: Option<i64> = Some(i64::MIN);
    for a in &ty.types {
        let (lo, hi) = match a {
            Atomic::TLiteralInt(n) => (Some(*n), Some(*n)),
            Atomic::TIntRange { min, max } => (*min, *max),
            Atomic::TPositiveInt => (Some(1), None),
            Atomic::TNonNegativeInt => (Some(0), None),
            Atomic::TNegativeInt => (None, Some(-1)),
            Atomic::TInt => (None, None),
            _ => return None,
        };
        min = match (min, lo) {
            (Some(m), Some(l)) => Some(m.min(l)),
            _ => None,
        };
        max = match (max, hi) {
            (Some(m), Some(h)) => Some(m.max(h)),
            _ => None,
        };
    }
    Some((min, max))
}

/// Infer the return type of `min($a, $b, ...)` when all arguments are purely integer.
///
/// For `min(a, b)`:
/// - result_min = min(a_min, b_min) — the smallest possible value we could see
/// - result_max = min(a_max, b_max) — the smallest of the upper bounds
pub(crate) fn min_return_type(arg_types: &[Type]) -> Option<Type> {
    if arg_types.is_empty() {
        return None;
    }
    // Only engage when every argument is purely integer.
    let bounds: Vec<(Option<i64>, Option<i64>)> = arg_types
        .iter()
        .map(int_type_bounds)
        .collect::<Option<_>>()?;
    let result_min = bounds.iter().fold(None::<Option<i64>>, |acc, (lo, _)| {
        Some(match (acc, lo) {
            (None, v) => *v,
            (Some(Some(a)), Some(b)) => Some(a.min(*b)),
            _ => None,
        })
    })?;
    let result_max = bounds.iter().fold(None::<Option<i64>>, |acc, (_, hi)| {
        Some(match (acc, hi) {
            (None, v) => *v,
            (Some(Some(a)), Some(b)) => Some(a.min(*b)),
            (Some(None), Some(b)) => Some(*b),
            (Some(Some(a)), None) => Some(a),
            _ => None,
        })
    })?;
    Some(Type::single(make_int_range_atom(result_min, result_max)))
}

/// Infer the return type of `max($a, $b, ...)` when all arguments are purely integer.
///
/// For `max(a, b)`:
/// - result_min = max(a_min, b_min) — the largest of the lower bounds
/// - result_max = max(a_max, b_max) — the largest possible value we could see
pub(crate) fn max_return_type(arg_types: &[Type]) -> Option<Type> {
    if arg_types.is_empty() {
        return None;
    }
    let bounds: Vec<(Option<i64>, Option<i64>)> = arg_types
        .iter()
        .map(int_type_bounds)
        .collect::<Option<_>>()?;
    let result_min = bounds.iter().fold(None::<Option<i64>>, |acc, (lo, _)| {
        Some(match (acc, lo) {
            (None, v) => *v,
            (Some(Some(a)), Some(b)) => Some(a.max(*b)),
            (Some(None), Some(b)) => Some(*b),
            (Some(Some(a)), None) => Some(a),
            _ => None,
        })
    })?;
    let result_max = bounds.iter().fold(None::<Option<i64>>, |acc, (_, hi)| {
        Some(match (acc, hi) {
            (None, v) => *v,
            (Some(Some(a)), Some(b)) => Some(a.max(*b)),
            _ => None,
        })
    })?;
    Some(Type::single(make_int_range_atom(result_min, result_max)))
}

/// Canonicalise (min, max) int bounds into the most specific int Atomic.
fn make_int_range_atom(min: Option<i64>, max: Option<i64>) -> Atomic {
    match (min, max) {
        (Some(1), None) => Atomic::TPositiveInt,
        (Some(0), None) => Atomic::TNonNegativeInt,
        (None, Some(-1)) => Atomic::TNegativeInt,
        (None, None) => Atomic::TInt,
        (min, max) => Atomic::TIntRange { min, max },
    }
}

/// Infer the return type of `rand($min, $max)` / `mt_rand($min, $max)` /
/// `random_int($min, $max)` when both bounds are known integer literals.
///
/// With no arguments, `rand()` / `mt_rand()` return an unspecified int — fall
/// through to the stub. With two literal bounds, narrow to `int<min, max>`.
pub(crate) fn rand_return_type(arg_types: &[Type]) -> Option<Type> {
    let (min_ty, max_ty) = (arg_types.first()?, arg_types.get(1)?);
    let extract_literal = |ty: &Type| {
        if ty.types.len() == 1 {
            if let Atomic::TLiteralInt(n) = ty.types[0] {
                return Some(n);
            }
        }
        None
    };
    let lo = extract_literal(min_ty)?;
    let hi = extract_literal(max_ty)?;
    if lo > hi {
        return None; // degenerate — let stub handle it
    }
    Some(Type::single(make_int_range_atom(Some(lo), Some(hi))))
}

/// The default PHP array-key type, `int|string`, used when a source array's
/// key type cannot be determined more precisely.
fn array_key_type() -> Type {
    let mut k = Type::single(Atomic::TInt);
    k.add_type(Atomic::TString);
    k
}

/// Infer the result type of `array_map($callback, $array, ...)`.
///
/// PHP semantics: `array_map` applies `$callback` to each element and returns
/// an array of the callback's return values. With a single source array the
/// keys are preserved; with multiple arrays the result is re-indexed with
/// integer keys. A `null` callback (zip mode) is not modeled — we return
/// `None` so the generic stub `array` return is kept.
///
/// Returns `None` when the callback return type is unknown, so the caller falls
/// back to the stub return type instead of fabricating `array<…, mixed>`.
pub(crate) fn infer_array_map_return(
    ea: &ExpressionAnalyzer<'_>,
    arg_types: &[Type],
) -> Option<Type> {
    let callback = arg_types.first()?;
    // `array_map(null, ...)` (zip mode) and other non-callable first args are
    // out of scope; only proceed for a genuinely callable first argument.
    if callback.types.iter().any(|a| matches!(a, Atomic::TNull)) {
        return None;
    }
    let value = callable_return_type(callback, ea)?;
    // A `void`/`never` callback is degenerate (the runtime fills `null`); don't
    // fabricate an `array<…, void>` element type that would surface dubious
    // downstream diagnostics — keep the generic stub `array`.
    if value
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TVoid | Atomic::TNever))
    {
        return None;
    }

    // Key type: preserved from the single source array; integer-keyed when
    // multiple arrays are zipped together.
    let key = if arg_types.len() == 2 {
        let (k, _) = crate::stmt::infer_foreach_types(&arg_types[1]);
        if k.is_mixed() {
            array_key_type()
        } else {
            k
        }
    } else {
        Type::single(Atomic::TInt)
    };

    // Preserve the non-empty property: array_map on a non-empty input is also non-empty.
    let src_is_non_empty = arg_types.get(1).is_some_and(|t| {
        !t.types.is_empty()
            && t.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
                )
            })
    });
    let atom = if src_is_non_empty {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    };
    Some(Type::single(atom))
}

/// Infer the result type of `array_filter($array, $callback?, ...)`.
///
/// Filtering never changes the element types — it only removes entries — so the
/// result carries the source array's key and value types (made possibly-empty;
/// list-ness is dropped because filtering can leave gaps). Returns `None` when
/// the source element types are unknown so the generic stub `array` is kept.
pub(crate) fn infer_array_filter_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
}

/// Infer the result type of `array_values($array)`.
///
/// Re-indexing produces a `list<TValue>` (or `non-empty-list<TValue>` when the
/// source is provably non-empty). Returns `None` when element types are unknown
/// so the generic stub `list<TValue>` binding falls back to `list<mixed>`.
pub(crate) fn infer_array_values_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    let atomic = if is_non_empty_collection(source) {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Returns `(callback_arg_index, min_required_arity)` for built-in functions that enforce a
/// minimum callback arity via `check_min_arity_callback`. Functions with more complex rules
/// (array_map, array_filter) use their own specialized handlers instead.
pub(crate) fn callback_min_arity_spec(fn_name: &str) -> Option<(usize, usize)> {
    match fn_name {
        "array_reduce" => Some((1, 2)),
        "usort" | "uasort" | "uksort" => Some((1, 2)),
        "array_walk" | "array_walk_recursive" => Some((1, 1)),
        _ => None,
    }
}

/// Validate a callback argument against a minimum required arity.
pub(crate) fn check_min_arity_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    fn_name: &str,
    callback_idx: usize,
    min_arity: usize,
    arg_types: &[Type],
    arg_spans: &[Span],
) {
    if arg_types.len() <= callback_idx || arg_spans.len() <= callback_idx {
        return;
    }

    let callback_ty = &arg_types[callback_idx];
    let callback_span = arg_spans[callback_idx];

    if !is_valid_callable_type(callback_ty) {
        ea.emit(
            IssueKind::InvalidArgument {
                param: "callback".to_string(),
                fn_name: fn_name.to_string(),
                expected: "callable".to_string(),
                actual: callback_ty.to_string(),
            },
            Severity::Error,
            callback_span,
        );
        return;
    }

    if let Some(params) = extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        if required_count < min_arity {
            let expected_plural = if min_arity == 1 { "" } else { "s" };
            let actual_plural = if required_count == 1 { "" } else { "s" };
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: fn_name.to_string(),
                    expected: format!(
                        "callable accepting at least {} argument{}",
                        min_arity, expected_plural
                    ),
                    actual: format!(
                        "callable accepting {} argument{}",
                        required_count, actual_plural
                    ),
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Validate a callback argument against a typed callable parameter (e.g., callable(str,str,str):bool).
/// Emits InvalidArgument if the provided callable has more required params than expected.
pub(crate) fn check_typed_callable_arg(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_ty: &Type,
    expected_params: &[FnParam],
    arg_span: Span,
) {
    if let Some(actual_params) = extract_callable_params(arg_ty, ea) {
        let expected_required = expected_params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let actual_required = actual_params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();

        if actual_required > expected_required {
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: "typed_callable".to_string(),
                    expected: format!("callable with {} required parameter(s)", expected_required),
                    actual: format!("callable with {} required parameter(s)", actual_required),
                },
                Severity::Error,
                arg_span,
            );
        }
    }
}

/// Helper: extract a readable function name from union for diagnostic output.
fn callback_name_for_diagnostic(callback_ty: &Type) -> String {
    if let Some(Atomic::TLiteralString(fn_name)) = callback_ty.types.first() {
        fn_name.to_string()
    } else {
        "(closure)".to_string()
    }
}
