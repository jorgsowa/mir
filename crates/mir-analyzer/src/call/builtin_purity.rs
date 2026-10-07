//! Callback-taking builtins, pure only when their callback is. A stub `@pure`
//! can't express that condition; unconditionally pure builtins are tagged in
//! the stubs instead.

use php_ast::owned::{Arg, CallableCreateKind, Expr, ExprKind};

use mir_codebase::definitions::DeclaredParam;

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;

fn takes_callback_purity(fqn: &str) -> bool {
    let name = fqn.strip_prefix('\\').unwrap_or(fqn);
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_map"
            | "array_filter"
            | "array_reduce"
            | "array_all"
            | "array_any"
            | "array_find"
            | "array_find_key"
            | "usort"
            | "uasort"
            | "uksort"
            | "preg_replace_callback"
    )
}

/// Whether a call to the callback builtin `fqn` is pure inside a `@pure` body.
pub(crate) fn callback_builtin_call_is_pure(
    ea: &ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    fqn: &str,
    params: &[DeclaredParam],
    args: &[Arg],
) -> bool {
    if !takes_callback_purity(fqn) {
        return false;
    }
    // A spread can land on the callback position.
    if args.iter().any(|a| a.unpack) {
        return false;
    }
    let Some(idx) = params.iter().position(|p| p.name.as_ref() == "callback") else {
        return false;
    };
    match crate::call::resolve_named_arg_type_index(params, args, idx)
        .and_then(|i| args.get(i))
        .and_then(|a| a.value.as_ref())
    {
        Some(callback) => callback_is_pure(ea, ctx, callback),
        None => true,
    }
}

fn callback_is_pure(ea: &ExpressionAnalyzer<'_>, ctx: &FlowState, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Null => true,
        // Their bodies inherit the pure context and are checked where declared.
        ExprKind::Closure(_) | ExprKind::ArrowFunction(_) => ctx.is_in_pure_fn,
        // Callable strings always name a global function.
        ExprKind::String(name) => {
            !name.contains("::") && function_is_pure(ea, name.trim_start_matches('\\'), None)
        }
        ExprKind::CallableCreate(cc) => match &cc.kind {
            CallableCreateKind::Function(target) => match &target.kind {
                ExprKind::Identifier(name) => {
                    let resolved = crate::db::resolve_name(ea.db, &ea.file, name.as_ref());
                    let fallback = (!name.contains('\\')).then_some(name.as_ref());
                    function_is_pure(ea, &resolved, fallback)
                }
                _ => false,
            },
            CallableCreateKind::StaticMethod { class, method } => {
                let (ExprKind::Identifier(class), ExprKind::Identifier(method)) =
                    (&class.kind, &method.kind)
                else {
                    return false;
                };
                let resolved = crate::db::resolve_name(ea.db, &ea.file, class.as_ref());
                let fqcn = match crate::util::php_ident_lowercase(&resolved).as_str() {
                    "self" => ctx.self_fqcn.as_deref(),
                    "parent" => ctx.parent_fqcn.as_deref(),
                    "static" => ctx.static_fqcn.as_deref().or(ctx.self_fqcn.as_deref()),
                    _ => Some(resolved.as_str()),
                };
                fqcn.is_some_and(|fqcn| {
                    crate::db::method_is_pure_in_chain(
                        ea.db,
                        crate::db::Fqcn::from_str(ea.db, fqcn),
                        method.as_ref(),
                    )
                })
            }
            _ => false,
        },
        _ => false,
    }
}

fn function_is_pure(ea: &ExpressionAnalyzer<'_>, fqn: &str, fallback: Option<&str>) -> bool {
    let lookup = |n: &str| {
        crate::db::find_function_from(ea.db, &ea.file, crate::db::Fqcn::from_str(ea.db, n))
    };
    let Some(f) = lookup(fqn).or_else(|| fallback.and_then(lookup)) else {
        return false;
    };
    f.is_pure
}
