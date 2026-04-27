use php_ast::ast::{ExprKind, FunctionCallExpr};
use php_ast::Span;

use mir_codebase::storage::AssertionKind;
use mir_issues::{IssueKind, Severity};
use mir_types::Union;

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::generic::{check_template_bounds, infer_template_bindings};
use crate::symbol::SymbolKind;
use crate::taint::{classify_sink, is_expr_tainted, SinkKind};

use super::args::{
    check_args, expr_can_be_passed_by_reference, spread_element_type, CheckArgsParams,
};
use super::CallAnalyzer;

impl CallAnalyzer {
    pub fn analyze_function_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &FunctionCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
    ) -> Union {
        let fn_name = match &call.name.kind {
            ExprKind::Identifier(name) => (*name).to_string(),
            _ => {
                ea.analyze(call.name, ctx);
                for arg in call.args.iter() {
                    ea.analyze(&arg.value, ctx);
                }
                return Union::mixed();
            }
        };

        // Taint sink check (M19): before evaluating args so we can inspect raw exprs
        if let Some(sink_kind) = classify_sink(&fn_name) {
            for arg in call.args.iter() {
                if is_expr_tainted(&arg.value, ctx) {
                    let issue_kind = match sink_kind {
                        SinkKind::Html => IssueKind::TaintedHtml,
                        SinkKind::Sql => IssueKind::TaintedSql,
                        SinkKind::Shell => IssueKind::TaintedShell,
                    };
                    ea.emit(issue_kind, Severity::Error, span);
                    break;
                }
            }
        }

        // PHP resolves `foo()` as `\App\Ns\foo` first, then `\foo` if not found.
        // A leading `\` means explicit global namespace.
        let fn_name = fn_name
            .strip_prefix('\\')
            .map(|s: &str| s.to_string())
            .unwrap_or(fn_name);
        let resolved_fn_name: String = {
            let qualified = ea.codebase.resolve_class_name(&ea.file, &fn_name);
            if ea.codebase.functions.contains_key(qualified.as_str()) {
                qualified
            } else if ea.codebase.functions.contains_key(fn_name.as_str()) {
                fn_name.clone()
            } else {
                qualified
            }
        };

        // Pre-mark by-reference parameter variables as defined BEFORE evaluating args
        if let Some(func) = ea.codebase.functions.get(resolved_fn_name.as_str()) {
            for (i, param) in func.params.iter().enumerate() {
                if param.is_byref {
                    if param.is_variadic {
                        for arg in call.args.iter().skip(i) {
                            if let ExprKind::Variable(name) = &arg.value.kind {
                                let var_name = name.as_str().trim_start_matches('$');
                                if !ctx.var_is_defined(var_name) {
                                    ctx.set_var(var_name, Union::mixed());
                                }
                            }
                        }
                    } else if let Some(arg) = call.args.get(i) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let var_name = name.as_str().trim_start_matches('$');
                            if !ctx.var_is_defined(var_name) {
                                ctx.set_var(var_name, Union::mixed());
                            }
                        }
                    }
                }
            }
        }

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

        // When call_user_func / call_user_func_array is called with a bare string
        // literal as the callable argument, treat that string as a direct FQN
        // reference so the named function is not flagged as dead code.
        // Note: 'helper' always resolves to \helper (global) — no namespace
        // fallback applies to runtime callable strings.
        if matches!(
            resolved_fn_name.as_str(),
            "call_user_func" | "call_user_func_array"
        ) {
            if let Some(arg) = call.args.first() {
                if let ExprKind::String(name) = &arg.value.kind {
                    let fqn = name.strip_prefix('\\').unwrap_or(name);
                    if let Some(func) = ea.codebase.functions.get(fqn) {
                        ea.codebase.mark_function_referenced_at(
                            &func.fqn,
                            ea.file.clone(),
                            arg.span.start,
                            arg.span.end,
                        );
                    }
                }
            }
        }

        if let Some(func) = ea.codebase.functions.get(resolved_fn_name.as_str()) {
            let name_span = call.name.span;
            ea.codebase.mark_function_referenced_at(
                &func.fqn,
                ea.file.clone(),
                name_span.start,
                name_span.end,
            );
            let deprecated = func.deprecated.clone();
            let params = func.params.clone();
            let template_params = func.template_params.clone();
            let return_ty_raw = func
                .effective_return_type()
                .cloned()
                .unwrap_or_else(Union::mixed);

            if let Some(msg) = deprecated {
                ea.emit(
                    IssueKind::DeprecatedCall {
                        name: resolved_fn_name.clone(),
                        message: Some(msg).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    span,
                );
            }

            check_args(
                ea,
                CheckArgsParams {
                    fn_name: &fn_name,
                    params: &params,
                    arg_types: &arg_types,
                    arg_spans: &call.args.iter().map(|a| a.span).collect::<Vec<_>>(),
                    arg_names: &call
                        .args
                        .iter()
                        .map(|a| a.name.as_ref().map(|n| n.to_string_repr().into_owned()))
                        .collect::<Vec<_>>(),
                    arg_can_be_byref: &call
                        .args
                        .iter()
                        .map(|a| expr_can_be_passed_by_reference(&a.value))
                        .collect::<Vec<_>>(),
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                },
            );

            for (i, param) in params.iter().enumerate() {
                if param.is_byref {
                    if param.is_variadic {
                        for arg in call.args.iter().skip(i) {
                            if let ExprKind::Variable(name) = &arg.value.kind {
                                let var_name = name.as_str().trim_start_matches('$');
                                ctx.set_var(var_name, Union::mixed());
                            }
                        }
                    } else if let Some(arg) = call.args.get(i) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let var_name = name.as_str().trim_start_matches('$');
                            ctx.set_var(var_name, Union::mixed());
                        }
                    }
                }
            }

            for assertion in func
                .assertions
                .iter()
                .filter(|a| a.kind == AssertionKind::Assert)
            {
                if let Some(index) = params.iter().position(|p| p.name == assertion.param) {
                    if let Some(arg) = call.args.get(index) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            ctx.set_var(
                                name.as_str().trim_start_matches('$'),
                                assertion.ty.clone(),
                            );
                        }
                    }
                }
            }

            let return_ty = if !template_params.is_empty() {
                let bindings = infer_template_bindings(&template_params, &params, &arg_types);
                for (name, inferred, bound) in check_template_bounds(&bindings, &template_params) {
                    ea.emit(
                        IssueKind::InvalidTemplateParam {
                            name: name.to_string(),
                            expected_bound: format!("{bound}"),
                            actual: format!("{inferred}"),
                        },
                        Severity::Error,
                        span,
                    );
                }
                return_ty_raw.substitute_templates(&bindings)
            } else {
                return_ty_raw
            };

            ea.record_symbol(
                call.name.span,
                SymbolKind::FunctionCall(func.fqn.clone()),
                return_ty.clone(),
            );
            return return_ty;
        }

        ea.emit(
            IssueKind::UndefinedFunction { name: fn_name },
            Severity::Error,
            span,
        );
        Union::mixed()
    }
}
