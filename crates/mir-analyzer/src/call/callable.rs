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

/// Infer the return type of a string function that preserves non-emptiness:
/// `strtolower`, `strtoupper`, `mb_strtolower`, `mb_strtoupper`, `ucfirst`,
/// `lcfirst`, `ucwords`.
///
/// These functions can never return an empty string when given a non-empty
/// input (they only change casing, never remove characters). When the argument
/// is provably non-empty, we return `non-empty-string` instead of the stub's
/// plain `string`.
pub(crate) fn string_preserve_non_empty(arg_types: &[Type]) -> Option<Type> {
    let arg = arg_types.first()?;
    if is_non_empty_string(arg) {
        Some(Type::single(Atomic::TNonEmptyString))
    } else {
        None
    }
}

/// Infer the return type of `number_format()`.
///
/// `number_format()` always returns a non-empty string — even `number_format(0)`
/// returns "0". The stub declares `string`; we refine that here.
pub(crate) fn number_format_return_type() -> Type {
    Type::single(Atomic::TNonEmptyString)
}

/// Infer the return type of `str_repeat($input, $count)`.
///
/// When the input is provably non-empty AND the count is a positive literal,
/// the result is guaranteed non-empty. Falls through (returns `None`) for
/// the general case so the stub's `string` is used.
pub(crate) fn str_repeat_return_type(arg_types: &[Type]) -> Option<Type> {
    let input = arg_types.first()?;
    let count = arg_types.get(1)?;
    let count_is_positive = count.types.iter().any(|a| match a {
        Atomic::TLiteralInt(n) => *n >= 1,
        Atomic::TPositiveInt => true,
        Atomic::TIntRange { min, .. } => min.is_some_and(|m| m >= 1),
        _ => false,
    }) && count.types.iter().all(|a| match a {
        Atomic::TLiteralInt(n) => *n >= 1,
        Atomic::TPositiveInt => true,
        Atomic::TIntRange { min, .. } => min.is_some_and(|m| m >= 1),
        _ => false,
    });
    if count_is_positive && is_non_empty_string(input) {
        Some(Type::single(Atomic::TNonEmptyString))
    } else {
        None
    }
}

/// Infer the return type of `array_fill($start_index, $count, $value)`.
///
/// When the count argument is provably >= 1, the result is `non-empty-list<T>`
/// where T is the type of `$value`. Falls through to `None` otherwise so the
/// stub's generic `array` is used.
pub(crate) fn array_fill_return_type(arg_types: &[Type]) -> Option<Type> {
    let count = arg_types.get(1)?;
    let value = arg_types.get(2)?;
    let count_is_positive = !count.types.is_empty()
        && count.types.iter().all(|a| match a {
            Atomic::TLiteralInt(n) => *n >= 1,
            Atomic::TPositiveInt => true,
            Atomic::TIntRange { min, .. } => min.is_some_and(|m| m >= 1),
            _ => false,
        });
    if count_is_positive {
        Some(Type::single(Atomic::TNonEmptyList {
            value: Box::new(value.clone()),
        }))
    } else {
        None
    }
}

/// Infer the return type of `implode($separator, $array)` / `join($separator, $array)`.
///
/// When the array argument is a non-empty collection of non-empty strings,
/// the result is a `non-empty-string`. Falls through to `None` otherwise.
pub(crate) fn implode_return_type(arg_types: &[Type]) -> Option<Type> {
    // implode supports both 1-arg (array only) and 2-arg (separator, array) forms.
    let arr = if arg_types.len() == 1 {
        arg_types.first()?
    } else {
        arg_types.get(1)?
    };
    if !is_non_empty_collection(arr) {
        return None;
    }
    // Check that all elements of the array are non-empty strings.
    let all_elements_non_empty = arr.types.iter().all(|a| match a {
        Atomic::TNonEmptyList { value } | Atomic::TList { value } => is_non_empty_string(value),
        Atomic::TNonEmptyArray { value, .. } | Atomic::TArray { value, .. } => {
            is_non_empty_string(value)
        }
        Atomic::TKeyedArray { properties, .. } => {
            properties.values().all(|p| is_non_empty_string(&p.ty))
        }
        _ => false,
    });
    if all_elements_non_empty {
        Some(Type::single(Atomic::TNonEmptyString))
    } else {
        None
    }
}

/// Infer the return type of `str_split($string, $length)`.
///
/// When the string argument is provably non-empty, every chunk is non-empty
/// and there is at least one chunk, so the result is `non-empty-list<non-empty-string>`.
pub(crate) fn str_split_return_type(arg_types: &[Type]) -> Option<Type> {
    let s = arg_types.first()?;
    if is_non_empty_string(s) {
        Some(Type::single(Atomic::TNonEmptyList {
            value: Box::new(Type::single(Atomic::TNonEmptyString)),
        }))
    } else {
        None
    }
}

/// Infer the return type of `array_keys($array)`.
///
/// When the argument is a statically non-empty array, upgrades `list<K>` in
/// the stub's template-resolved return type to `non-empty-list<K>` so the key
/// type from Psalm-style template inference is preserved. Returns the stub
/// return unchanged when the source is not provably non-empty.
pub(crate) fn array_keys_return_type(arg_types: &[Type], return_ty: &Type) -> Type {
    let Some(arr) = arg_types.first() else {
        return return_ty.clone();
    };
    if !is_non_empty_collection(arr) {
        return return_ty.clone();
    }
    // Upgrade list<K> → non-empty-list<K> while keeping the stub's key type.
    let mut result = Type::empty();
    result.from_docblock = return_ty.from_docblock;
    for atomic in &return_ty.types {
        match atomic {
            Atomic::TList { value } => {
                result.add_type(Atomic::TNonEmptyList {
                    value: value.clone(),
                });
            }
            other => result.add_type(other.clone()),
        }
    }
    if result.is_empty() {
        return_ty.clone()
    } else {
        result
    }
}

/// Infer the return type of `array_reverse($array)`.
///
/// Preserves non-emptiness: a non-empty array reversed is still non-empty.
/// Uses the same value type as the source array.
pub(crate) fn array_reverse_return_type(arg_types: &[Type]) -> Option<Type> {
    let arr = arg_types.first()?;
    if arr.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(arr);
    if value.is_mixed() {
        return None;
    }
    let atomic = if is_non_empty_collection(arr) {
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

/// Returns true when a `sprintf` format string guarantees a non-empty result.
///
/// The result is non-empty when:
/// - the format string has any literal character that isn't consumed by a `%s`
///   specifier (literal prefix/suffix, or any non-`%s` specifier like `%d`, `%%`)
///
/// This is conservative: we return false for any format that cannot be proven
/// non-empty from the format string alone (e.g. pure `%s%s`).
fn sprintf_format_guarantees_non_empty(fmt: &str) -> bool {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            i += 1;
            if i >= bytes.len() {
                // Dangling `%` — treated as literal
                return true;
            }
            // Skip optional flags, width, precision
            while i < bytes.len()
                && matches!(
                    bytes[i],
                    b'+' | b'-' | b' ' | b'0'..=b'9' | b'.' | b'\'' | b'('
                )
            {
                i += 1;
            }
            if i >= bytes.len() {
                return true;
            }
            match bytes[i] {
                // `%%` is a literal `%` — always non-empty
                b'%' => return true,
                // `%s` can produce an empty string — skip without declaring non-empty
                b's' => {}
                // All other specifiers produce non-empty output
                _ => return true,
            }
            i += 1;
        } else {
            // Literal character outside any format specifier
            return true;
        }
    }
    false
}

/// Infer the return type of `sprintf($format, ...)`.
///
/// Infer the result type of `explode($separator, $string, $limit)`.
///
/// When the separator is provably non-empty, PHP always returns at least one
/// element (the whole subject string when the separator is not found). Upgrades
/// the array portion of `stub_return` to `non-empty-list<string>` while
/// preserving any `false` component (PHP 7.x stub). Returns `None` when the
/// separator's non-emptiness cannot be proven so the stub type is used as-is.
pub(crate) fn explode_return_type(arg_types: &[Type], stub_return: &Type) -> Option<Type> {
    let separator = arg_types.first()?;
    let sep_non_empty = separator.types.iter().any(|a| {
        matches!(a, Atomic::TNonEmptyString)
            || matches!(a, Atomic::TLiteralString(s) if !s.as_ref().is_empty())
    });
    if !sep_non_empty {
        return None;
    }
    let mut result = Type::single(Atomic::TNonEmptyList {
        value: Box::new(Type::single(Atomic::TString)),
    });
    // Preserve the `|false` that phpstorm-stubs emits for PHP < 8.0.
    if stub_return
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TFalse))
    {
        result.add_type(Atomic::TFalse);
    }
    Some(result)
}

/// When the format string is a literal and `sprintf_format_guarantees_non_empty`
/// is true, the result is `non-empty-string`. Falls through to `None` otherwise.
pub(crate) fn sprintf_return_type(arg_types: &[Type]) -> Option<Type> {
    let fmt_ty = arg_types.first()?;
    if fmt_ty.types.len() != 1 {
        return None;
    }
    let fmt = match &fmt_ty.types[0] {
        Atomic::TLiteralString(s) => s.as_ref(),
        _ => return None,
    };
    if sprintf_format_guarantees_non_empty(fmt) {
        Some(Type::single(Atomic::TNonEmptyString))
    } else {
        None
    }
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

    // Single-array mode: detect list-ness and non-emptiness from the source.
    // Multi-array (zip) mode: always produces array<int, T>.
    if arg_types.len() == 2 {
        let source = &arg_types[1];
        let src_is_list = !source.types.is_empty()
            && source
                .types
                .iter()
                .all(|a| matches!(a, Atomic::TList { .. } | Atomic::TNonEmptyList { .. }));
        let src_is_non_empty = !source.types.is_empty()
            && source.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
                )
            });
        let atom = match (src_is_list, src_is_non_empty) {
            (true, true) => Atomic::TNonEmptyList {
                value: Box::new(value),
            },
            (true, false) => Atomic::TList {
                value: Box::new(value),
            },
            (false, true) => {
                let (k, _) = crate::stmt::infer_foreach_types(source);
                let key = if k.is_mixed() { array_key_type() } else { k };
                Atomic::TNonEmptyArray {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
            (false, false) => {
                let (k, _) = crate::stmt::infer_foreach_types(source);
                let key = if k.is_mixed() { array_key_type() } else { k };
                Atomic::TArray {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
        };
        Some(Type::single(atom))
    } else {
        // Multi-array zip mode: integer-keyed, not a list (multi-array semantics).
        let src_is_non_empty = arg_types.get(1).is_some_and(|t| {
            !t.types.is_empty()
                && t.types.iter().all(|a| {
                    matches!(
                        a,
                        Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
                    )
                })
        });
        let key = Type::single(Atomic::TInt);
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

/// Infer the result type of `array_slice($array, $offset, $length, $preserve_keys)`.
///
/// When the source is a list type and `preserve_keys` is false (the default),
/// the result is re-indexed to start from 0 → returns `list<TValue>`.
/// When the source is a generic array, key and value types are preserved.
/// The result is always possibly-empty (we cannot know whether the slice is
/// non-empty without evaluating the offset/length at analysis time).
pub(crate) fn array_slice_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    // Check if preserve_keys is explicitly true (4th arg is TTrue or TBool).
    let preserve_keys = arg_types.get(3).is_some_and(|t| {
        t.types
            .iter()
            .any(|a| matches!(a, Atomic::TTrue | Atomic::TBool))
            && !t
                .types
                .iter()
                .any(|a| matches!(a, Atomic::TFalse | Atomic::TNull))
    });

    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }

    // When the source is a list and preserve_keys is false (default), the
    // result is also a list (integer keys are re-indexed from 0).
    let is_source_list = source
        .types
        .iter()
        .all(|a| matches!(a, Atomic::TList { .. } | Atomic::TNonEmptyList { .. }));

    if is_source_list && !preserve_keys {
        return Some(Type::single(Atomic::TList {
            value: Box::new(value),
        }));
    }

    // Generic array: preserve key type too.
    let (key, _) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() {
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

/// Infer the result type of `array_merge($arr1, $arr2, ...)`.
///
/// When ALL arguments are list types, the merged result is also a list (PHP
/// re-indexes integer keys from 0 for array_merge). The result is non-empty
/// if at least one argument is provably non-empty.
/// Falls back to `None` when any arg is a non-list array (string-keyed arrays
/// or mixed-key arrays) — the generic stub handles those cases.
pub(crate) fn infer_array_merge_return(arg_types: &[Type]) -> Option<Type> {
    if arg_types.is_empty() {
        return None;
    }
    // All args must be list types for us to produce a list result.
    let all_lists = arg_types.iter().all(|t| {
        !t.types.is_empty()
            && t.types
                .iter()
                .all(|a| matches!(a, Atomic::TList { .. } | Atomic::TNonEmptyList { .. }))
    });
    if !all_lists {
        return None;
    }
    // Compute the union of all element types.
    let mut value = Type::empty();
    for arg in arg_types {
        let (_, v) = crate::stmt::infer_foreach_types(arg);
        value.merge_with(&v);
    }
    if value.is_empty() || value.is_mixed() {
        return None;
    }
    // Non-empty if ANY arg is non-empty.
    let any_non_empty = arg_types.iter().any(is_non_empty_collection);
    let atomic = if any_non_empty {
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

/// Infer the return type of `array_unique($array)`.
///
/// `array_unique` preserves keys and drops duplicates; a non-empty input always
/// yields a non-empty result. List-ness is NOT preserved (keys can have gaps after
/// deduplication). Returns `None` to fall back to the stub for unknown inputs.
pub(crate) fn array_unique_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    let atomic = if is_non_empty_collection(source) {
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
    Some(Type::single(atomic))
}

/// Infer the return type of `range($start, $end, $step?)`.
///
/// When both bounds are integer literals (or single-value integer ranges), return
/// `non-empty-list<int<min, max>>`. Otherwise fall back to `None`.
/// `range()` always returns a list (re-indexed from 0) and is always non-empty.
pub(crate) fn range_return_type(arg_types: &[Type]) -> Option<Type> {
    let start = arg_types.first()?;
    let end = arg_types.get(1)?;

    fn single_int_bound(t: &Type) -> Option<i64> {
        if t.types.len() != 1 {
            return None;
        }
        match &t.types[0] {
            Atomic::TLiteralInt(n) => Some(*n),
            Atomic::TIntRange {
                min: Some(lo),
                max: Some(hi),
            } if lo == hi => Some(*lo),
            _ => None,
        }
    }

    let lo = single_int_bound(start)?;
    let hi = single_int_bound(end)?;
    let (range_min, range_max) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    let elem = Atomic::TIntRange {
        min: Some(range_min),
        max: Some(range_max),
    };
    Some(Type::single(Atomic::TNonEmptyList {
        value: Box::new(Type::single(elem)),
    }))
}

/// Infer the return type of `array_key_first($array)` / `array_key_last($array)`.
///
/// For non-empty collections the result is always `string|int` (never null).
/// For `list` / `non-empty-list` it is always `int`.
/// Returns `None` to fall back to the stub when the collection is possibly empty.
pub(crate) fn array_key_first_last_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() || !is_non_empty_collection(source) {
        return None;
    }
    let all_list = source
        .types
        .iter()
        .all(|a| matches!(a, Atomic::TNonEmptyList { .. } | Atomic::TList { .. }));
    if all_list {
        Some(Type::single(Atomic::TInt))
    } else {
        let mut ty = Type::single(Atomic::TInt);
        ty.add_type(Atomic::TString);
        Some(ty)
    }
}

/// Infer the return type of `array_pop($array)` / `array_shift($array)`.
///
/// When the collection element type is known and the collection is provably
/// non-empty, the return is `T` (not `T|null`). When the collection is typed
/// but possibly empty, returns `T|null`. Falls back to `None` (→ stub) for
/// unknown / `mixed` sources.
pub(crate) fn array_pop_shift_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    if is_non_empty_collection(source) {
        Some(value)
    } else {
        let mut ty = value;
        ty.add_type(Atomic::TNull);
        Some(ty)
    }
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

/// Infer the by-ref array type after an in-place sort function.
///
/// All sort functions preserve element types. Re-indexing sorts (`sort`, `rsort`, `usort`,
/// `shuffle`) also re-index integer keys from 0, making the result a `list<T>`.
/// Key-preserving sorts (`asort`, `arsort`, `ksort`, `krsort`, `uasort`, `uksort`) leave
/// key types unchanged — so the original type is returned as-is.
///
/// The main benefit over the stub's generic `array` is that element types are not lost.
pub(crate) fn sort_byref_type(arr: &Type, reindex: bool) -> Type {
    if arr.is_mixed() {
        return arr.clone();
    }
    if !reindex {
        return arr.clone();
    }
    let (_, value) = crate::stmt::infer_foreach_types(arr);
    if value.is_mixed() {
        return arr.clone();
    }
    let atom = if is_non_empty_collection(arr) {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Type::single(atom)
}

/// Infer the return type of `array_search($needle, $haystack)`.
///
/// The stub returns `string|int|false`. When the haystack has a known key type
/// (e.g. `list` or `array<string, …>`), the success case is narrowed to that key
/// type, reducing the union from `string|int|false` to `key_type|false`.
pub(crate) fn array_search_return_type(arg_types: &[Type]) -> Option<Type> {
    let haystack = arg_types.get(1)?;
    if haystack.is_mixed() {
        return None;
    }
    let (key, _) = crate::stmt::infer_foreach_types(haystack);
    if key.is_mixed() {
        return None;
    }
    let mut result = key;
    result.add_type(Atomic::TFalse);
    Some(result)
}

/// Infer the return type of `preg_split($pattern, $subject, $limit?, $flags?)`.
///
/// `preg_split` always produces at least one string unless `PREG_SPLIT_NO_EMPTY` (value 1) is
/// set and every part is empty. When `$flags` is 0 (default) or absent, the result is
/// guaranteed non-empty: `non-empty-list<string>|false`. When `PREG_SPLIT_OFFSET_CAPTURE` (4)
/// is set each element is an array, which we don't model — falls back to stub.
/// Falls back to stub `array|false` for other flag combinations.
pub(crate) fn preg_split_return_type(arg_types: &[Type]) -> Option<Type> {
    // flags is the 4th argument (index 3). When absent the default is 0.
    let flags_ty = arg_types.get(3);
    let flags_zero = match flags_ty {
        None => true,
        Some(t) => t.types.len() == 1 && matches!(t.types.first(), Some(Atomic::TLiteralInt(0))),
    };
    if !flags_zero {
        return None;
    }
    // No PREG_SPLIT_NO_EMPTY → always at least one part.
    let mut result = Type::single(Atomic::TNonEmptyList {
        value: Box::new(Type::single(Atomic::TString)),
    });
    result.add_type(Atomic::TFalse);
    Some(result)
}

/// Helper: extract a readable function name from union for diagnostic output.
fn callback_name_for_diagnostic(callback_ty: &Type) -> String {
    if let Some(Atomic::TLiteralString(fn_name)) = callback_ty.types.first() {
        fn_name.to_string()
    } else {
        "(closure)".to_string()
    }
}
