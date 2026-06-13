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
    let min = match arg_types.first() {
        Some(t) if is_non_empty_collection(t) => 1,
        _ => 0,
    };
    Some(Type::single(Atomic::TIntRange {
        min: Some(min),
        max: None,
    }))
}

/// Result type of `strlen($s)` / `mb_strlen($s)`: a byte/character length is
/// always `>= 0`, i.e. `int<0, max>`.
pub(crate) fn non_negative_int() -> Type {
    Type::single(Atomic::TIntRange {
        min: Some(0),
        max: None,
    })
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

    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
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
