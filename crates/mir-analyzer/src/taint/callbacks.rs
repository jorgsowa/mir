use super::*;

/// Whether a call in `expr` takes a `function () {}` literal argument.
pub fn has_closure_argument(expr: &Expr) -> bool {
    let args = match &unwrap_parens(expr).kind {
        ExprKind::FunctionCall(fc) => &fc.args,
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => &mc.args,
        ExprKind::StaticMethodCall(smc) => &smc.args,
        _ => return false,
    };
    args.iter().any(|a| {
        a.value.as_ref().is_some_and(|v| {
            matches!(unwrap_parens(v).kind, ExprKind::Closure(_)) || has_closure_argument(v)
        })
    })
}

/// The closure a call target evaluates to: a variable bound to one, or a literal.
pub(super) fn callee_closure(
    callee: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<crate::flow_state::ClosureSummary>> {
    let callee = unwrap_parens(callee);
    match &callee.kind {
        ExprKind::Variable(name) => ctx
            .closures
            .by_var
            .get(&mir_types::Name::from(name.trim_start_matches('$')))
            .cloned(),
        ExprKind::Closure(_) => ctx.closures.by_span.get(&callee.span.start).cloned(),
        ExprKind::ArrowFunction(af) => Some(
            ctx.closures
                .by_span
                .get(&callee.span.start)
                .cloned()
                .unwrap_or_else(|| std::sync::Arc::new(arrow_summary(af, ctx, db, file))),
        ),
        _ => None,
    }
}

/// The taint a captured variable carries inside a closure: its source taint,
/// and a slot past the closure's parameters standing for the enclosing body's
/// labels, which `captures` records.
pub fn captured_taint(outer: Taint, param_count: usize, captures: &mut Vec<Taint>) -> Taint {
    let slot = param_count + captures.len();
    if outer.labels().is_empty() || slot >= MAX_TRACKED_PARAMS {
        return outer.concrete();
    }
    captures.push(outer.labels());
    outer.concrete() | Taint::param(slot)
}

/// Variables an expression reads, in source order.
pub(super) struct VariableReads(pub(super) Vec<mir_types::Name>);

impl php_ast::owned::visitor::OwnedVisitor for VariableReads {
    fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
        if let ExprKind::Variable(name) = &expr.kind {
            let name = mir_types::Name::from(name.trim_start_matches('$'));
            if !self.0.contains(&name) {
                self.0.push(name);
            }
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// An arrow function's summary, from its body evaluated with labelled parameters.
pub fn arrow_summary(
    af: &php_ast::owned::ArrowFunctionExpr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> crate::flow_state::ClosureSummary {
    let params: Vec<(mir_types::Name, bool)> = af
        .params
        .iter()
        .map(|p| {
            (
                mir_types::Name::new(p.name.as_str().unwrap_or("")),
                p.variadic,
            )
        })
        .collect();
    let mut body_ctx = ctx.clone();
    for (name, _) in &params {
        body_ctx.set_var(name.as_str(), mir_types::Type::mixed());
    }
    label_params(&mut body_ctx, params.iter().map(|(n, _)| n.as_str()));
    for (name, taint) in std::sync::Arc::make_mut(&mut body_ctx.tainted_vars).iter_mut() {
        if !params.iter().any(|(p, _)| p == name) {
            *taint = taint.concrete();
        }
    }
    let mut reads = VariableReads(Vec::new());
    let _ = php_ast::owned::visitor::OwnedVisitor::visit_expr(&mut reads, &af.body);
    let mut captures = Vec::new();
    for name in reads.0 {
        if params.iter().any(|(p, _)| *p == name) {
            continue;
        }
        let taint = captured_taint(ctx.var_taint(name.as_str()), params.len(), &mut captures);
        if !taint.is_empty() {
            std::sync::Arc::make_mut(&mut body_ctx.tainted_vars).insert(name, taint);
        }
    }
    crate::flow_state::ClosureSummary {
        params,
        summary: TaintSummary {
            returns: expr_taint(&af.body, &body_ctx, db, file),
            ..TaintSummary::default()
        },
        captures,
    }
}

/// Expressions passed for a closure's parameter `index`.
pub(super) fn closure_param_args<'e>(
    params: &[(mir_types::Name, bool)],
    args: &'e [php_ast::owned::Arg],
    index: usize,
) -> Vec<&'e Expr> {
    let Some((name, variadic)) = params.get(index) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut position = 0;
    for arg in args {
        let Some(value) = &arg.value else {
            position += 1;
            continue;
        };
        match &arg.name {
            Some(arg_name) => {
                if crate::parser::name_to_string_owned(arg_name) == name.as_str() {
                    out.push(value);
                }
            }
            None => {
                if arg.unpack || position == index || (*variadic && position >= index) {
                    out.push(value);
                }
                position += 1;
            }
        }
    }
    out
}

pub(super) fn closure_call_taint(
    closure: &crate::flow_state::ClosureSummary,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    closure.summary.call_result(|slot| {
        closure.capture(slot).unwrap_or_else(|| {
            closure_param_args(&closure.params, args, slot)
                .into_iter()
                .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file))
        })
    })
}

/// A callback named by a string (`'trim'`) or a first-class callable (`trim(...)`).
pub(super) fn callback_function_name(callback: &Expr) -> Option<&str> {
    match &unwrap_parens(callback).kind {
        ExprKind::String(name) if !name.contains("::") => Some(name.as_ref()),
        ExprKind::CallableCreate(c) => match &c.kind {
            php_ast::owned::CallableCreateKind::Function(callee) => match &callee.kind {
                ExprKind::Identifier(name) => Some(name.as_ref()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Taint of what calling `callback` returns when it receives `inputs`.
pub(super) fn callback_result(
    callback: &Expr,
    inputs: &[Taint],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let all = inputs.iter().fold(Taint::NONE, |acc, t| acc | *t);
    let input = |index: usize| inputs.get(index).copied().unwrap_or_default();
    if let Some(closure) = callee_closure(callback, ctx, db, file) {
        return closure
            .summary
            .call_result(|slot| closure.capture(slot).unwrap_or_else(|| input(slot)));
    }
    let Some(name) = callback_function_name(callback) else {
        return all;
    };
    let lower = crate::util::php_ident_lowercase(name.trim_start_matches('\\'));
    if let Some(removed) = sanitized_kinds(&lower) {
        return all.without(removed);
    }
    if let Some(restored) = unescaped_kinds(&lower) {
        return all.restore(restored);
    }
    let resolved = crate::db::resolve_name(db, file, name);
    let Some(func) =
        crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))
    else {
        return all;
    };
    if func
        .return_type
        .as_deref()
        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty))
    {
        return Taint::NONE;
    }
    match CalleeBody::of(func.location.as_ref(), file) {
        CalleeBody::Summarized if !demanding_taint(all, ctx).is_empty() => {
            crate::db::function_taint_summary(db, file, func.fqn.as_ref())
                .map_or(Taint::NONE, |s| s.call_result(input))
        }
        CalleeBody::Summarized => Taint::NONE,
        _ => all,
    }
}

/// Builtins that call a callback on an array's elements: the callback's
/// argument index and, for each of its parameters, the argument it receives.
pub(super) fn callback_inputs(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
) -> Option<(usize, Vec<usize>)> {
    Some(match fn_name_lower {
        "array_map" => (0, (1..args.len().max(2)).collect()),
        "array_filter" | "array_walk" | "array_walk_recursive" => (1, vec![0]),
        "usort" | "uasort" | "uksort" => (1, vec![0, 0]),
        "array_reduce" => (1, vec![2, 0]),
        "preg_replace_callback" => (1, vec![2]),
        _ => return None,
    })
}

/// Result taint of a builtin that calls a callback on its input.
pub(super) fn callback_builtin_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let arg_taint = |index: usize| {
        args.get(index)
            .and_then(|a| a.value.as_ref())
            .map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file))
    };
    let (callback_index, input_args) = callback_inputs(fn_name_lower, args)?;
    let inputs: Vec<Taint> = input_args.iter().map(|i| arg_taint(*i)).collect();
    let callback = args.get(callback_index).and_then(|a| a.value.as_ref());
    let called = || {
        callback.map_or(Taint::NONE, |cb| {
            callback_result(cb, &inputs, ctx, db, file)
        })
    };
    Some(match fn_name_lower {
        "array_map" => called(),
        "array_filter" => arg_taint(0),
        "array_reduce" => arg_taint(2) | called(),
        "preg_replace_callback" => arg_taint(2) | called(),
        _ => return None,
    })
}

/// Report data a callback builtin hands to a callback whose body reaches a sink.
pub fn check_callback_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let Some((callback_index, input_args)) = callback_inputs(fn_name_lower, args) else {
        return;
    };
    let Some(callback) = args.get(callback_index).and_then(|a| a.value.as_ref()) else {
        return;
    };
    let closure = callee_closure(callback, ctx, ea.db, &ea.file);
    let param_sinks = match &closure {
        Some(closure) => closure.summary.param_sinks.clone(),
        None => {
            let Some(name) = callback_function_name(callback) else {
                return;
            };
            let resolved = crate::db::resolve_name(ea.db, &ea.file, name);
            crate::db::function_param_sinks(ea.db, &ea.file, &resolved)
                .map(|s| {
                    s.param_sinks
                        .iter()
                        .flat_map(|sink| {
                            s.slot_params(sink.param)
                                .into_iter()
                                .map(|param| ParamSink {
                                    param,
                                    ..sink.clone()
                                })
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
    };
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &param_sinks {
        if let Some(captured) = closure.as_ref().and_then(|c| c.capture(sink.param)) {
            ea.taint_summary
                .record_param_sinks(captured, sink.kinds, &sink.issue);
            continue;
        }
        let Some(value) = input_args
            .get(sink.param)
            .and_then(|i| args.get(*i))
            .and_then(|a| a.value.as_ref())
        else {
            continue;
        };
        let taint = expr_taint(value, ctx, ea.db, &ea.file);
        ea.taint_summary
            .record_param_sinks(taint, sink.kinds, &sink.issue);
        if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
            emitted.push(sink.issue.clone());
            ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
        }
    }
}

/// Report arguments a call of a closure passes to a sink inside it.
pub fn check_closure_call_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    callee: &Expr,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let Some(closure) = callee_closure(callee, ctx, ea.db, &ea.file) else {
        return;
    };
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &closure.summary.param_sinks {
        // Source data a capture holds was reported inside the closure's body.
        if let Some(captured) = closure.capture(sink.param) {
            ea.taint_summary
                .record_param_sinks(captured, sink.kinds, &sink.issue);
            continue;
        }
        for value in closure_param_args(&closure.params, args, sink.param) {
            let taint = expr_taint(value, ctx, ea.db, &ea.file);
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
            }
        }
    }
}
