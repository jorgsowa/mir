use super::*;
use crate::ast_ext::ArrayElementExt as _;
use crate::ast_ext::NameRefExt as _;

/// Taint of the key names of the array `expr` yields when iterated, given the `whole` taint of its
/// contents: arrays whose keys are tracked exactly don't pass their values' taint on to their keys.
pub fn iteration_key_taint(
    expr: &Expr,
    whole: Taint,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let tracked_variable =
        matches!(&unwrap_parens(expr).kind, ExprKind::Variable(var) if ctx.has_exact_keys(var));
    if whole.is_empty() && !tracked_variable {
        return key_names_taint(expr, ctx);
    }
    exact_key_names(expr, ctx, db, file).unwrap_or_else(|| whole | key_names_taint(expr, ctx))
}

/// Taint of the key names of the array `expr` evaluates to, when it is known exactly.
pub fn exact_key_names(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let expr = unwrap_parens(expr);
    match &expr.kind {
        ExprKind::Variable(var) => ctx
            .has_exact_keys(var)
            .then(|| ctx.key_names_taint(var.trim_start_matches('$'))),
        ExprKind::Array(elements) => {
            let mut keys = Taint::NONE;
            for element in elements {
                if element.unpack() {
                    return None;
                }
                if let Some(key) = &element.key {
                    keys |= expr_taint(key, ctx, db, file);
                }
            }
            Some(keys)
        }
        ExprKind::FunctionCall(_)
        | ExprKind::MethodCall(_)
        | ExprKind::NullsafeMethodCall(_)
        | ExprKind::StaticMethodCall(_) => call_keys_taint(expr, ctx, db, file),
        _ => None,
    }
}

/// Key names the array a call to user code returns carries, from the callee's summary.
fn call_keys_taint(
    call: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    if !db.taint_analysis() {
        return None;
    }
    match &call.kind {
        ExprKind::FunctionCall(fc) => {
            let name = fc.name.kind.name_ref()?;
            if callee_closure(&fc.name, ctx, db, file).is_some() {
                return None;
            }
            let mut resolved = resolve_function_name(db, file, name.as_ref());
            let mut func =
                crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved));
            if func.is_none() && !name.contains('\\') && resolved.contains('\\') {
                resolved = name.to_string();
                func = crate::db::find_function_from(
                    db,
                    file,
                    crate::db::Fqcn::from_str(db, &resolved),
                );
            }
            let func = func.filter(|f| {
                !f.is_taint_source
                    && f.taint.is_none()
                    && CalleeBody::of(f.location.as_ref(), file) == CalleeBody::Summarized
            })?;
            if expr_taint(call, ctx, db, file).is_empty() {
                return Some(Taint::NONE);
            }
            summary_keys(crate::db::function_taint_summary(
                db,
                file,
                func.fqn.as_ref(),
            )?)
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            let ExprKind::Identifier(method_name) = &mc.method.kind else {
                return None;
            };
            // A tainted object's methods return its data.
            if !expr_taint(&mc.object, ctx, db, file).is_empty() {
                return None;
            }
            let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
            let recv_ty =
                crate::expr::assignment::resolve_chained_receiver_type(&mc.object, ctx, db, file)?;
            let mut keys = None;
            for atom in &recv_ty.types {
                let mir_types::Atomic::TNamedObject { fqcn, .. } = atom else {
                    continue;
                };
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                let found = method_keys(
                    here,
                    &method_lower,
                    &mc.args,
                    Some(&mc.object),
                    false,
                    ctx,
                    db,
                    file,
                )?;
                keys = Some(keys.unwrap_or(Taint::NONE) | found);
            }
            keys
        }
        ExprKind::StaticMethodCall(smc) => {
            let (Some(class), ExprKind::Identifier(method_name)) =
                (smc.class.kind.name_ref(), &smc.method.kind)
            else {
                return None;
            };
            let resolved = crate::db::resolve_name(db, file, class.as_ref());
            let fqcn = match resolved.as_str() {
                "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())?,
                "parent" => ctx.parent_fqcn.clone()?,
                s => std::sync::Arc::from(s),
            };
            let forwards_this = ["self", "static", "parent"]
                .iter()
                .any(|k| class.eq_ignore_ascii_case(k));
            let this_receiver = forwards_this.then(|| Expr {
                kind: ExprKind::Variable("this".into()),
                span: smc.class.span,
            });
            method_keys(
                crate::db::Fqcn::from_str(db, fqcn.as_ref()),
                &crate::util::php_ident_lowercase(method_name.as_ref()),
                &smc.args,
                this_receiver.as_ref(),
                false,
                ctx,
                db,
                file,
            )
        }
        _ => None,
    }
}

fn summary_keys(summary: TaintSummary) -> Option<Taint> {
    match summary.return_keys {
        ReturnKeys::Exact(keys) => Some(keys),
        _ => None,
    }
}

/// Key names `class::method` returns, over every implementation an abstract declaration has.
#[allow(clippy::too_many_arguments)]
fn method_keys(
    class: crate::db::Fqcn<'_>,
    method_lower: &str,
    args: &[php_ast::owned::Arg],
    receiver: Option<&Expr>,
    dispatched: bool,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let (owner, m) = crate::db::find_method_respecting_precedence(db, class, method_lower)?;
    if m.is_taint_source || m.taint.is_some() || !is_user_code(m.location.as_ref()) {
        return None;
    }
    let implementations =
        crate::db::taint_sources::method_implementations(db, class.name(db).as_str(), method_lower);
    if !implementations.is_empty() {
        return implementations
            .iter()
            .try_fold(Taint::NONE, |acc, implementor| {
                let here = crate::db::Fqcn::from_str(db, implementor.as_str());
                Some(acc | method_keys(here, method_lower, args, receiver, true, ctx, db, file)?)
            });
    }
    let whole = method_call_taint(
        class,
        method_lower,
        args,
        receiver,
        dispatched,
        ctx,
        db,
        file,
    )
    .unwrap_or_default();
    if whole.is_empty() {
        return Some(Taint::NONE);
    }
    let summarized = CalleeBody::of(m.location.as_ref(), file) == CalleeBody::Summarized
        && !m.is_abstract
        && !crate::db::is_interface(db, owner.as_ref());
    if !summarized {
        return None;
    }
    summary_keys(crate::db::method_taint_summary(
        db,
        file,
        owner.as_ref(),
        method_lower,
    )?)
}
