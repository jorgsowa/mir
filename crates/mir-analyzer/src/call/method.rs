use std::sync::Arc;

use php_ast::ast::{ExprKind, MethodCallExpr};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::generic::{build_class_bindings, check_template_bounds, infer_template_bindings};
use crate::symbol::SymbolKind;

use super::args::{
    check_args, check_method_visibility, spread_element_type, substitute_static_in_return,
    CheckArgsParams,
};
use super::CallAnalyzer;

impl CallAnalyzer {
    pub fn analyze_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &MethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
        nullsafe: bool,
    ) -> Union {
        let obj_ty = ea.analyze(call.object, ctx);

        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_str(),
            _ => return Union::mixed(),
        };

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked and side
        // effects (taint, etc.) are recorded.
        let arg_types: Vec<Union> = call
            .args
            .iter()
            .map(|arg| {
                let ty = ea.analyze(&arg.value, ctx);
                if arg.unpack {
                    spread_element_type(&ty)
                } else {
                    ty
                }
            })
            .collect();

        let arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        if obj_ty.contains(|t| matches!(t, Atomic::TNull)) {
            if nullsafe {
                // ?-> is fine, just returns null on null receiver
            } else if obj_ty.is_single() {
                ea.emit(
                    IssueKind::NullMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
                return Union::mixed();
            } else {
                ea.emit(
                    IssueKind::PossiblyNullMethodCall {
                        method: method_name.to_string(),
                    },
                    Severity::Info,
                    span,
                );
            }
        }

        if obj_ty.is_mixed() {
            ea.emit(
                IssueKind::MixedMethodCall {
                    method: method_name.to_string(),
                },
                Severity::Info,
                span,
            );
            return Union::mixed();
        }

        let receiver = obj_ty.remove_null();
        let mut result = Union::empty();

        for atomic in &receiver.types {
            match atomic {
                Atomic::TNamedObject {
                    fqcn,
                    type_params: receiver_type_params,
                } => {
                    let fqcn_resolved = ea.codebase.resolve_class_name(&ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    result = Union::merge(
                        &result,
                        &resolve_method_return(
                            ea,
                            ctx,
                            call,
                            span,
                            method_name,
                            fqcn,
                            receiver_type_params.as_slice(),
                            &arg_types,
                            &arg_spans,
                        ),
                    );
                }
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => {
                    let fqcn_resolved = ea.codebase.resolve_class_name(&ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    result = Union::merge(
                        &result,
                        &resolve_method_return(
                            ea,
                            ctx,
                            call,
                            span,
                            method_name,
                            fqcn,
                            &[],
                            &arg_types,
                            &arg_spans,
                        ),
                    );
                }
                Atomic::TObject | Atomic::TTemplateParam { .. } => {
                    result = Union::merge(&result, &Union::mixed());
                }
                _ => {
                    result = Union::merge(&result, &Union::mixed());
                }
            }
        }

        if nullsafe && obj_ty.is_nullable() {
            result.add_type(Atomic::TNull);
        }

        let final_ty = if result.is_empty() {
            Union::mixed()
        } else {
            result
        };

        for atomic in &obj_ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                ea.record_symbol(
                    call.method.span,
                    SymbolKind::MethodCall {
                        class: fqcn.clone(),
                        method: Arc::from(method_name),
                    },
                    final_ty.clone(),
                );
                break;
            }
        }
        final_ty
    }
}

/// Resolves method return type for a known receiver FQCN, shared between the
/// `TNamedObject` and `TSelf`/`TStaticObject`/`TParent` branches.
#[allow(clippy::too_many_arguments)]
fn resolve_method_return<'a, 'arena, 'src>(
    ea: &mut ExpressionAnalyzer<'a>,
    ctx: &Context,
    call: &MethodCallExpr<'arena, 'src>,
    span: Span,
    method_name: &str,
    fqcn: &Arc<str>,
    receiver_type_params: &[Union],
    arg_types: &[Union],
    arg_spans: &[Span],
) -> Union {
    if let Some(method) = ea.codebase.get_method(fqcn, method_name) {
        ea.codebase.mark_method_referenced_at(
            fqcn,
            method_name,
            ea.file.clone(),
            call.method.span.start,
            call.method.span.end,
        );
        if let Some(msg) = method.deprecated.clone() {
            ea.emit(
                IssueKind::DeprecatedMethodCall {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                    message: Some(msg).filter(|m| !m.is_empty()),
                },
                Severity::Info,
                span,
            );
        }
        check_method_visibility(ea, &method, ctx, span);

        let arg_names: Vec<Option<String>> = call
            .args
            .iter()
            .map(|a| a.name.as_ref().map(|n| n.to_string_repr().into_owned()))
            .collect();
        check_args(
            ea,
            CheckArgsParams {
                fn_name: method_name,
                params: &method.params,
                arg_types,
                arg_spans,
                arg_names: &arg_names,
                call_span: span,
                has_spread: call.args.iter().any(|a| a.unpack),
            },
        );

        let ret_raw = method
            .effective_return_type()
            .cloned()
            .unwrap_or_else(Union::mixed);
        let ret_raw = substitute_static_in_return(ret_raw, fqcn);

        let class_tps = ea.codebase.get_class_template_params(fqcn);
        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
        for (k, v) in ea.codebase.get_inherited_template_bindings(fqcn) {
            bindings.entry(k).or_insert(v);
        }

        if !method.template_params.is_empty() {
            let method_bindings =
                infer_template_bindings(&method.template_params, &method.params, arg_types);
            for key in method_bindings.keys() {
                if bindings.contains_key(key) {
                    ea.emit(
                        IssueKind::ShadowedTemplateParam {
                            name: key.to_string(),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }
            bindings.extend(method_bindings);
            for (name, inferred, bound) in check_template_bounds(&bindings, &method.template_params)
            {
                ea.emit(
                    IssueKind::InvalidTemplateParam {
                        name: name.to_string(),
                        expected_bound: format!("{}", bound),
                        actual: format!("{}", inferred),
                    },
                    Severity::Error,
                    span,
                );
            }
        }

        if !bindings.is_empty() {
            ret_raw.substitute_templates(&bindings)
        } else {
            ret_raw
        }
    } else if ea.codebase.type_exists(fqcn) && !ea.codebase.has_unknown_ancestor(fqcn) {
        let is_interface = ea.codebase.interfaces.contains_key(fqcn.as_ref());
        let is_abstract = ea.codebase.is_abstract_class(fqcn.as_ref());
        if is_interface || is_abstract || ea.codebase.get_method(fqcn, "__call").is_some() {
            Union::mixed()
        } else {
            ea.emit(
                IssueKind::UndefinedMethod {
                    class: fqcn.to_string(),
                    method: method_name.to_string(),
                },
                Severity::Error,
                span,
            );
            Union::mixed()
        }
    } else {
        Union::mixed()
    }
}
