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
        ExprKind::PropertyAccess(pa) => {
            let ExprKind::Variable(object) = &pa.object.kind else {
                return None;
            };
            let prop = crate::expr::helpers::extract_string_from_expr(&pa.property)?;
            ctx.closures
                .by_var
                .get(&crate::flow_state::prop_closure_key(
                    object.trim_start_matches('$'),
                    &prop,
                ))
                .cloned()
        }
        _ => None,
    }
}

/// The summary of a first-class callable of a named function (`id(...)`), laid out like a closure's.
pub fn callable_summary(
    callable: &Expr,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<crate::flow_state::ClosureSummary>> {
    if !db.taint_analysis() {
        return None;
    }
    let name = callback_function_name(callable)?;
    let lower = crate::util::php_ident_lowercase(name.trim_start_matches('\\'));
    let resolved = crate::db::resolve_name(db, file, name);
    let func = crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))?;
    let tracked = func.params.len().min(MAX_TRACKED_PARAMS);
    let through =
        |kinds: Taint| (0..tracked).fold(Taint::NONE, |acc, slot| acc | Taint::label(slot, kinds));
    let mut summary = TaintSummary::default();
    let is_user_body = CalleeBody::of(func.location.as_ref(), file) == CalleeBody::Summarized;
    if is_user_body {
        if let Some(inferred) = crate::db::function_taint_summary(db, file, func.fqn.as_ref()) {
            summary = inferred;
        }
    } else if !func
        .return_type
        .as_deref()
        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty))
    {
        summary.returns = match sanitized_kinds(&lower) {
            Some(removed) => through(Taint::ALL.difference(removed)),
            None => through(Taint::ALL),
        };
    }
    if !is_user_body {
        for sink in builtin_sinks(&lower) {
            let positions = match sink.index {
                Some(index) => index..index + 1,
                None => 0..tracked,
            };
            for param in positions.filter(|param| *param < MAX_TRACKED_PARAMS) {
                summary.param_sinks.push(ParamSink {
                    param,
                    kinds: sink.kind.taint(),
                    issue: sink.kind.issue(),
                });
            }
        }
    }
    Some(std::sync::Arc::new(crate::flow_state::ClosureSummary {
        params: func
            .params
            .iter()
            .map(|p| (mir_types::Name::new(p.name.as_ref()), p.is_variadic))
            .collect(),
        summary,
        captures: Vec::new(),
    }))
}

/// The closure an assignment's value evaluates to, when it is one.
pub fn assigned_closure(
    value: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<crate::flow_state::ClosureSummary>> {
    match &value.kind {
        ExprKind::Closure(_) | ExprKind::ArrowFunction(_) => {
            ctx.closures.by_span.get(&value.span.start).cloned()
        }
        ExprKind::Variable(source) => ctx
            .closures
            .by_var
            .get(&mir_types::Name::from(source.trim_start_matches('$')))
            .cloned(),
        ExprKind::PropertyAccess(_) => callee_closure(value, ctx, db, file),
        ExprKind::CallableCreate(_) => callable_summary(value, db, file),
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
    bound_arg_exprs(args, index, name.as_str(), *variadic, |key| {
        params.iter().any(|(p, _)| p.as_str() == key)
    })
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
    let input_for = |index: usize, variadic: bool| {
        let from = if variadic { index } else { inputs.len() };
        let first = inputs.get(index).copied().unwrap_or_default();
        inputs
            .get(from..)
            .unwrap_or_default()
            .iter()
            .fold(first, |acc, t| acc | *t)
    };
    if let Some(closure) = callee_closure(callback, ctx, db, file) {
        return closure.summary.call_result(|slot| {
            closure.capture(slot).unwrap_or_else(|| {
                input_for(slot, closure.params.get(slot).is_some_and(|(_, v)| *v))
            })
        });
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
            crate::db::function_taint_summary(db, file, func.fqn.as_ref()).map_or(
                Taint::NONE,
                |s| {
                    s.call_result(|slot| {
                        input_for(slot, func.params.get(slot).is_some_and(|p| p.is_variadic))
                    })
                },
            )
        }
        CalleeBody::Summarized => Taint::NONE,
        _ => all,
    }
}

/// Names of the parameters `callback` declares, when it is a closure or a named function.
fn callback_param_names(
    callback: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Vec<String>> {
    if let Some(closure) = callee_closure(callback, ctx, db, file) {
        return Some(closure.params.iter().map(|(n, _)| n.to_string()).collect());
    }
    let resolved = crate::db::resolve_name(db, file, callback_function_name(callback)?);
    let func = crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))?;
    Some(func.params.iter().map(|p| p.name.to_string()).collect())
}

/// The arguments `call_user_func_array` binds to a callback's parameters, from an array literal.
fn array_literal_inputs<'e>(
    list: &'e Expr,
    callback: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Vec<Option<&'e Expr>>> {
    let ExprKind::Array(elements) = &unwrap_parens(list).kind else {
        return None;
    };
    let keyed = elements
        .iter()
        .any(|el| matches!(literal_element_key(el), ElementKey::Named(_)));
    let names = if keyed {
        callback_param_names(callback, ctx, db, file)?
    } else {
        Vec::new()
    };
    let mut inputs: Vec<Option<&Expr>> = vec![None; names.len()];
    let mut position = 0;
    for el in elements {
        let slot = match literal_element_key(el) {
            ElementKey::Named(key) => names.iter().position(|n| n == key)?,
            ElementKey::Positional => {
                position += 1;
                position - 1
            }
            ElementKey::Unknown => return None,
        };
        if slot >= inputs.len() {
            inputs.resize(slot + 1, None);
        }
        inputs[slot] = Some(&el.value);
    }
    Some(inputs)
}

/// Builtins that call a callback: the callback's argument index and, for each of its
/// parameters, the argument it receives.
pub(super) fn callback_inputs<'e>(
    fn_name_lower: &str,
    args: &'e [php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<(usize, Vec<Option<&'e Expr>>)> {
    let value = |index: usize| args.get(index).and_then(|a| a.value.as_ref());
    let (callback_index, input_args): (usize, Vec<usize>) = match fn_name_lower {
        "array_map" => (0, (1..args.len().max(2)).collect()),
        "array_filter" | "array_walk" | "array_walk_recursive" => (1, vec![0]),
        "usort" | "uasort" | "uksort" => (1, vec![0, 0]),
        "array_reduce" => (1, vec![2, 0]),
        "preg_replace_callback" => (1, vec![2]),
        "call_user_func" if args.iter().all(|a| !a.unpack && a.name.is_none()) => {
            (0, (1..args.len()).collect())
        }
        "call_user_func_array" if args.len() == 2 && args.iter().all(|a| a.name.is_none()) => {
            let callback = value(0)?;
            let inputs = array_literal_inputs(value(1)?, callback, ctx, db, file)?;
            return Some((0, inputs));
        }
        _ => return None,
    };
    Some((callback_index, input_args.into_iter().map(value).collect()))
}

/// Result taint of a builtin that calls a callback on its input.
pub(super) fn callback_builtin_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let (callback_index, input_args) = callback_inputs(fn_name_lower, args, ctx, db, file)?;
    let arg_taint = |index: usize| {
        args.get(index)
            .and_then(|a| a.value.as_ref())
            .map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file))
    };
    let inputs: Vec<Taint> = input_args
        .iter()
        .map(|v| v.map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file)))
        .collect();
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
        // A callback that isn't a literal name or closure is data the call result may carry.
        "call_user_func" | "call_user_func_array" => {
            let literal = callback.is_some_and(|cb| {
                callee_closure(cb, ctx, db, file).is_some() || callback_function_name(cb).is_some()
            });
            called() | if literal { Taint::NONE } else { arg_taint(0) }
        }
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
    let Some((callback_index, input_args)) =
        callback_inputs(fn_name_lower, args, ctx, ea.db, &ea.file)
    else {
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
        let Some(value) = input_args.get(sink.param).copied().flatten() else {
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
