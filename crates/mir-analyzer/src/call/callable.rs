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
                // TKeyedArray marked as is_list is a numeric list, not a callable
                if *is_list {
                    return false;
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
