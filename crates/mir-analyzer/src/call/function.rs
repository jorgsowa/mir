use php_ast::owned::{ExprKind, FunctionCallExpr};
use php_ast::Span;

use std::sync::Arc;

use mir_codebase::storage::{Assertion, AssertionKind, FnParam, TemplateParam};
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::generic::{check_template_bounds_with_inheritance, infer_template_bindings};
use crate::symbol::ReferenceKind;
use crate::taint::{classify_sink, is_expr_tainted, SinkKind};

use super::args::{
    check_args, expr_can_be_passed_by_reference_owned, spread_element_type, CheckArgsParams,
};
use super::callable::extract_callable_params;
use super::CallAnalyzer;

struct ResolvedFn {
    fqn: std::sync::Arc<str>,
    deprecated: Option<std::sync::Arc<str>>,
    params: Vec<FnParam>,
    template_params: Vec<TemplateParam>,
    assertions: Vec<Assertion>,
    return_ty_raw: Type,
    throws: Arc<[Arc<str>]>,
}

fn resolve_fn(ea: &ExpressionAnalyzer<'_>, fqn: &str) -> Option<ResolvedFn> {
    let db = ea.db;
    let inferred = crate::db::inferred_function_return_type_demand(db, fqn);
    let here = crate::db::Fqcn::from_str(db, fqn);
    if let Some(f) = crate::db::find_function(db, here) {
        let return_ty_raw = f
            .return_type
            .clone()
            .or(inferred)
            .map(|t| (*t).clone())
            .unwrap_or_else(Type::mixed);
        return Some(ResolvedFn {
            fqn: f.fqn.clone(),
            deprecated: f.deprecated.clone(),
            params: f.params.to_vec(),
            template_params: f.template_params.clone(),
            assertions: f.assertions.clone(),
            return_ty_raw,
            throws: Arc::<[Arc<str>]>::from(f.throws.as_slice()),
        });
    }
    None
}

impl CallAnalyzer {
    pub fn analyze_function_call<'a>(
        ea: &mut ExpressionAnalyzer<'a>,
        call: &FunctionCallExpr,
        ctx: &mut FlowState,
        span: Span,
    ) -> Type {
        let fn_name = match &call.name.kind {
            ExprKind::Identifier(name) => name.as_ref().to_string(),
            _ => {
                let callee_ty = ea.analyze(&call.name, ctx);
                for arg in call.args.iter() {
                    ea.analyze(&arg.value, ctx);
                }

                // Validate callable arity
                if let Some(params) = extract_callable_params(&callee_ty, ea) {
                    let required_count = params
                        .iter()
                        .filter(|p| !p.is_optional && !p.is_variadic)
                        .count();
                    let has_variadic = params.iter().any(|p| p.is_variadic);
                    let max_params = params.len();
                    let actual_count = call.args.len();

                    if actual_count < required_count {
                        ea.emit(
                            IssueKind::TooFewArguments {
                                fn_name: "callable".to_string(),
                                expected: required_count,
                                actual: actual_count,
                            },
                            Severity::Error,
                            span,
                        );
                    } else if !has_variadic && actual_count > max_params {
                        ea.emit(
                            IssueKind::TooManyArguments {
                                fn_name: "callable".to_string(),
                                expected: max_params,
                                actual: actual_count,
                            },
                            Severity::Error,
                            span,
                        );
                    }
                }

                for atomic in &callee_ty.types {
                    match atomic {
                        Atomic::TClosure { return_type, .. } => return *return_type.clone(),
                        Atomic::TCallable {
                            return_type: Some(rt),
                            ..
                        } => return *rt.clone(),
                        _ => {}
                    }
                }
                return Type::mixed();
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
        if matches!(
            fn_name.to_ascii_lowercase().as_str(),
            "var_dump" | "shell_exec"
        ) {
            ea.emit(
                IssueKind::ForbiddenCode {
                    message: format!("Use of {} is forbidden", fn_name),
                },
                Severity::Warning,
                span,
            );
        }
        let resolved_fn_name: String = {
            let imports = ea.db.file_imports(&ea.file);
            let qualified = if let Some(imported) = imports.get(&Name::new(fn_name.as_str())) {
                imported.as_str().to_string()
            } else if fn_name.contains('\\') {
                crate::db::resolve_name(ea.db, &ea.file, &fn_name)
            } else if let Some(ns) = ea.db.file_namespace(&ea.file) {
                format!("{}\\{}", ns, fn_name)
            } else {
                fn_name.clone()
            };
            let fn_exists = |name: &str| -> bool {
                let db = ea.db;
                let here = crate::db::Fqcn::from_str(db, name);
                crate::db::find_function(db, here).is_some()
            };
            if fn_exists(qualified.as_str()) {
                qualified
            } else if fn_exists(fn_name.as_str()) {
                fn_name.clone()
            } else {
                qualified
            }
        };

        // Resolve once; reused below for by-ref pre-marking and full analysis.
        let resolved = resolve_fn(ea, resolved_fn_name.as_str());

        // Pre-mark by-reference parameter variables as defined BEFORE evaluating args
        if let Some(ref resolved) = resolved {
            for (i, param) in resolved.params.iter().enumerate() {
                if param.is_byref {
                    if param.is_variadic {
                        for arg in call.args.iter().skip(i) {
                            if let ExprKind::Variable(name) = &arg.value.kind {
                                let var_name = name.as_ref().trim_start_matches('$');
                                if !ctx.var_is_defined(var_name) {
                                    ctx.set_var(var_name, Type::mixed());
                                }
                            }
                        }
                    } else if let Some(arg) = call.args.get(i) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let var_name = name.as_ref().trim_start_matches('$');
                            if !ctx.var_is_defined(var_name) {
                                ctx.set_var(var_name, Type::mixed());
                            }
                        }
                    }
                }
            }
        }

        let mut arg_types = super::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        // `ClassName::class` is a PHP compile-time constant — the class need not
        // be loaded.  Suppress UndefinedClass while analysing arg 0 of the
        // three PHP existence-probe functions so that the common guard pattern
        // `class_exists(\Foo\Bar::class)` does not produce a false positive.
        let is_existence_probe = matches!(
            resolved_fn_name.as_str(),
            "class_exists" | "interface_exists" | "trait_exists"
        );
        for (i, arg) in call.args.iter().enumerate() {
            let ty = if is_existence_probe && i == 0 {
                ea.with_class_exists_arg(|ea| ea.analyze(&arg.value, ctx))
            } else {
                ea.analyze(&arg.value, ctx)
            };
            arg_types.push(if arg.unpack {
                spread_element_type(&ty)
            } else {
                ty
            });
        }

        let arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();

        // When call_user_func / call_user_func_array is called with a bare string
        // literal as the callable argument, treat that string as a direct FQN
        // reference so the named function is not flagged as dead code.
        // Note: 'helper' always resolves to \helper (global) — no namespace
        // fallback applies to runtime callable strings.
        let mut call_user_func_string_arg = false;
        if matches!(
            resolved_fn_name.as_str(),
            "call_user_func" | "call_user_func_array"
        ) {
            if let Some(arg) = call.args.first() {
                if let ExprKind::String(name) = &arg.value.kind {
                    call_user_func_string_arg = true;
                    // Always look in global namespace (with explicit backslash prefix)
                    let fqn = if name.as_ref().starts_with('\\') {
                        name.as_ref().to_string()
                    } else {
                        format!("\\{}", name.as_ref())
                    };
                    let here = crate::db::Fqcn::from_str(ea.db, &fqn);
                    let canonical_fqn: Option<Arc<str>> =
                        crate::db::find_function(ea.db, here).map(|f| f.fqn.clone());
                    if let Some(canonical_fqn) = canonical_fqn {
                        ea.record_ref(Arc::from(canonical_fqn.as_ref()), arg.span);
                    }
                }
            }
        }

        // compact() reads variables by string name at runtime; mark each string-literal arg as read
        if fn_name == "compact" {
            for arg in call.args.iter() {
                if let ExprKind::String(name) = &arg.value.kind {
                    ctx.read_vars.insert(mir_types::Name::from(name.as_ref()));
                    ctx.mark_consumed(name.as_ref());
                }
            }
        }

        if let Some(resolved) = resolved {
            ea.record_ref(resolved.fqn.clone(), call.name.span);
            let deprecated = resolved.deprecated;
            let params = resolved.params;
            let template_params = resolved.template_params;
            let return_ty_raw = resolved.return_ty_raw;

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

            {
                let used_short = fn_name.rsplit('\\').next().unwrap_or(&fn_name);
                let canonical_short = resolved
                    .fqn
                    .rsplit('\\')
                    .next()
                    .unwrap_or(resolved.fqn.as_ref());
                if used_short != canonical_short && used_short.eq_ignore_ascii_case(canonical_short)
                {
                    ea.emit(
                        IssueKind::WrongCaseFunction {
                            used: used_short.to_string(),
                            canonical: canonical_short.to_string(),
                        },
                        Severity::Info,
                        call.name.span,
                    );
                }
            }

            check_args(
                ea,
                CheckArgsParams {
                    fn_name: &fn_name,
                    params: &params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &call
                        .args
                        .iter()
                        .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
                        .collect::<Vec<_>>(),
                    arg_can_be_byref: &call
                        .args
                        .iter()
                        .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
                        .collect::<Vec<_>>(),
                    call_span: span,
                    has_spread: call.args.iter().any(|a| a.unpack),
                    template_params: &template_params,
                },
            );

            // Validate callbacks for built-in PHP functions with special callback requirements.
            // Functions with dynamic or mode-dependent arity use specialized handlers.
            // Functions with a fixed minimum arity are declared in callback_min_arity_spec.
            match resolved_fn_name.as_str() {
                "array_map" => {
                    super::callable::check_array_map_callback(ea, &arg_types, &arg_spans)
                }
                "array_filter" => {
                    super::callable::check_array_filter_callback(ea, &arg_types, &arg_spans)
                }
                fn_name => {
                    if let Some((cb_idx, min_arity)) =
                        super::callable::callback_min_arity_spec(fn_name)
                    {
                        super::callable::check_min_arity_callback(
                            ea, fn_name, cb_idx, min_arity, &arg_types, &arg_spans,
                        );
                    }
                }
            }

            for (i, param) in params.iter().enumerate() {
                if param.is_byref {
                    let output_ty = param
                        .ty
                        .as_ref()
                        .map(|t| (**t).clone())
                        .unwrap_or_else(Type::mixed);
                    if param.is_variadic {
                        for arg in call.args.iter().skip(i) {
                            if let ExprKind::Variable(name) = &arg.value.kind {
                                let var_name = name.as_ref().trim_start_matches('$');
                                ctx.set_var(var_name, output_ty.clone());
                            }
                        }
                    } else if let Some(arg) = call.args.get(i) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let var_name = name.as_ref().trim_start_matches('$');
                            ctx.set_var(var_name, output_ty);
                        }
                    }
                }
            }

            let template_bindings = if !template_params.is_empty() {
                let bindings = infer_template_bindings(&template_params, &params, &arg_types);
                for (name, inferred, bound) in
                    check_template_bounds_with_inheritance(ea.db, &bindings, &template_params)
                {
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
                Some(bindings)
            } else {
                None
            };

            for assertion in resolved
                .assertions
                .iter()
                .filter(|a| a.kind == AssertionKind::Assert)
            {
                if let Some(index) = params.iter().position(|p| p.name == assertion.param) {
                    if let Some(arg) = call.args.get(index) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let asserted_ty = match &template_bindings {
                                Some(b) => assertion.ty.substitute_templates(b),
                                None => assertion.ty.clone(),
                            };
                            ctx.set_var(name.as_ref().trim_start_matches('$'), asserted_ty);
                        }
                    }
                }
            }

            let return_ty = match &template_bindings {
                Some(bindings) => return_ty_raw.substitute_templates(bindings),
                None => return_ty_raw,
            };

            let return_ty = return_ty.resolve_conditional_returns(|param_name| {
                params
                    .iter()
                    .position(|p| p.name.as_ref() == param_name)
                    .and_then(|idx| arg_types.get(idx))
                    .cloned()
            });

            super::ARG_TYPES_BUF.with(|b| {
                let mut g = b.borrow_mut();
                if g.as_ref().map_or(0, |v| v.capacity()) < arg_types.capacity() {
                    *g = Some(arg_types);
                }
            });

            // Check inter-procedural throws: if callee declares @throws, check if caller covers them.
            // Unchecked exceptions (RuntimeException / LogicException descendants) are skipped by
            // PHP convention — see [`is_unchecked_exception`].
            for callee_throw in resolved.throws.iter() {
                if crate::db::is_unchecked_exception(ea.db, callee_throw.as_ref()) {
                    continue;
                }
                if !ctx.fn_declared_throws.iter().any(|declared| {
                    declared.as_ref() == callee_throw.as_ref()
                        || crate::db::extends_or_implements(
                            ea.db,
                            callee_throw.as_ref(),
                            declared.as_ref(),
                        )
                }) {
                    ea.emit(
                        IssueKind::MissingThrowsDocblock {
                            class: callee_throw.to_string(),
                        },
                        Severity::Info,
                        span,
                    );
                }
            }

            ea.record_symbol(
                call.name.span,
                ReferenceKind::FunctionCall(resolved.fqn.clone()),
                return_ty.clone(),
            );
            return return_ty;
        }

        // Soft-fallback: if the build-time stub index recognises this name as
        // a PHP built-in, the codebase miss is a stub-loading race rather
        // than user error — the auto-discovery scanner missed it, the
        // session is in essentials-only mode without auto-discovery, or the
        // analyzer is mid-ingest. Suppressing the diagnostic avoids a class
        // of false positives that would otherwise plague consumers running
        // the lazy-stub setup. However, don't suppress if the function is
        // version-filtered (e.g. @removed in the target version) — it should
        // be reported as undefined.
        if let Some(stub_path) = crate::stubs::stub_path_for_function(&fn_name) {
            if let Some(stub_src) = crate::stubs::stub_content_for_path(stub_path) {
                // Parse the stub to check if this function is version-compatible.
                if let Some(docblock_text) = extract_function_docblock(stub_src, &fn_name) {
                    let doc = crate::parser::DocblockParser::parse(docblock_text);
                    // Check if the function is available in the current PHP version.
                    if ea
                        .php_version
                        .includes_symbol(doc.since.as_deref(), doc.removed.as_deref())
                    {
                        return Type::mixed();
                    }
                } else {
                    // No docblock found; assume the function is available (conservative).
                    return Type::mixed();
                }
            }
        }
        // Don't emit UndefinedFunction if call_user_func/call_user_func_array with string arg
        // - string args are runtime callable names that may not exist at compile time
        if !call_user_func_string_arg {
            ea.emit(
                IssueKind::UndefinedFunction { name: fn_name },
                Severity::Error,
                span,
            );
        }
        Type::mixed()
    }
}

/// Extract the docblock for a function from PHP stub source code.
/// Returns the docblock text (without /** */ delimiters) if found.
fn extract_function_docblock<'a>(src: &'a str, fn_name: &str) -> Option<&'a str> {
    // Simple extraction: find /** ... */ followed by function declaration.
    let fn_pattern = format!("function {fn_name}");
    extract_docblock_before(src, &fn_pattern)
}

/// Extract the docblock for a class from PHP stub source code.
/// Returns the docblock text (without /** */ delimiters) if found.
pub(crate) fn extract_class_docblock<'a>(src: &'a str, class_name: &str) -> Option<&'a str> {
    // Handle both class and interface declarations.
    // Extract the short name (after last backslash if present).
    let short_name = class_name.split('\\').next_back().unwrap_or(class_name);

    // Try case-insensitive matching for "class" declarations.
    let class_pattern_lower = format!("class {}", short_name.to_lowercase());
    if let Some(docblock) = extract_docblock_case_insensitive(src, &class_pattern_lower) {
        return Some(docblock);
    }

    // Try case-insensitive matching for "interface" declarations.
    let interface_pattern_lower = format!("interface {}", short_name.to_lowercase());
    extract_docblock_case_insensitive(src, &interface_pattern_lower)
}

/// Generic docblock extraction: find /** ... */ before a pattern (case-sensitive).
fn extract_docblock_before<'a>(src: &'a str, pattern: &str) -> Option<&'a str> {
    if let Some(pos) = src.find(pattern) {
        extract_docblock_at_position(src, pos)
    } else {
        None
    }
}

/// Case-insensitive docblock extraction: find /** ... */ before a pattern.
fn extract_docblock_case_insensitive<'a>(src: &'a str, pattern: &str) -> Option<&'a str> {
    let src_lower = src.to_lowercase();
    if let Some(pos) = src_lower.find(pattern) {
        extract_docblock_at_position(src, pos)
    } else {
        None
    }
}

/// Extract docblock before a given byte position in the source.
fn extract_docblock_at_position(src: &str, pos: usize) -> Option<&str> {
    // Look back for /** from the position.
    if let Some(doc_start_pos) = src[..pos].rfind("/**") {
        if let Some(doc_end_pos) = src[doc_start_pos..].find("*/") {
            let end_abs = doc_start_pos + doc_end_pos;
            let docblock_raw = &src[doc_start_pos + 3..end_abs];
            return Some(docblock_raw);
        }
    }
    None
}
