use php_ast::owned::{ExprKind, FunctionCallExpr};
use php_ast::Span;

use std::sync::Arc;

use mir_codebase::definitions::{Assertion, AssertionKind, DeclaredParam, TemplateParam};
use mir_issues::{IssueKind, Severity};
use mir_types::atomic::FnParam as TypeFnParam;
use mir_types::{Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::generic::{check_template_bounds_with_inheritance, infer_template_bindings};
use crate::symbol::ReferenceKind;
use crate::taint::{classify_sink, is_expr_tainted, SinkKind};

use super::args::{
    check_args, distinct_spans_for_expansion, expand_sole_spread_arg,
    expr_can_be_passed_by_reference_owned, spread_element_type, CheckArgsParams,
};
use super::callable::extract_callable_params;
use super::CallAnalyzer;

struct ResolvedFn {
    fqn: std::sync::Arc<str>,
    deprecated: Option<std::sync::Arc<str>>,
    params: Vec<DeclaredParam>,
    template_params: Vec<TemplateParam>,
    assertions: Vec<Assertion>,
    return_ty_raw: Type,
    throws: Arc<[Arc<str>]>,
    no_named_arguments: bool,
    is_pure: bool,
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
            no_named_arguments: f.no_named_arguments,
            is_pure: f.is_pure,
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

                if callee_ty.is_mixed() {
                    ea.emit(IssueKind::MixedFunctionCall, Severity::Info, span);
                }

                // Extract typed params once — used for both pre-marking (before arg
                // analysis) and output writeback (after the call).
                let callee_params = typed_params_from_callee(&callee_ty, ea);

                // `$obj(...)` invoking an object's __invoke() is a real reference to
                // that method — record it, or find-references/go-to-definition on
                // __invoke never sees call sites reached only this way (unlike every
                // other call form, which always records the resolved method).
                for atomic in &callee_ty.types {
                    if let Atomic::TNamedObject { fqcn, .. } = atomic {
                        if let Some((_, storage)) = crate::db::find_method_respecting_precedence(
                            ea.db,
                            crate::db::Fqcn::from_str(ea.db, fqcn.as_ref()),
                            "__invoke",
                        ) {
                            ea.record_ref(
                                Arc::from(format!(
                                    "meth:{}::{}",
                                    fqcn,
                                    crate::util::php_ident_lowercase(&storage.name)
                                )),
                                call.name.span,
                            );
                            ea.record_symbol(
                                call.name.span,
                                ReferenceKind::MethodCall {
                                    class: Arc::from(fqcn.as_ref()),
                                    method: Arc::from("__invoke"),
                                },
                                callee_ty.clone(),
                            );
                        }
                    }
                }

                // Pre-mark by-ref parameter variables as defined BEFORE evaluating
                // args, so a previously-undefined variable passed to an out-param
                // (e.g. `$fn($x, $out)` where $out is fresh) is not flagged as
                // UndefinedVariable when the argument expression is analyzed.
                if let Some((_, ref params)) = callee_params {
                    super::premark_byref_arg_vars(params, &call.args, ctx);
                }

                // Collect arg types, spans, names and byref flags for type checking.
                let mut inner_arg_types: Vec<Type> = Vec::with_capacity(call.args.len());
                let mut sole_spread_ty: Option<Type> = None;
                for arg in call.args.iter() {
                    let ty = ea.analyze(&arg.value, ctx);
                    super::consume_arg_assignment(&arg.value, ctx);
                    if arg.unpack {
                        if call.args.len() == 1 {
                            sole_spread_ty = Some(ty.clone());
                        }
                        inner_arg_types.push(spread_element_type(&ty));
                    } else {
                        inner_arg_types.push(ty);
                    }
                }
                let mut inner_arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();
                let mut inner_arg_names: Vec<Option<String>> = call
                    .args
                    .iter()
                    .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
                    .collect();
                let mut inner_arg_byref: Vec<bool> = call
                    .args
                    .iter()
                    .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
                    .collect();
                let mut has_spread = call.args.iter().any(|a| a.unpack);
                let mut arity_unknown = has_spread;
                // A sole spread arg over a literal, sequentially-keyed shape can be
                // expanded into one binding per element so each parameter is checked
                // individually instead of only the first (see expand_sole_spread_arg).
                // `arity_unknown` stays true even after expansion — PHP allows
                // extra/spread positional args, so a concretely-known count still
                // shouldn't trigger TooFew/TooManyArguments.
                if let Some(expanded) = sole_spread_ty.and_then(|t| expand_sole_spread_arg(&t)) {
                    inner_arg_spans =
                        distinct_spans_for_expansion(inner_arg_spans[0], expanded.len());
                    inner_arg_names = vec![None; expanded.len()];
                    inner_arg_byref = vec![false; expanded.len()];
                    inner_arg_types = expanded;
                    has_spread = false;
                    arity_unknown = true;
                }

                if let Some((ref callee_fn_name, ref params)) = callee_params {
                    // Full type + arity checking via check_args.
                    check_args(
                        ea,
                        CheckArgsParams {
                            fn_name: callee_fn_name,
                            params,
                            arg_types: &inner_arg_types,
                            arg_spans: &inner_arg_spans,
                            arg_names: &inner_arg_names,
                            arg_can_be_byref: &inner_arg_byref,
                            call_span: span,
                            has_spread,
                            arity_unknown,
                            template_params: &[],
                            no_named_arguments: false,
                        },
                    );
                } else if let Some(params) = extract_callable_params(&callee_ty, ea) {
                    // Arity-only fallback when full param types are unavailable.
                    // A spread arg (`...$args`) makes the real argument count
                    // unknowable from `call.args.len()` alone — same
                    // `arity_unknown` signal `check_args` above uses to skip
                    // TooFew/TooManyArguments for the exact same reason.
                    let required_count = params
                        .iter()
                        .filter(|p| !p.is_optional && !p.is_variadic)
                        .count();
                    let has_variadic = params.iter().any(|p| p.is_variadic);
                    let max_params = params.len();
                    let actual_count = call.args.len();

                    if arity_unknown {
                        // Skip TooFew/TooManyArguments — can't be checked precisely.
                    } else if actual_count < required_count {
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

                // Write back output types to by-ref argument variables.
                if let Some((_, ref params)) = callee_params {
                    for (i, param) in params.iter().enumerate() {
                        if param.is_byref {
                            let output_ty = param
                                .out_ty
                                .as_ref()
                                .or(param.ty.as_ref())
                                .map(|t| (**t).clone())
                                .unwrap_or_else(Type::mixed);
                            if param.is_variadic {
                                for arg in call.args.iter().skip(i) {
                                    if let ExprKind::Variable(name) = &arg.value.kind {
                                        ctx.set_var(
                                            name.trim_start_matches('$'),
                                            output_ty.clone(),
                                        );
                                    }
                                }
                            } else if let Some(arg) = call.args.get(i) {
                                if let ExprKind::Variable(name) = &arg.value.kind {
                                    ctx.set_var(name.trim_start_matches('$'), output_ty);
                                }
                            }
                        }
                    }
                }

                for atomic in &callee_ty.types {
                    match atomic {
                        Atomic::TClosure { data } => return data.return_type.clone(),
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
            let relevant = sink_kind.tainted_arg_indices();
            for (i, arg) in call.args.iter().enumerate() {
                // Path/payload sinks only care about their specific argument
                // (e.g. a tainted *path*, not tainted *data* written to a
                // constant path); output/query/command sinks check any arg.
                if relevant.is_some_and(|idxs| !idxs.contains(&i)) {
                    continue;
                }
                if is_expr_tainted(&arg.value, ctx) {
                    let issue_kind = match sink_kind {
                        SinkKind::Html => IssueKind::TaintedHtml,
                        SinkKind::Sql => IssueKind::TaintedSql,
                        SinkKind::Shell => IssueKind::TaintedShell,
                        SinkKind::File => IssueKind::TaintedInput {
                            sink: "file".to_string(),
                        },
                        SinkKind::Unserialize => IssueKind::TaintedInput {
                            sink: "unserialize".to_string(),
                        },
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
            super::premark_byref_arg_vars(&resolved.params, &call.args, ctx);
        }

        let mut arg_types = super::ARG_TYPES_BUF
            .with(|b| b.borrow_mut().take())
            .unwrap_or_default();
        arg_types.clear();
        let mut sole_spread_ty: Option<Type> = None;
        for arg in call.args.iter() {
            let ty = ea.analyze(&arg.value, ctx);
            super::consume_arg_assignment(&arg.value, ctx);
            if arg.unpack {
                if call.args.len() == 1 {
                    sole_spread_ty = Some(ty.clone());
                }
                arg_types.push(spread_element_type(&ty));
            } else {
                arg_types.push(ty);
            }
        }

        let mut arg_spans: Vec<Span> = call.args.iter().map(|a| a.span).collect();
        let mut arg_names: Vec<Option<String>> = call
            .args
            .iter()
            .map(|a| a.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        let mut arg_can_be_byref: Vec<bool> = call
            .args
            .iter()
            .map(|a| expr_can_be_passed_by_reference_owned(&a.value))
            .collect();
        let mut has_spread = call.args.iter().any(|a| a.unpack);
        let mut arity_unknown = has_spread;
        // A sole spread arg over a literal, sequentially-keyed shape can be
        // expanded into one binding per element so each parameter (and
        // template-binding inference below) is checked individually instead
        // of only the first (see expand_sole_spread_arg). `arity_unknown`
        // stays true even after expansion — PHP allows extra/spread
        // positional args, so a concretely-known count still shouldn't
        // trigger TooFew/TooManyArguments.
        if let Some(expanded) = sole_spread_ty.and_then(|t| expand_sole_spread_arg(&t)) {
            arg_spans = distinct_spans_for_expansion(arg_spans[0], expanded.len());
            arg_names = vec![None; expanded.len()];
            arg_can_be_byref = vec![false; expanded.len()];
            arg_types = expanded;
            has_spread = false;
            arity_unknown = true;
        }

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
                    if let Some((class_name, method_name)) = name.as_ref().split_once("::") {
                        // "Class::method" static-callable string — resolve
                        // and record both, or a static method reachable only
                        // this way is falsely flagged UnusedMethod (and its
                        // class UnusedClass).
                        let resolved_class = crate::db::resolve_name(ea.db, &ea.file, class_name);
                        let here = crate::db::Fqcn::from_str(ea.db, &resolved_class);
                        if let Some((owner_fqcn, method)) =
                            crate::db::find_method_in_chain(ea.db, here, method_name)
                        {
                            ea.record_ref(Arc::from(format!("cls:{resolved_class}")), arg.span);
                            ea.record_ref(
                                Arc::from(format!(
                                    "meth:{owner_fqcn}::{}",
                                    crate::util::php_ident_lowercase(&method.name)
                                )),
                                arg.span,
                            );
                        }
                    } else {
                        // Runtime callable strings always resolve in the global
                        // namespace — no current-namespace fallback applies, unlike a
                        // direct `helper()` call. The function index itself is never
                        // keyed with a leading backslash (see the identical
                        // `strip_prefix` above for `resolved_fn_name`), so a lookup
                        // must strip one here too, not add one — a prepended `\`
                        // makes every lookup key mismatch and silently fail.
                        let fqn = name.as_ref().trim_start_matches('\\');
                        let here = crate::db::Fqcn::from_str(ea.db, fqn);
                        let canonical_fqn: Option<Arc<str>> =
                            crate::db::find_function(ea.db, here).map(|f| f.fqn.clone());
                        if let Some(canonical_fqn) = canonical_fqn {
                            ea.record_ref(Arc::from(format!("fn:{canonical_fqn}")), arg.span);
                        }
                    }
                }
            }
        }

        // A string-literal class name passed to one of these reflection-like
        // builtins is a real (runtime) reference to that class — record it,
        // or a class checked/reflected on only this way is falsely flagged
        // UnusedClass. `is_a`/`is_subclass_of`/`method_exists` take the class
        // name in a different argument position than `class_exists`'s family.
        // `class_alias`'s original-class argument is a hard requirement (PHP
        // fatals if it doesn't exist), unlike the `*_exists` guards, but it's
        // still just existence + reference recording here — no diagnostic is
        // raised either way, matching how the rest of this table treats a
        // string that fails to resolve to a real class.
        let class_name_arg_index: Option<usize> = match resolved_fn_name
            .to_ascii_lowercase()
            .as_str()
        {
            "class_exists" | "interface_exists" | "trait_exists" | "enum_exists" => Some(0),
            "is_a" | "is_subclass_of" => Some(1),
            "method_exists" => Some(0),
            "class_alias" => Some(0),
            "class_implements" | "class_parents" | "class_uses" | "get_class_methods" => Some(0),
            _ => None,
        };
        if let Some(idx) = class_name_arg_index {
            if let Some(arg) = call.args.get(idx) {
                if let ExprKind::String(name) = &arg.value.kind {
                    let resolved_class = crate::db::resolve_name(ea.db, &ea.file, name.as_ref());
                    if crate::db::class_exists(ea.db, &resolved_class) {
                        ea.record_ref(Arc::from(format!("cls:{resolved_class}")), arg.span);
                    }
                }
            }
        }

        // compact() reads variables by string name at runtime; mark each string-literal arg as read.
        // A non-literal argument (`compact($names)`/`compact(...$names)`) reads an
        // unknowable set of names — reuse the same blanket exemption extract() gets
        // below rather than risk flagging a variable that's actually read this way.
        if fn_name == "compact" {
            for arg in call.args.iter() {
                if let ExprKind::String(name) = &arg.value.kind {
                    ctx.read_vars.insert(mir_types::Name::from(name.as_ref()));
                    ctx.mark_consumed(name.as_ref());
                } else {
                    ctx.has_dynamic_var_read = true;
                }
            }
        }

        // extract() defines variables whose names are only known at runtime (the
        // keys of the passed array). After such a call, reads of otherwise-unknown
        // variables must not be reported as undefined — the same handling as
        // variable-variables.
        if fn_name.eq_ignore_ascii_case("extract") {
            ctx.has_dynamic_var_def = true;
        }

        if let Some(resolved) = resolved {
            ea.record_ref(Arc::from(format!("fn:{}", resolved.fqn)), call.name.span);
            let deprecated = resolved.deprecated;
            let params = resolved.params;
            let template_params = resolved.template_params;
            let return_ty_raw = resolved.return_ty_raw;
            let no_named_arguments = resolved.no_named_arguments;
            let is_pure = resolved.is_pure;

            if ctx.is_in_pure_fn && !is_pure {
                ea.emit(
                    IssueKind::ImpureFunctionCall {
                        fn_name: fn_name.rsplit('\\').next().unwrap_or(&fn_name).to_string(),
                    },
                    Severity::Warning,
                    span,
                );
            }

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

            if let Some((used, canonical_str)) =
                crate::fqcn_case_mismatch(&resolved_fn_name, resolved.fqn.as_ref())
            {
                ea.emit(
                    IssueKind::WrongCaseFunction {
                        used,
                        canonical: canonical_str,
                    },
                    Severity::Info,
                    call.name.span,
                );
            }

            check_args(
                ea,
                CheckArgsParams {
                    fn_name: &fn_name,
                    params: &params,
                    arg_types: &arg_types,
                    arg_spans: &arg_spans,
                    arg_names: &arg_names,
                    arg_can_be_byref: &arg_can_be_byref,
                    call_span: span,
                    has_spread,
                    arity_unknown,
                    template_params: &template_params,
                    no_named_arguments,
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

            let template_bindings = if !template_params.is_empty() {
                let (bindings, unchecked) = infer_template_bindings(
                    ea.db,
                    &template_params,
                    &params,
                    &arg_types,
                    &arg_names,
                );
                for (name, inferred, bound) in check_template_bounds_with_inheritance(
                    ea.db,
                    &bindings,
                    &template_params,
                    &unchecked,
                    None,
                ) {
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

            for (i, param) in params.iter().enumerate() {
                if param.is_byref {
                    // Prefer @param-out type if declared; fall back to declared in-type.
                    // Substitute the function's own inferred template bindings so a
                    // generic identity/setter-style helper reports the concrete
                    // argument type, not the raw template atom.
                    let output_ty = param
                        .out_ty
                        .as_ref()
                        .or(param.ty.as_ref())
                        .map(|t| (**t).clone())
                        .unwrap_or_else(Type::mixed);
                    let output_ty = match &template_bindings {
                        Some(bindings) => output_ty.substitute_templates(bindings),
                        None => output_ty,
                    };
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

            for assertion in resolved
                .assertions
                .iter()
                .filter(|a| a.kind == AssertionKind::Assert)
            {
                if let Some(index) = params.iter().position(|p| p.name == assertion.param) {
                    if let Some(arg) = call.args.get(index) {
                        if let ExprKind::Variable(name) = &arg.value.kind {
                            let var_name = name.as_ref().trim_start_matches('$');
                            let asserted_ty = match &template_bindings {
                                Some(b) => assertion.ty.substitute_templates(b),
                                None => assertion.ty.clone(),
                            };
                            let asserted_ty = if assertion.negated {
                                crate::narrowing::negate_assertion_type(
                                    &ctx.get_var(var_name),
                                    &asserted_ty,
                                    ea.db,
                                )
                            } else {
                                asserted_ty
                            };
                            ctx.set_var(var_name, asserted_ty);
                        } else if let Some((obj, prop)) =
                            crate::narrowing::extract_prop_access(&arg.value)
                        {
                            let asserted_ty = match &template_bindings {
                                Some(b) => assertion.ty.substitute_templates(b),
                                None => assertion.ty.clone(),
                            };
                            let asserted_ty = if assertion.negated {
                                let current = crate::narrowing::resolve_prop_current_type(
                                    ctx, &obj, &prop, ea.db, &ea.file,
                                );
                                crate::narrowing::negate_assertion_type(
                                    &current,
                                    &asserted_ty,
                                    ea.db,
                                )
                            } else {
                                asserted_ty
                            };
                            // `$obj->prop` on a null `$obj` reads as null, so
                            // proving the property itself is non-nullable
                            // also proves `$obj` wasn't null.
                            let proved_prop_non_null = !asserted_ty.is_nullable();
                            ctx.set_prop_refined(&obj, &prop, asserted_ty);
                            crate::narrowing::narrow_receiver_non_null_on_prop_match(
                                ctx,
                                &obj,
                                proved_prop_non_null,
                            );
                        } else if let Some((fqcn, prop)) =
                            crate::narrowing::extract_static_prop_access(
                                &arg.value, ctx, ea.db, &ea.file,
                            )
                        {
                            let asserted_ty = match &template_bindings {
                                Some(b) => assertion.ty.substitute_templates(b),
                                None => assertion.ty.clone(),
                            };
                            let asserted_ty = if assertion.negated {
                                let current = crate::narrowing::resolve_static_prop_current_type(
                                    ctx, &fqcn, &prop, ea.db,
                                );
                                crate::narrowing::negate_assertion_type(
                                    &current,
                                    &asserted_ty,
                                    ea.db,
                                )
                            } else {
                                asserted_ty
                            };
                            ctx.set_prop_refined(&fqcn, &prop, asserted_ty);
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

            // Built-in array transformers whose stub return type is a generic
            // `array`: refine the element type from the callback / source array
            // so binding sites (e.g. `foreach` over the result) get a usable
            // value type. Falls back to the stub return when inference is unsure.
            let return_ty = match resolved_fn_name.as_str() {
                "array_map" => {
                    let callback_expr = call.args.first().map(|a| &a.value);
                    super::callable::infer_array_map_return(ea, &arg_types, ctx, callback_expr)
                        .unwrap_or(return_ty)
                }
                "array_filter" => {
                    super::callable::infer_array_filter_return(&arg_types).unwrap_or(return_ty)
                }
                "array_reduce" => {
                    let callback_expr = call.args.get(1).map(|a| &a.value);
                    super::callable::infer_array_reduce_return(ea, &arg_types, ctx, callback_expr)
                        .unwrap_or(return_ty)
                }
                "array_values" => {
                    super::callable::infer_array_values_return(&arg_types).unwrap_or(return_ty)
                }
                "array_merge" => {
                    super::callable::infer_array_merge_return(&arg_types).unwrap_or(return_ty)
                }
                // array_fill with a positive count returns a non-empty list.
                "array_fill" => {
                    super::callable::array_fill_return_type(&arg_types).unwrap_or(return_ty)
                }
                // implode/join with a non-empty array of non-empty strings returns non-empty-string.
                "implode" | "join" => {
                    super::callable::implode_return_type(&arg_types).unwrap_or(return_ty)
                }
                // str_split with a non-empty string returns a non-empty list<non-empty-string>.
                "str_split" => {
                    super::callable::str_split_return_type(&arg_types).unwrap_or(return_ty)
                }
                // explode with a non-empty separator always returns non-empty-list<string>.
                "explode" => super::callable::explode_return_type(&arg_types, &return_ty)
                    .unwrap_or(return_ty),
                // array_slice preserves the element type (and list structure when not
                // preserving keys).
                "array_slice" => {
                    super::callable::array_slice_return_type(&arg_types).unwrap_or(return_ty)
                }
                // array_keys of a non-empty array returns a non-empty list (preserving the
                // stub's key type from template resolution).
                "array_keys" => super::callable::array_keys_return_type(&arg_types, &return_ty),
                // array_reverse preserves the non-emptiness of the source array.
                "array_reverse" => {
                    super::callable::array_reverse_return_type(&arg_types).unwrap_or(return_ty)
                }
                // array_unique preserves key/value types and non-empty status.
                "array_unique" => {
                    super::callable::array_unique_return(&arg_types).unwrap_or(return_ty)
                }
                // range($start, $end) with integer bounds returns non-empty-list<int<min,max>>.
                "range" => super::callable::range_return_type(&arg_types).unwrap_or(return_ty),
                // array_key_first/array_key_last: non-null for non-empty input; int for lists.
                "array_key_first" | "array_key_last" => {
                    super::callable::array_key_first_last_return(&arg_types).unwrap_or(return_ty)
                }
                // array_pop/array_shift: return value type (not mixed) when source is typed.
                "array_pop" | "array_shift" => {
                    super::callable::array_pop_shift_return(&arg_types).unwrap_or(return_ty)
                }
                // Faithful integer-range returns: counts and lengths are
                // non-negative (and counts of non-empty collections are `>= 1`).
                "count" | "sizeof" => {
                    super::callable::count_return_type(&arg_types).unwrap_or(return_ty)
                }
                "strlen" | "mb_strlen" => super::callable::strlen_return_type(&arg_types),
                "abs" => super::callable::abs_return_type(&arg_types).unwrap_or(return_ty),
                // floor() and ceil() always return a whole-valued float — represent as
                // TIntegralFloat so passing the result to an int param doesn't emit a FP.
                "floor" | "ceil" => Type::single(Atomic::TIntegralFloat),
                // round() without a precision arg (or with precision=0) is also always integral.
                "round" => {
                    let precision_is_integral = arg_types
                        .get(1)
                        .is_none_or(|t| t.types.len() == 1 && t.types[0] == Atomic::TLiteralInt(0));
                    if precision_is_integral {
                        Type::single(Atomic::TIntegralFloat)
                    } else {
                        return_ty
                    }
                }
                "intdiv" => {
                    // intdiv() throws the exact same DivisionByZeroError as `$a / 0` for
                    // a literal-zero divisor — report it the same way BinaryOp::Div does,
                    // rather than only narrowing the return type.
                    if let Some(divisor_ty) = arg_types.get(1) {
                        if crate::expr::operand_is_definitely_zero(divisor_ty) {
                            ea.emit(
                                IssueKind::DivisionByZero {
                                    op: "intdiv".to_string(),
                                },
                                Severity::Error,
                                arg_spans.get(1).copied().unwrap_or(span),
                            );
                        }
                    }
                    super::callable::intdiv_return_type(&arg_types).unwrap_or(return_ty)
                }
                "min" => super::callable::min_return_type(&arg_types).unwrap_or(return_ty),
                "max" => super::callable::max_return_type(&arg_types).unwrap_or(return_ty),
                "rand" | "mt_rand" | "random_int" => {
                    super::callable::rand_return_type(&arg_types).unwrap_or(return_ty)
                }
                // preg_match returns 1 on match, 0 on no-match, false on error.
                "preg_match" => {
                    let mut ty = Type::single(Atomic::TIntRange {
                        min: Some(0),
                        max: Some(1),
                    });
                    ty.add_type(Atomic::TFalse);
                    ty
                }
                // preg_match_all returns the count of matches (>= 0) or false on error.
                "preg_match_all" => {
                    let mut ty = Type::single(Atomic::TNonNegativeInt);
                    ty.add_type(Atomic::TFalse);
                    ty
                }
                // Case-folding, encoding, and similar string functions that preserve non-emptiness:
                // a non-empty input always produces a non-empty output, and these functions
                // always return string (not string|false).
                "strtolower"
                | "strtoupper"
                | "mb_strtolower"
                | "mb_strtoupper"
                | "ucfirst"
                | "lcfirst"
                | "ucwords"
                | "mb_convert_case"
                | "mb_convert_kana"
                | "htmlspecialchars"
                | "htmlentities"
                | "html_entity_decode"
                | "htmlspecialchars_decode"
                | "addslashes"
                | "addcslashes"
                | "nl2br"
                | "urlencode"
                | "urldecode"
                | "rawurlencode"
                | "rawurldecode"
                | "base64_encode"
                | "quoted_printable_encode"
                | "quoted_printable_decode"
                | "str_rot13"
                | "str_pad"
                | "chunk_split"
                | "wordwrap" => {
                    super::callable::string_preserve_non_empty(&arg_types).unwrap_or(return_ty)
                }
                // sprintf/vsprintf: non-empty when the format string guarantees it.
                // vsprintf's args are passed as a single array, but the return-type
                // inference only ever looks at arg_types[0] (the format string), so
                // the same helper applies unchanged.
                "sprintf" | "vsprintf" => {
                    super::callable::sprintf_return_type(&arg_types).unwrap_or(return_ty)
                }
                // number_format() always returns a non-empty string.
                "number_format" => super::callable::number_format_return_type(),
                // str_repeat() with a non-empty string and positive count returns non-empty.
                "str_repeat" => {
                    super::callable::str_repeat_return_type(&arg_types).unwrap_or(return_ty)
                }
                // array_chunk splits an array into sub-arrays; outer list is non-empty when
                // source is non-empty; chunks are list<T> by default (preserve_keys=false).
                "array_chunk" => {
                    super::callable::array_chunk_return_type(&arg_types).unwrap_or(return_ty)
                }
                // array_fill_keys uses the values of $keys as result keys and $value as each result value.
                "array_fill_keys" => {
                    super::callable::array_fill_keys_return_type(&arg_types).unwrap_or(return_ty)
                }
                // preg_split with default flags always returns at least one part.
                "preg_split" => {
                    super::callable::preg_split_return_type(&arg_types).unwrap_or(return_ty)
                }
                // array_search: narrow key type from haystack rather than returning string|int|false.
                "array_search" => {
                    super::callable::array_search_return_type(&arg_types).unwrap_or(return_ty)
                }
                // date/time formatting functions always return non-empty strings.
                "date" | "gmdate" | "date_format" => Type::single(Atomic::TNonEmptyString),
                // Encoding/conversion functions: strip |false from stubs — they only
                // return false on bad input that PHP code never checks for in practice.
                "mb_convert_encoding" => super::callable::string_preserve_non_empty(&arg_types)
                    .or_else(|| super::callable::string_if_string_arg(&arg_types, 0))
                    .unwrap_or(return_ty),
                "iconv" => {
                    // iconv($from_encoding, $to_encoding, $str) — $str is arg 2
                    super::callable::string_if_string_arg(&arg_types, 2).unwrap_or(return_ty)
                }
                // preg_replace/preg_replace_callback: strip |null when subject is a string.
                // The null case only fires on a regex error, which PHP code rarely handles.
                "preg_replace" | "preg_replace_callback" => {
                    // subject is arg 2
                    super::callable::string_if_string_arg(&arg_types, 2).unwrap_or(return_ty)
                }
                // substr_replace: strip |array when $string is a scalar string.
                "substr_replace" => {
                    super::callable::string_if_string_arg(&arg_types, 0).unwrap_or(return_ty)
                }
                // filter_var: map a literal FILTER_VALIDATE_* $filter argument to its
                // real result type instead of the stub's blanket `mixed`.
                "filter_var" => {
                    super::callable::filter_var_return_type(&arg_types).unwrap_or(return_ty)
                }
                _ => return_ty,
            };

            let mut return_ty = return_ty;
            ea.apply_function_call_plugins(
                resolved.fqn.as_ref(),
                &call.args,
                &arg_types,
                span,
                &mut return_ty,
            );

            // array_push/array_unshift: the by-ref loop above set $arr to the stub's
            // generic `array` type — replace it with the precise post-push type derived
            // from the original array type (arg_types[0]) and the pushed value types.
            if matches!(resolved_fn_name.as_str(), "array_push" | "array_unshift") {
                if let (Some(arr_arg), Some(original_arr)) = (call.args.first(), arg_types.first())
                {
                    if let ExprKind::Variable(name) = &arr_arg.value.kind {
                        let var_name = name.as_ref().trim_start_matches('$');
                        let push_types: Vec<Type> = arg_types.iter().skip(1).cloned().collect();
                        let new_type = super::callable::array_push_unshift_byref_type(
                            original_arr,
                            &push_types,
                        );
                        ctx.set_var(var_name, new_type);
                    }
                }
            }

            // Sort functions: the by-ref loop above set $arr to generic `array`; restore
            // the original element type. Re-indexing sorts also convert to a list.
            {
                let reindex = matches!(
                    resolved_fn_name.as_str(),
                    "sort" | "rsort" | "usort" | "shuffle"
                );
                let preserve = matches!(
                    resolved_fn_name.as_str(),
                    "asort" | "arsort" | "ksort" | "krsort" | "uasort" | "uksort"
                );
                if reindex || preserve {
                    if let (Some(arr_arg), Some(original_arr)) =
                        (call.args.first(), arg_types.first())
                    {
                        if let ExprKind::Variable(name) = &arr_arg.value.kind {
                            let var_name = name.as_ref().trim_start_matches('$');
                            let new_type = super::callable::sort_byref_type(original_arr, reindex);
                            ctx.set_var(var_name, new_type);
                        }
                    }
                }
            }

            // preg_match / preg_match_all: the by-ref loop above wrote the stub's
            // generic `string[]` to `$matches`. Override with the flag-aware type:
            // no PREG_OFFSET_CAPTURE → list<string>; with it → list<array{0:string,1:int}>.
            // preg_match_all wraps one more list level.
            if matches!(resolved_fn_name.as_str(), "preg_match" | "preg_match_all") {
                if let Some(matches_arg) = call.args.get(2) {
                    if let ExprKind::Variable(name) = &matches_arg.value.kind {
                        let var_name = name.as_ref().trim_start_matches('$');
                        let flags: i64 = arg_types
                            .get(3)
                            .and_then(|t| {
                                t.types.iter().find_map(|a| {
                                    if let Atomic::TLiteralInt(v) = a {
                                        Some(*v)
                                    } else {
                                        None
                                    }
                                })
                            })
                            .unwrap_or(0);
                        let new_type = if resolved_fn_name.as_str() == "preg_match" {
                            super::callable::preg_match_matches_type(flags)
                        } else {
                            super::callable::preg_match_all_matches_type(flags)
                        };
                        ctx.set_var(var_name, new_type);
                    }
                }
            }

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
        // Also skip when guarded by `function_exists('fn')` (PHP function names
        // are case-insensitive). The short name is matched too, since a bare
        // call in a namespace falls back to the global function the guard names.
        let short_fn = fn_name.rsplit('\\').next().unwrap_or(&fn_name);
        let guarded = ctx
            .function_exists_guards
            .iter()
            .any(|g| g.eq_ignore_ascii_case(&fn_name) || g.eq_ignore_ascii_case(short_fn));
        if !call_user_func_string_arg && !guarded {
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
    let class_pattern_lower = format!("class {}", crate::util::php_ident_lowercase(short_name));
    if let Some(docblock) = extract_docblock_case_insensitive(src, &class_pattern_lower) {
        return Some(docblock);
    }

    // Try case-insensitive matching for "interface" declarations.
    let interface_pattern_lower =
        format!("interface {}", crate::util::php_ident_lowercase(short_name));
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

/// Convert `mir_types::atomic::FnParam` (from TClosure) to `mir_codebase::definitions::DeclaredParam`
/// so they can be passed to `check_args`.
fn type_param_to_storage_param(p: &TypeFnParam) -> DeclaredParam {
    DeclaredParam {
        name: p.name,
        ty: p.ty.as_ref().map(|t| Arc::new(t.to_union())),
        out_ty: p.out_ty.as_ref().map(|t| Arc::new(t.to_union())),
        has_default: p.default.is_some(),
        is_variadic: p.is_variadic,
        is_byref: p.is_byref,
        is_optional: p.is_optional,
    }
}

/// Try to extract a callable name and full typed params from a callee type union.
/// Returns `Some((name, params))` for:
/// - `TClosure` — name is `"{closure}"`, params from the closure's param list
/// - `TNamedObject` with `__invoke` method — name is `"Fqcn::__invoke"`, params from DB
///
/// Returns `None` if the union contains a bare `TCallable { params: None }` (unknown arity),
/// same guard as `extract_callable_params`.
fn typed_params_from_callee(
    union: &Type,
    ea: &ExpressionAnalyzer<'_>,
) -> Option<(String, Vec<DeclaredParam>)> {
    // Bare callable with unknown arity — we cannot determine params statically.
    if union
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TCallable { params: None, .. }))
    {
        return None;
    }

    for atomic in &union.types {
        match atomic {
            Atomic::TClosure { data } => {
                let storage_params = data
                    .params
                    .iter()
                    .map(type_param_to_storage_param)
                    .collect();
                return Some(("{closure}".to_string(), storage_params));
            }
            Atomic::TCallable {
                params: Some(params),
                ..
            } => {
                let storage_params = params.iter().map(type_param_to_storage_param).collect();
                return Some(("callable".to_string(), storage_params));
            }
            Atomic::TNamedObject { fqcn, .. } => {
                if let Some((_, storage)) = crate::db::find_method_respecting_precedence(
                    ea.db,
                    crate::db::Fqcn::from_str(ea.db, fqcn.as_ref()),
                    "__invoke",
                ) {
                    let fn_name = format!("{}::__invoke", fqcn);
                    return Some((fn_name, storage.params.to_vec()));
                }
            }
            _ => {}
        }
    }
    None
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
