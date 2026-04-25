/// Call analyzer — resolves function/method calls, checks arguments, returns
/// the inferred return type.
use std::sync::Arc;

use php_ast::ast::{
    ExprKind, FunctionCallExpr, MethodCallExpr, StaticDynMethodCallExpr, StaticMethodCallExpr,
};
use php_ast::Span;

use mir_codebase::storage::{FnParam, MethodStorage, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::generic::{build_class_bindings, check_template_bounds, infer_template_bindings};
use crate::symbol::SymbolKind;
use crate::taint::{classify_sink, is_expr_tainted, SinkKind};

// ---------------------------------------------------------------------------
// CallAnalyzer
// ---------------------------------------------------------------------------

pub struct CallAnalyzer;

impl CallAnalyzer {
    // -----------------------------------------------------------------------
    // Function calls: name(args)
    // -----------------------------------------------------------------------

    pub fn analyze_function_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &FunctionCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
    ) -> Union {
        // Resolve function name first (needed for sink check before arg eval)
        let fn_name = match &call.name.kind {
            ExprKind::Identifier(name) => (*name).to_string(),
            _ => {
                // dynamic call — evaluate name and args for read tracking
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
                    break; // one report per call site is enough
                }
            }
        }

        // Resolve the function name: try namespace-qualified first, then global fallback.
        // PHP resolves `foo()` as `\App\Ns\foo` first, then `\foo` if not found.
        // A leading `\` means explicit global namespace (e.g. `\assert` = global `assert`).
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
                // Keep the qualified name so the "unknown" error is informative
                qualified
            }
        };

        // Pre-mark by-reference parameter variables as defined BEFORE evaluating args,
        // so that passing an uninitialized variable to a by-ref param does not emit
        // UndefinedVariable (the function will initialize it).
        if let Some(func) = ea.codebase.functions.get(resolved_fn_name.as_str()) {
            for (i, param) in func.params.iter().enumerate() {
                if param.is_byref {
                    if param.is_variadic {
                        // Variadic by-ref: mark every remaining argument (e.g. sscanf output vars).
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

        // Evaluate all arguments
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

        // Look up user-defined function in codebase
        if let Some(func) = ea.codebase.functions.get(resolved_fn_name.as_str()) {
            // Use the name expression span, not the full call span, so the LSP
            // highlights only the function identifier.
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

            // Emit DeprecatedCall if the function is marked @deprecated
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
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                },
            );

            // Also ensure by-ref vars are defined after the call (for post-call usage)
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

            // Generic: substitute template params in return type
            let return_ty = if !template_params.is_empty() {
                let bindings = infer_template_bindings(&template_params, &params, &arg_types);
                // Check bounds
                for (name, inferred, bound) in check_template_bounds(&bindings, &template_params) {
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

        // Unknown function — report the unqualified name to keep the message readable
        ea.emit(
            IssueKind::UndefinedFunction { name: fn_name },
            Severity::Error,
            span,
        );
        Union::mixed()
    }

    // -----------------------------------------------------------------------
    // Method calls: $obj->method(args)
    // -----------------------------------------------------------------------

    pub fn analyze_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &MethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
        nullsafe: bool,
    ) -> Union {
        let obj_ty = ea.analyze(call.object, ctx);

        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) | ExprKind::Variable(name) => name.as_str(),
            _ => return Union::mixed(),
        };

        // Always analyze arguments — even when the receiver is null/mixed and we
        // return early — so that variable reads inside args are tracked (read_vars)
        // and side effects (taint, etc.) are recorded.
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

        // Null checks
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

        // Mixed receiver
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
                    // Resolve short names to FQCN — docblock types may not be fully qualified.
                    let fqcn_resolved = ea.codebase.resolve_class_name(&ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    if let Some(method) = ea.codebase.get_method(fqcn, method_name) {
                        // Record reference for dead-code detection (M18).
                        // Use call.method.span (the identifier only), not the full call
                        // span, so the LSP highlights just the method name.
                        ea.codebase.mark_method_referenced_at(
                            fqcn,
                            method_name,
                            ea.file.clone(),
                            call.method.span.start,
                            call.method.span.end,
                        );
                        // Emit DeprecatedMethodCall if the method is marked @deprecated
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
                        // Visibility check (simplified — only checks private from outside)
                        check_method_visibility(ea, &method, ctx, span);

                        // Arg type check
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
                                arg_types: &arg_types,
                                arg_spans: &arg_spans,
                                arg_names: &arg_names,
                                call_span: span,
                                has_spread: call.args.iter().any(|a| a.unpack),
                            },
                        );

                        let ret_raw = method
                            .effective_return_type()
                            .cloned()
                            .unwrap_or_else(Union::mixed);
                        // Bind `static` return type to the actual receiver class (LSB).
                        let ret_raw = substitute_static_in_return(ret_raw, fqcn);

                        // Build class-level bindings from receiver's concrete type params (e.g. Collection<User> → T=User)
                        let class_tps = ea.codebase.get_class_template_params(fqcn);
                        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
                        // Add bindings from @extends type args (e.g. class UserRepo extends BaseRepo<User> → T=User)
                        for (k, v) in ea.codebase.get_inherited_template_bindings(fqcn) {
                            bindings.entry(k).or_insert(v);
                        }

                        // Extend with method-level bindings; warn on name collision (method shadows class template)
                        if !method.template_params.is_empty() {
                            let method_bindings = infer_template_bindings(
                                &method.template_params,
                                &method.params,
                                &arg_types,
                            );
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
                            for (name, inferred, bound) in
                                check_template_bounds(&bindings, &method.template_params)
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

                        let ret = if !bindings.is_empty() {
                            ret_raw.substitute_templates(&bindings)
                        } else {
                            ret_raw
                        };
                        result = Union::merge(&result, &ret);
                    } else if ea.codebase.type_exists(fqcn)
                        && !ea.codebase.has_unknown_ancestor(fqcn)
                    {
                        // Class is known AND has no unscanned ancestors → genuine UndefinedMethod.
                        // If the class has an external/unscanned parent (e.g. a PHPUnit TestCase),
                        // the method might be inherited from that parent; skip to avoid false positives.
                        // Classes with __call handle any method dynamically — suppress.
                        // Interface types: method may exist on the concrete implementation — suppress
                        // (UndefinedInterfaceMethod is not emitted at default error level).
                        let is_interface = ea.codebase.interfaces.contains_key(fqcn.as_ref());
                        let is_abstract = ea.codebase.is_abstract_class(fqcn.as_ref());
                        if is_interface
                            || is_abstract
                            || ea.codebase.get_method(fqcn, "__call").is_some()
                        {
                            result = Union::merge(&result, &Union::mixed());
                        } else {
                            ea.emit(
                                IssueKind::UndefinedMethod {
                                    class: fqcn.to_string(),
                                    method: method_name.to_string(),
                                },
                                Severity::Error,
                                span,
                            );
                            result = Union::merge(&result, &Union::mixed());
                        }
                    } else {
                        result = Union::merge(&result, &Union::mixed());
                    }
                }
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => {
                    let receiver_type_params: &[mir_types::Union] = &[];
                    // Resolve short names to FQCN — docblock types may not be fully qualified.
                    let fqcn_resolved = ea.codebase.resolve_class_name(&ea.file, fqcn);
                    let fqcn = &std::sync::Arc::from(fqcn_resolved.as_str());
                    if let Some(method) = ea.codebase.get_method(fqcn, method_name) {
                        // Record reference for dead-code detection (M18).
                        // Use call.method.span (the identifier only), not the full call
                        // span, so the LSP highlights just the method name.
                        ea.codebase.mark_method_referenced_at(
                            fqcn,
                            method_name,
                            ea.file.clone(),
                            call.method.span.start,
                            call.method.span.end,
                        );
                        // Emit DeprecatedMethodCall if the method is marked @deprecated
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
                        // Visibility check (simplified — only checks private from outside)
                        check_method_visibility(ea, &method, ctx, span);

                        // Arg type check
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
                                arg_types: &arg_types,
                                arg_spans: &arg_spans,
                                arg_names: &arg_names,
                                call_span: span,
                                has_spread: call.args.iter().any(|a| a.unpack),
                            },
                        );

                        let ret_raw = method
                            .effective_return_type()
                            .cloned()
                            .unwrap_or_else(Union::mixed);
                        // Bind `static` return type to the actual receiver class (LSB).
                        let ret_raw = substitute_static_in_return(ret_raw, fqcn);

                        // Build class-level bindings from receiver's concrete type params (e.g. Collection<User> → T=User)
                        let class_tps = ea.codebase.get_class_template_params(fqcn);
                        let mut bindings = build_class_bindings(&class_tps, receiver_type_params);
                        // Add bindings from @extends type args (e.g. class UserRepo extends BaseRepo<User> → T=User)
                        for (k, v) in ea.codebase.get_inherited_template_bindings(fqcn) {
                            bindings.entry(k).or_insert(v);
                        }

                        // Extend with method-level bindings; warn on name collision (method shadows class template)
                        if !method.template_params.is_empty() {
                            let method_bindings = infer_template_bindings(
                                &method.template_params,
                                &method.params,
                                &arg_types,
                            );
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
                            for (name, inferred, bound) in
                                check_template_bounds(&bindings, &method.template_params)
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

                        let ret = if !bindings.is_empty() {
                            ret_raw.substitute_templates(&bindings)
                        } else {
                            ret_raw
                        };
                        result = Union::merge(&result, &ret);
                    } else if ea.codebase.type_exists(fqcn)
                        && !ea.codebase.has_unknown_ancestor(fqcn)
                    {
                        // Class is known AND has no unscanned ancestors → genuine UndefinedMethod.
                        // If the class has an external/unscanned parent (e.g. a PHPUnit TestCase),
                        // the method might be inherited from that parent; skip to avoid false positives.
                        // Classes with __call handle any method dynamically — suppress.
                        // Interface types: method may exist on the concrete implementation — suppress
                        // (UndefinedInterfaceMethod is not emitted at default error level).
                        let is_interface = ea.codebase.interfaces.contains_key(fqcn.as_ref());
                        let is_abstract = ea.codebase.is_abstract_class(fqcn.as_ref());
                        if is_interface
                            || is_abstract
                            || ea.codebase.get_method(fqcn, "__call").is_some()
                        {
                            result = Union::merge(&result, &Union::mixed());
                        } else {
                            ea.emit(
                                IssueKind::UndefinedMethod {
                                    class: fqcn.to_string(),
                                    method: method_name.to_string(),
                                },
                                Severity::Error,
                                span,
                            );
                            result = Union::merge(&result, &Union::mixed());
                        }
                    } else {
                        result = Union::merge(&result, &Union::mixed());
                    }
                }
                Atomic::TObject => {
                    result = Union::merge(&result, &Union::mixed());
                }
                // Template type parameters (e.g. `T` in `@template T`) are unbound at
                // analysis time — we cannot know which methods the concrete type will have,
                // so we must not emit UndefinedMethod here. Treat as mixed and move on.
                Atomic::TTemplateParam { .. } => {
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
        // Record method call symbol using the first named object in the receiver.
        // Use call.method.span (the identifier only), not the full call span, so
        // the LSP highlights just the method name.
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

    // -----------------------------------------------------------------------
    // Static method calls: ClassName::method(args)
    // -----------------------------------------------------------------------

    pub fn analyze_static_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticMethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
        span: Span,
    ) -> Union {
        let method_name = match &call.method.kind {
            ExprKind::Identifier(name) => name.as_str(),
            _ => return Union::mixed(),
        };

        let fqcn = match &call.class.kind {
            ExprKind::Identifier(name) => ea.codebase.resolve_class_name(&ea.file, name.as_ref()),
            _ => return Union::mixed(),
        };

        let fqcn = resolve_static_class(&fqcn, ctx);

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

        if let Some(method) = ea.codebase.get_method(&fqcn, method_name) {
            let method_span = call.method.span;
            ea.codebase.mark_method_referenced_at(
                &fqcn,
                method_name,
                ea.file.clone(),
                method_span.start,
                method_span.end,
            );
            // Emit DeprecatedMethodCall if the method is marked @deprecated
            if let Some(msg) = method.deprecated.clone() {
                ea.emit(
                    IssueKind::DeprecatedMethodCall {
                        class: fqcn.clone(),
                        method: method_name.to_string(),
                        message: Some(msg).filter(|m| !m.is_empty()),
                    },
                    Severity::Info,
                    span,
                );
            }
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
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                },
            );
            let ret_raw = method
                .effective_return_type()
                .cloned()
                .unwrap_or_else(Union::mixed);
            let fqcn_arc: std::sync::Arc<str> = Arc::from(fqcn.as_str());
            let ret = substitute_static_in_return(ret_raw, &fqcn_arc);
            ea.record_symbol(
                method_span,
                SymbolKind::StaticCall {
                    class: fqcn_arc,
                    method: Arc::from(method_name),
                },
                ret.clone(),
            );
            ret
        } else if ea.codebase.type_exists(&fqcn) && !ea.codebase.has_unknown_ancestor(&fqcn) {
            // Class is known AND has no unscanned ancestors → genuine UndefinedMethod.
            // Classes with __call handle any method dynamically — suppress.
            // Interface: concrete impl may have the method — suppress at default error level.
            let is_interface = ea.codebase.interfaces.contains_key(fqcn.as_str());
            let is_abstract = ea.codebase.is_abstract_class(&fqcn);
            if is_interface || is_abstract || ea.codebase.get_method(&fqcn, "__call").is_some() {
                Union::mixed()
            } else {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: fqcn,
                        method: method_name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
                Union::mixed()
            }
        } else {
            // Unknown/external class or class with unscanned ancestor — do not emit false positive
            Union::mixed()
        }
    }

    // -----------------------------------------------------------------------
    // Dynamic static method calls: ClassName::$variable(args)
    // -----------------------------------------------------------------------

    pub fn analyze_static_dyn_method_call<'a, 'arena, 'src>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &StaticDynMethodCallExpr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        // Evaluate args for side-effects / taint propagation.
        for arg in call.args.iter() {
            ea.analyze(&arg.value, ctx);
        }
        Union::mixed()
    }
}

// ---------------------------------------------------------------------------
// Public helper for constructor argument checking (used by expr.rs)
// ---------------------------------------------------------------------------

pub struct CheckArgsParams<'a> {
    pub fn_name: &'a str,
    pub params: &'a [FnParam],
    pub arg_types: &'a [Union],
    pub arg_spans: &'a [Span],
    pub arg_names: &'a [Option<String>],
    pub call_span: Span,
    pub has_spread: bool,
}

pub fn check_constructor_args(
    ea: &mut ExpressionAnalyzer<'_>,
    class_name: &str,
    p: CheckArgsParams<'_>,
) {
    let ctor_name = format!("{}::__construct", class_name);
    check_args(
        ea,
        CheckArgsParams {
            fn_name: &ctor_name,
            ..p
        },
    );
}

// ---------------------------------------------------------------------------
// Argument type checking
// ---------------------------------------------------------------------------

fn check_args(ea: &mut ExpressionAnalyzer<'_>, p: CheckArgsParams<'_>) {
    let CheckArgsParams {
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        call_span,
        has_spread,
    } = p;
    // Build a remapped (param_index → (arg_type, arg_span)) map that handles
    // named arguments (PHP 8.0+).
    let has_named = arg_names.iter().any(|n| n.is_some());

    // param_to_arg maps param index → (Union, Span)
    let mut param_to_arg: Vec<Option<(Union, Span)>> = vec![None; params.len()];

    if has_named {
        let mut positional = 0usize;
        for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
            if let Some(Some(name)) = arg_names.get(i) {
                // Named arg: find the param by name
                if let Some(pi) = params.iter().position(|p| p.name.as_ref() == name.as_str()) {
                    param_to_arg[pi] = Some((ty.clone(), *span));
                }
            } else {
                // Positional arg: fill the next unfilled slot
                while positional < params.len() && param_to_arg[positional].is_some() {
                    positional += 1;
                }
                if positional < params.len() {
                    param_to_arg[positional] = Some((ty.clone(), *span));
                    positional += 1;
                }
            }
        }
    } else {
        // Pure positional — fast path
        for (i, (ty, span)) in arg_types.iter().zip(arg_spans.iter()).enumerate() {
            if i < params.len() {
                param_to_arg[i] = Some((ty.clone(), *span));
            }
        }
    }

    let required_count = params
        .iter()
        .filter(|p| !p.is_optional && !p.is_variadic)
        .count();
    let provided_count = if params.iter().any(|p| p.is_variadic) {
        arg_types.len()
    } else {
        arg_types.len().min(params.len())
    };

    if provided_count < required_count && !has_spread {
        ea.emit(
            IssueKind::InvalidArgument {
                param: format!("#{}", provided_count + 1),
                fn_name: fn_name.to_string(),
                expected: format!("{} argument(s)", required_count),
                actual: format!("{} provided", provided_count),
            },
            Severity::Error,
            call_span,
        );
        return;
    }

    for (i, (param, slot)) in params.iter().zip(param_to_arg.iter()).enumerate() {
        let (arg_ty, arg_span) = match slot {
            Some(pair) => pair,
            None => continue, // optional param not supplied
        };
        let arg_span = *arg_span;
        let _ = i;

        if let Some(raw_param_ty) = &param.ty {
            // For variadic params annotated as list<T>, each argument should match T, not list<T>.
            let param_ty_owned;
            let param_ty: &Union = if param.is_variadic {
                if let Some(elem_ty) = raw_param_ty.types.iter().find_map(|a| match a {
                    Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                        Some(*value.clone())
                    }
                    _ => None,
                }) {
                    param_ty_owned = elem_ty;
                    &param_ty_owned
                } else {
                    raw_param_ty
                }
            } else {
                raw_param_ty
            };
            // Null check: param is not nullable but arg could be null.
            // Check definite null (single TNull) before possibly-null union to emit the
            // correct severity: a literal `null` is InvalidArgument, not PossiblyNullArgument.
            if !param_ty.is_nullable()
                && !param_ty.is_mixed()
                && arg_ty.is_single()
                && arg_ty.contains(|t| matches!(t, Atomic::TNull))
            {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                        expected: format!("{}", param_ty),
                        actual: format!("{}", arg_ty),
                    },
                    Severity::Error,
                    arg_span,
                );
            } else if !param_ty.is_nullable() && !param_ty.is_mixed() && arg_ty.is_nullable() {
                ea.emit(
                    IssueKind::PossiblyNullArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                    },
                    Severity::Info,
                    arg_span,
                );
            }

            // Type compatibility check: first try the fast structural check, then fall
            // back to a codebase-aware check that handles class hierarchy and FQCN resolution.
            if !arg_ty.is_subtype_of_simple(param_ty)
                && !param_ty.is_mixed()
                && !arg_ty.is_mixed()
                && !named_object_subtype(arg_ty, param_ty, ea)
                && !param_contains_template_or_unknown(param_ty, ea)
                && !param_contains_template_or_unknown(arg_ty, ea)
                && !array_list_compatible(arg_ty, param_ty, ea)
                // Skip when param is more specific than arg (coercion, not hard error):
                // e.g. string → non-empty-string, int → positive-int, string → string|null
                // Only applies when arg is a single type; union args like int|string passed to
                // an int param must still error even though int <: int|string.
                && !(arg_ty.is_single() && param_ty.is_subtype_of_simple(arg_ty))
                // Skip when non-null part of param is a subtype of arg (e.g. non-empty-string|null ← string)
                && !(arg_ty.is_single() && param_ty.remove_null().is_subtype_of_simple(arg_ty))
                // Skip when any atomic in param is a subtype of arg (e.g. non-empty-string|list ← string)
                && !(arg_ty.is_single() && param_ty.types.iter().any(|p| Union::single(p.clone()).is_subtype_of_simple(arg_ty)))
                // Skip when arg is compatible after removing null/false (PossiblyNull/FalseArgument
                // handles these separately and they may appear in the baseline)
                && !arg_ty.remove_null().is_subtype_of_simple(param_ty)
                && !arg_ty.remove_false().is_subtype_of_simple(param_ty)
                && !named_object_subtype(&arg_ty.remove_null(), param_ty, ea)
                && !named_object_subtype(&arg_ty.remove_false(), param_ty, ea)
            {
                ea.emit(
                    IssueKind::InvalidArgument {
                        param: param.name.to_string(),
                        fn_name: fn_name.to_string(),
                        expected: format!("{}", param_ty),
                        actual: format!("{}", arg_ty),
                    },
                    Severity::Error,
                    arg_span,
                );
            }
        }
    }
}

/// Returns true if every atomic in `arg` can be assigned to some atomic in `param`
/// using codebase-aware class hierarchy checks.
///
/// Handles two common false-positive cases:
/// 1. `BackOffBuilder` stored as short name in param vs FQCN in arg → resolve both.
/// 2. `DateTimeImmutable` extends `DateTimeInterface` → use `extends_or_implements`.
fn named_object_subtype(arg: &Union, param: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    use mir_types::Atomic;
    // Every atomic in arg must satisfy the param
    arg.types.iter().all(|a_atomic| {
        // Extract FQCN from the arg atomic — handles TNamedObject, TSelf, TStaticObject, TParent
        let arg_fqcn: &Arc<str> = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => {
                // If the self/static refers to a trait, we can't know the concrete class — skip
                if ea.codebase.traits.contains_key(fqcn.as_ref()) {
                    return true;
                }
                fqcn
            }
            Atomic::TParent { fqcn } => fqcn,
            // TNever is bottom type — compatible with any param
            Atomic::TNever => return true,
            // Closure() types satisfy Closure or callable param
            Atomic::TClosure { .. } => {
                return param.types.iter().any(|p| match p {
                    Atomic::TClosure { .. } | Atomic::TCallable { .. } => true,
                    Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref() == "Closure",
                    _ => false,
                });
            }
            // callable satisfies Closure param (not flagged at default error level)
            Atomic::TCallable { .. } => {
                return param.types.iter().any(|p| match p {
                    Atomic::TCallable { .. } | Atomic::TClosure { .. } => true,
                    Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref() == "Closure",
                    _ => false,
                });
            }
            // class-string<X> is compatible with class-string<Y> if X extends/implements Y
            Atomic::TClassString(Some(arg_cls)) => {
                return param.types.iter().any(|p| match p {
                    Atomic::TClassString(None) | Atomic::TString => true,
                    Atomic::TClassString(Some(param_cls)) => {
                        arg_cls == param_cls
                            || ea
                                .codebase
                                .extends_or_implements(arg_cls.as_ref(), param_cls.as_ref())
                    }
                    _ => false,
                });
            }
            // Null satisfies param if param also contains null
            Atomic::TNull => {
                return param.types.iter().any(|p| matches!(p, Atomic::TNull));
            }
            // False satisfies param if param contains false or bool
            Atomic::TFalse => {
                return param
                    .types
                    .iter()
                    .any(|p| matches!(p, Atomic::TFalse | Atomic::TBool));
            }
            _ => return false, // non-named-object: not handled here
        };

        // An object with __invoke satisfies callable|null
        if param
            .types
            .iter()
            .any(|p| matches!(p, Atomic::TCallable { .. }))
        {
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());
            if ea.codebase.get_method(&resolved_arg, "__invoke").is_some()
                || ea
                    .codebase
                    .get_method(arg_fqcn.as_ref(), "__invoke")
                    .is_some()
            {
                return true;
            }
        }

        param.types.iter().any(|p_atomic| {
            let param_fqcn: &Arc<str> = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            // Resolve param_fqcn in case it's a short name stored from a type hint
            let resolved_param = ea
                .codebase
                .resolve_class_name(&ea.file, param_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());

            // Same class — check generic type params with variance
            let is_same_class = resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref();

            if is_same_class {
                let arg_type_params = match a_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                let param_type_params = match p_atomic {
                    Atomic::TNamedObject { type_params, .. } => type_params.as_slice(),
                    _ => &[],
                };
                if !arg_type_params.is_empty() || !param_type_params.is_empty() {
                    let class_tps = ea.codebase.get_class_template_params(&resolved_param);
                    return generic_type_params_compatible(
                        arg_type_params,
                        param_type_params,
                        &class_tps,
                        ea,
                    );
                }
                return true;
            }

            if ea.codebase.extends_or_implements(arg_fqcn.as_ref(), &resolved_param)
                || ea.codebase.extends_or_implements(arg_fqcn.as_ref(), param_fqcn.as_ref())
                || ea.codebase.extends_or_implements(&resolved_arg, &resolved_param)
                // ArgumentTypeCoercion (suppressed at level 3): param extends arg — arg is
                // broader than param. Not a hard error; only flagged at stricter error levels.
                || ea.codebase.extends_or_implements(param_fqcn.as_ref(), &resolved_arg)
                || ea.codebase.extends_or_implements(param_fqcn.as_ref(), arg_fqcn.as_ref())
                || ea.codebase.extends_or_implements(&resolved_param, &resolved_arg)
            {
                return true;
            }

            // If arg_fqcn is a short name (no namespace) that didn't resolve through the caller
            // file's imports (e.g., return type from a vendor method like `NonNull` from
            // `Type::nonNull()`), search codebase for any class with that short_name and check
            // if it satisfies the param type.
            if !arg_fqcn.contains('\\') && !ea.codebase.type_exists(&resolved_arg) {
                for entry in ea.codebase.classes.iter() {
                    if entry.value().short_name.as_ref() == arg_fqcn.as_ref() {
                        let actual_fqcn = entry.key().clone();
                        if ea
                            .codebase
                            .extends_or_implements(actual_fqcn.as_ref(), &resolved_param)
                            || ea
                                .codebase
                                .extends_or_implements(actual_fqcn.as_ref(), param_fqcn.as_ref())
                        {
                            return true;
                        }
                    }
                }
            }

            // If arg_fqcn is an interface, check if any known concrete class both implements
            // the interface AND extends/implements the param. This handles cases like
            // `ValueNode` (interface) whose implementations all extend `Node` (abstract class).
            let iface_key = if ea.codebase.interfaces.contains_key(arg_fqcn.as_ref()) {
                Some(arg_fqcn.as_ref())
            } else if ea.codebase.interfaces.contains_key(resolved_arg.as_str()) {
                Some(resolved_arg.as_str())
            } else {
                None
            };
            if let Some(iface_fqcn) = iface_key {
                let compatible = ea.codebase.classes.iter().any(|entry| {
                    let cls = entry.value();
                    cls.all_parents.iter().any(|p| p.as_ref() == iface_fqcn)
                        && (ea
                            .codebase
                            .extends_or_implements(entry.key().as_ref(), param_fqcn.as_ref())
                            || ea
                                .codebase
                                .extends_or_implements(entry.key().as_ref(), &resolved_param))
                });
                if compatible {
                    return true;
                }
            }

            // If arg is a fully-qualified vendor class not in our codebase, we can't verify
            // the hierarchy — suppress to avoid false positives on external libraries.
            if arg_fqcn.contains('\\')
                && !ea.codebase.type_exists(arg_fqcn.as_ref())
                && !ea.codebase.type_exists(&resolved_arg)
            {
                return true;
            }

            // If param is a fully-qualified vendor class not in our codebase, we can't verify
            // the required type — suppress to avoid false positives on external library params.
            if param_fqcn.contains('\\')
                && !ea.codebase.type_exists(param_fqcn.as_ref())
                && !ea.codebase.type_exists(&resolved_param)
            {
                return true;
            }

            false
        })
    })
}

/// Strict codebase-aware subtype check for generic type parameter positions.
///
/// Unlike `named_object_subtype`, this does NOT include the coercion direction (param extends arg).
/// That relaxation exists for outer argument checking only — applying it inside type parameter
/// positions would incorrectly accept e.g. `Box<Animal>` → `Box<Cat>` in a covariant context
/// because `Cat extends Animal` would trigger the coercion acceptance.
fn strict_named_object_subtype(arg: &Union, param: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    use mir_types::Atomic;
    arg.types.iter().all(|a_atomic| {
        let arg_fqcn: &Arc<str> = match a_atomic {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TNever => return true,
            _ => return false,
        };
        param.types.iter().any(|p_atomic| {
            let param_fqcn: &Arc<str> = match p_atomic {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                _ => return false,
            };
            let resolved_param = ea
                .codebase
                .resolve_class_name(&ea.file, param_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, arg_fqcn.as_ref());
            // Forward direction only — arg must extend/implement param. No coercion.
            resolved_param == resolved_arg
                || arg_fqcn.as_ref() == resolved_param.as_str()
                || resolved_arg == param_fqcn.as_ref()
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(arg_fqcn.as_ref(), param_fqcn.as_ref())
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param)
        })
    })
}

/// Check whether generic type parameters are compatible according to each parameter's declared
/// variance (`@template-covariant`, `@template-contravariant`, or invariant by default).
///
/// - Covariant: `C<Sub>` satisfies `C<Super>` when `Sub <: Super`.
/// - Contravariant: `C<Super>` satisfies `C<Sub>` when `Super <: Sub` (reversed).
/// - Invariant: exact structural match required.
fn generic_type_params_compatible(
    arg_params: &[Union],
    param_params: &[Union],
    template_params: &[mir_codebase::storage::TemplateParam],
    ea: &ExpressionAnalyzer<'_>,
) -> bool {
    // Mismatched arity (raw / uninstantiated generic) — be permissive.
    if arg_params.len() != param_params.len() {
        return true;
    }
    // No type params on either side — trivially compatible.
    if arg_params.is_empty() {
        return true;
    }

    for (i, (arg_p, param_p)) in arg_params.iter().zip(param_params.iter()).enumerate() {
        let variance = template_params
            .get(i)
            .map(|tp| tp.variance)
            .unwrap_or(mir_types::Variance::Invariant);

        let compatible = match variance {
            mir_types::Variance::Covariant => {
                // C<Cat> satisfies C<Animal> when Cat <: Animal.
                arg_p.is_subtype_of_simple(param_p)
                    || param_p.is_mixed()
                    || arg_p.is_mixed()
                    || strict_named_object_subtype(arg_p, param_p, ea)
            }
            mir_types::Variance::Contravariant => {
                // C<Animal> satisfies C<Cat> when Animal <: Cat (reversed direction).
                param_p.is_subtype_of_simple(arg_p)
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || strict_named_object_subtype(param_p, arg_p, ea)
            }
            mir_types::Variance::Invariant => {
                // Exact structural match or mutual subtyping.
                arg_p == param_p
                    || arg_p.is_mixed()
                    || param_p.is_mixed()
                    || (arg_p.is_subtype_of_simple(param_p) && param_p.is_subtype_of_simple(arg_p))
            }
        };

        if !compatible {
            return false;
        }
    }

    true
}

/// Returns true if the param type contains a template-like type (a TNamedObject whose FQCN
/// is a single uppercase letter or doesn't exist in the codebase) indicating the function
/// uses generics. We can't validate the argument type without full template instantiation.
fn param_contains_template_or_unknown(param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    param_ty.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, .. } => {
            !fqcn.contains('\\') && !ea.codebase.type_exists(fqcn.as_ref())
        }
        // class-string<T> where T is a template param (single-letter or unknown)
        Atomic::TClassString(Some(inner)) => {
            !inner.contains('\\') && !ea.codebase.type_exists(inner.as_ref())
        }
        Atomic::TArray { key: _, value }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { key: _, value }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                !fqcn.contains('\\') && !ea.codebase.type_exists(fqcn.as_ref())
            }
            _ => false,
        }),
        _ => false,
    })
}

/// Replace `TStaticObject` / `TSelf` in a method's return type with the actual receiver FQCN.
/// `static` (LSB) and `self` in trait context both resolve to the concrete receiver class.
fn substitute_static_in_return(ret: Union, receiver_fqcn: &Arc<str>) -> Union {
    use mir_types::Atomic;
    let from_docblock = ret.from_docblock;
    let types: Vec<Atomic> = ret
        .types
        .into_iter()
        .map(|a| match a {
            Atomic::TStaticObject { .. } | Atomic::TSelf { .. } => Atomic::TNamedObject {
                fqcn: receiver_fqcn.clone(),
                type_params: vec![],
            },
            other => other,
        })
        .collect();
    let mut result = Union::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

/// For a spread (`...`) argument, return the union of value types across all array atomics.
/// E.g. `array<int, int>` → `int`, `list<string>` → `string`, `mixed` → `mixed`.
/// This lets us compare the element type against the variadic param type.
pub fn spread_element_type(arr_ty: &Union) -> Union {
    use mir_types::Atomic;
    let mut result = Union::empty();
    for atomic in arr_ty.types.iter() {
        match atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                for t in value.types.iter() {
                    result.add_type(t.clone());
                }
            }
            Atomic::TKeyedArray { properties, .. } => {
                for (_key, prop) in properties.iter() {
                    for t in prop.ty.types.iter() {
                        result.add_type(t.clone());
                    }
                }
            }
            // If the spread value isn't an array (or is mixed), treat as mixed
            _ => return Union::mixed(),
        }
    }
    if result.types.is_empty() {
        Union::mixed()
    } else {
        result
    }
}

/// Returns true if both arg and param are array/list types whose value types are compatible
/// with FQCN resolution (e.g., `array<int, FQCN>` satisfies `list<ShortName>`).
/// Recursive codebase-aware union compatibility check.
/// Returns true if every atomic in `arg_ty` is compatible with `param_ty`,
/// handling nested lists/arrays and FQCN resolution.
fn union_compatible(arg_ty: &Union, param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|av| {
        // Named object: use FQCN resolution
        let av_fqcn: &Arc<str> = match av {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } | Atomic::TParent { fqcn } => {
                fqcn
            }
            // Nested list/array: recurse
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                return param_ty.types.iter().any(|pv| {
                    let pv_val: &Union = match pv {
                        Atomic::TArray { value, .. }
                        | Atomic::TNonEmptyArray { value, .. }
                        | Atomic::TList { value }
                        | Atomic::TNonEmptyList { value } => value,
                        _ => return false,
                    };
                    union_compatible(value, pv_val, ea)
                });
            }
            Atomic::TKeyedArray { .. } => return true,
            _ => return Union::single(av.clone()).is_subtype_of_simple(param_ty),
        };

        param_ty.types.iter().any(|pv| {
            let pv_fqcn: &Arc<str> = match pv {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };
            // Template param wildcard
            if !pv_fqcn.contains('\\') && !ea.codebase.type_exists(pv_fqcn.as_ref()) {
                return true;
            }
            let resolved_param = ea.codebase.resolve_class_name(&ea.file, pv_fqcn.as_ref());
            let resolved_arg = ea.codebase.resolve_class_name(&ea.file, av_fqcn.as_ref());
            resolved_param == resolved_arg
                || ea
                    .codebase
                    .extends_or_implements(av_fqcn.as_ref(), &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(&resolved_arg, &resolved_param)
                || ea
                    .codebase
                    .extends_or_implements(pv_fqcn.as_ref(), &resolved_arg)
                || ea
                    .codebase
                    .extends_or_implements(&resolved_param, &resolved_arg)
        })
    })
}

fn array_list_compatible(arg_ty: &Union, param_ty: &Union, ea: &ExpressionAnalyzer<'_>) -> bool {
    arg_ty.types.iter().all(|a_atomic| {
        let arg_value: &Union = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray { .. } => return true, // keyed arrays are compatible with any list/array
            _ => return false,
        };

        param_ty.types.iter().any(|p_atomic| {
            let param_value: &Union = match p_atomic {
                Atomic::TArray { value, .. }
                | Atomic::TNonEmptyArray { value, .. }
                | Atomic::TList { value }
                | Atomic::TNonEmptyList { value } => value,
                _ => return false,
            };

            union_compatible(arg_value, param_value, ea)
        })
    })
}

fn check_method_visibility(
    ea: &mut ExpressionAnalyzer<'_>,
    method: &MethodStorage,
    ctx: &Context,
    span: Span,
) {
    match method.visibility {
        Visibility::Private => {
            // Private methods can only be called from within the same class
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            if caller_fqcn != method.fqcn.as_ref() {
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: method.fqcn.to_string(),
                        method: method.name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            }
        }
        Visibility::Protected => {
            // Protected: callable only from within the declaring class or its subclasses
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            if caller_fqcn.is_empty() {
                // Called from outside any class — not allowed
                ea.emit(
                    IssueKind::UndefinedMethod {
                        class: method.fqcn.to_string(),
                        method: method.name.to_string(),
                    },
                    Severity::Error,
                    span,
                );
            } else {
                // Caller must be the method's class or a subclass of it
                let allowed = caller_fqcn == method.fqcn.as_ref()
                    || ea
                        .codebase
                        .extends_or_implements(caller_fqcn, method.fqcn.as_ref());
                if !allowed {
                    ea.emit(
                        IssueKind::UndefinedMethod {
                            class: method.fqcn.to_string(),
                            method: method.name.to_string(),
                        },
                        Severity::Error,
                        span,
                    );
                }
            }
        }
        Visibility::Public => {}
    }
}

fn resolve_static_class(name: &str, ctx: &Context) -> String {
    match name.to_lowercase().as_str() {
        "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
        "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
        "static" => ctx
            .static_fqcn
            .as_deref()
            .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
            .to_string(),
        _ => name.to_string(),
    }
}
