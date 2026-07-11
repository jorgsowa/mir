use php_ast::Span;

use mir_codebase::definitions::DeclaredParam;
use mir_issues::{IssueKind, Severity};
use mir_types::Type;

use crate::expr::ExpressionAnalyzer;

use super::ArgBinding;

#[allow(clippy::too_many_arguments)]
pub(super) fn check_counts(
    ea: &mut ExpressionAnalyzer<'_>,
    fn_name: &str,
    params: &[DeclaredParam],
    arg_types: &[Type],
    arg_spans: &[Span],
    arg_names: &[Option<String>],
    call_span: Span,
    has_spread: bool,
    arity_unknown: bool,
    no_named_arguments: bool,
) -> Vec<ArgBinding> {
    let variadic_index = params.iter().position(|p| p.is_variadic);
    let max_positional = variadic_index.unwrap_or(params.len());
    let mut param_to_arg: Vec<Option<(Type, Span, usize)>> = vec![None; params.len()];
    let mut arg_bindings: Vec<ArgBinding> = Vec::new();
    let mut positional = 0usize;
    let mut seen_named = false;
    let mut has_shape_error = false;

    // @no-named-arguments: emit per named arg before the regular processing.
    if no_named_arguments {
        for (i, span) in arg_spans.iter().enumerate() {
            if has_spread && i > 0 {
                break;
            }
            if let Some(Some(_)) = arg_names.get(i) {
                ea.emit(
                    IssueKind::InvalidNamedArguments {
                        fn_name: fn_name.to_string(),
                    },
                    Severity::Error,
                    *span,
                );
            }
        }
    }

    for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
        if has_spread && i > 0 {
            break;
        }

        if let Some(Some(name)) = arg_names.get(i) {
            seen_named = true;
            if let Some(pi) = params.iter().position(|p| p.name.as_ref() == name.as_str()) {
                if param_to_arg[pi].is_some() {
                    has_shape_error = true;
                    ea.emit(
                        IssueKind::InvalidNamedArgument {
                            fn_name: fn_name.to_string(),
                            name: name.to_string(),
                        },
                        Severity::Error,
                        *span,
                    );
                    continue;
                }
                param_to_arg[pi] = Some((ty.clone(), *span, i));
                arg_bindings.push(ArgBinding {
                    param_idx: pi,
                    arg_ty: ty.clone(),
                    arg_span: *span,
                    arg_idx: i,
                });
            } else if let Some(vi) = variadic_index {
                arg_bindings.push(ArgBinding {
                    param_idx: vi,
                    arg_ty: ty.clone(),
                    arg_span: *span,
                    arg_idx: i,
                });
            } else {
                has_shape_error = true;
                ea.emit(
                    IssueKind::InvalidNamedArgument {
                        fn_name: fn_name.to_string(),
                        name: name.to_string(),
                    },
                    Severity::Error,
                    *span,
                );
            }
            continue;
        }

        if seen_named && !has_spread {
            has_shape_error = true;
            ea.emit(
                IssueKind::InvalidNamedArgument {
                    fn_name: fn_name.to_string(),
                    name: format!("#{}", i + 1),
                },
                Severity::Error,
                *span,
            );
            continue;
        }

        while positional < max_positional && param_to_arg[positional].is_some() {
            positional += 1;
        }

        let Some(pi) = (if positional < max_positional {
            Some(positional)
        } else {
            variadic_index
        }) else {
            continue;
        };

        if pi < max_positional {
            param_to_arg[pi] = Some((ty.clone(), *span, i));
            positional += 1;
        }
        arg_bindings.push(ArgBinding {
            param_idx: pi,
            arg_ty: ty.clone(),
            arg_span: *span,
            arg_idx: i,
        });
    }

    let required_count = params
        .iter()
        .filter(|p| !p.is_optional && !p.is_variadic)
        .count();
    let provided_count = param_to_arg
        .iter()
        .take(required_count)
        .filter(|slot| slot.is_some())
        .count();

    if provided_count < required_count && !arity_unknown && !has_shape_error {
        ea.emit(
            IssueKind::TooFewArguments {
                fn_name: fn_name.to_string(),
                expected: required_count,
                actual: arg_types.len(),
            },
            Severity::Error,
            call_span,
        );
    }

    // PHP silently ignores surplus positional arguments passed to a closure
    // (they remain reachable via func_get_args()), so a direct closure call with
    // extra args is not an error. Named functions/methods keep the lint.
    if variadic_index.is_none()
        && arg_types.len() > params.len()
        && !arity_unknown
        && !has_shape_error
        && fn_name != "{closure}"
    {
        ea.emit(
            IssueKind::TooManyArguments {
                fn_name: fn_name.to_string(),
                expected: params.len(),
                actual: arg_types.len(),
            },
            Severity::Error,
            arg_spans.get(params.len()).copied().unwrap_or(call_span),
        );
    }

    arg_bindings
}
