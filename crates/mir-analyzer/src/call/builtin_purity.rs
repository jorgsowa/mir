//! Callback-taking builtins, pure only when their callback is. A stub `@pure`
//! can't express that condition; unconditionally pure builtins are tagged in
//! the stubs instead.

use mir_types::{Atomic, Type};
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
        ExprKind::Variable(name) => {
            let ty = ctx.get_var(name.as_ref().trim_start_matches('$'));
            !ty.types.is_empty()
                && ty.types.iter().all(|a| match a {
                    Atomic::TCallable { is_pure, .. } => *is_pure,
                    Atomic::TClosure { data } => data.is_pure,
                    _ => false,
                })
        }
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

/// Whether invoking a value of type `ty` may have side effects, judged only from
/// atoms that carry purity; objects and unknown types are left to other checks.
pub(crate) fn invoked_type_is_impure(ea: &ExpressionAnalyzer<'_>, ty: &Type) -> bool {
    ty.types.iter().any(|a| match a {
        Atomic::TCallable { is_pure, .. } => !is_pure,
        Atomic::TClosure { data } => !data.is_pure,
        Atomic::TCallableString => true,
        Atomic::TLiteralString(name) => {
            !name.contains("::") && !function_is_pure(ea, name.trim_start_matches('\\'), None)
        }
        _ => false,
    })
}

/// Purity of a builtin that depends on its argument types rather than its stub.
pub(crate) fn arg_dependent_builtin_purity(fqn: &str, arg_types: &[Type]) -> Option<bool> {
    let name = fqn.strip_prefix('\\').unwrap_or(fqn);
    if !name.eq_ignore_ascii_case("is_callable") {
        return None;
    }
    // A string or array argument can name a class, which is_callable() autoloads.
    let value = arg_types.first()?;
    Some(
        !value.types.is_empty()
            && value.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TNamedObject { .. }
                        | Atomic::TObject
                        | Atomic::TStaticObject { .. }
                        | Atomic::TSelf { .. }
                        | Atomic::TParent { .. }
                        | Atomic::TLiteralEnumCase { .. }
                        | Atomic::TClosure { .. }
                        | Atomic::TCallable { .. }
                        | Atomic::TNull
                )
            }),
    )
}
