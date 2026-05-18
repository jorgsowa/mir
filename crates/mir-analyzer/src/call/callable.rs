use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::atomic::FnParam;
use mir_types::{Atomic, Union};

use crate::expr::ExpressionAnalyzer;

/// Simple param info for arity checking (works with both codebase and types FnParam)
#[derive(Clone)]
pub(crate) struct ParamInfo {
    pub(crate) is_optional: bool,
    pub(crate) is_variadic: bool,
}

/// Extract callable parameter list for arity checking from a union when it can be determined statically:
/// - TClosure: return params directly
/// - TLiteralString: resolve to function node and return its params
/// - TIntersection: check parts for callable/closure types
/// - Everything else: None (param list is unknown at compile time)
pub(crate) fn extract_callable_params(
    union: &Union,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<Vec<ParamInfo>> {
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
                let here = crate::db::Fqcn::new(ea.db, fn_name.clone());
                let params: Option<Vec<ParamInfo>> = crate::db::find_function(ea.db, here)
                    .map(|f| {
                        f.params
                            .iter()
                            .map(|p| ParamInfo {
                                is_optional: p.is_optional,
                                is_variadic: p.is_variadic,
                            })
                            .collect()
                    })
                    .or_else(|| {
                        ea.db
                            .lookup_function_node(fn_name.as_ref())
                            .filter(|n| n.active(ea.db))
                            .map(|node| {
                                node.params(ea.db)
                                    .iter()
                                    .map(|p| ParamInfo {
                                        is_optional: p.is_optional,
                                        is_variadic: p.is_variadic,
                                    })
                                    .collect()
                            })
                    });
                if let Some(params) = params {
                    return Some(params);
                }
            }
            Atomic::TIntersection { parts } => {
                for part in parts {
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
pub(crate) fn is_valid_callable_type(union: &Union) -> bool {
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

/// Validate array_map callback: arity must be 1 (element arg).
/// Emits InvalidArgument if callback is not valid callable.
/// Emits TooFewArguments/TooManyArguments if callback arity doesn't match.
pub(crate) fn check_array_map_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Union],
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

    if let Some(params) = extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let has_variadic = params.iter().any(|p| p.is_variadic);
        let max_params = params.len();

        if required_count > 1 {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooFewArguments {
                    fn_name,
                    expected: required_count,
                    actual: 1,
                },
                Severity::Error,
                callback_span,
            );
        } else if !has_variadic && max_params == 0 {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooManyArguments {
                    fn_name,
                    expected: 0,
                    actual: 1,
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Validate array_filter callback.
/// Expected arity depends on mode (arg_types[2]):
/// - TLiteralInt(1) ARRAY_FILTER_USE_BOTH: 2 args (value, key)
/// - TLiteralInt(2) ARRAY_FILTER_USE_KEY: 1 arg (key)
/// - else/missing: 1 arg (value)
pub(crate) fn check_array_filter_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Union],
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
            Some(Atomic::TLiteralInt(1)) => 2,
            Some(Atomic::TLiteralInt(2)) => 1,
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
            let actual_msg = if has_variadic {
                format!("callable accepting at least {} argument(s)", required_count)
            } else {
                format!("callable accepting {} argument(s)", max_params)
            };
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: "array_filter".to_string(),
                    expected: format!("callable accepting {} arg(s)", expected_arity),
                    actual: actual_msg,
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Validate array_reduce callback: arity must be >= 2 (carry, element).
pub(crate) fn check_array_reduce_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Union],
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
                fn_name: "array_reduce".to_string(),
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
        if required_count < 2 {
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: "array_reduce".to_string(),
                    expected: "callable accepting at least 2 arguments".to_string(),
                    actual: format!("callable accepting {} argument(s)", required_count),
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Validate sort callback (usort, uasort, uksort, array_walk, array_walk_recursive).
/// All need arity >= 2 (for sorts: comparison args; for array_walk: element, key).
pub(crate) fn check_sort_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    fn_name: &str,
    arg_types: &[Union],
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
        if required_count < 2 {
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: fn_name.to_string(),
                    expected: "callable accepting at least 2 arguments".to_string(),
                    actual: format!("callable accepting {} argument(s)", required_count),
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
    arg_ty: &Union,
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
fn callback_name_for_diagnostic(callback_ty: &Union) -> String {
    if let Some(Atomic::TLiteralString(fn_name)) = callback_ty.types.first() {
        fn_name.to_string()
    } else {
        "(closure)".to_string()
    }
}
