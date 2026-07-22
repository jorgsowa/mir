mod args;
mod array_builtins;
mod callable;
mod function;
pub(crate) mod method;
mod opaque_callback;
mod static_call;

pub(crate) use args::substitute_static_in_return;
pub use args::{
    check_constructor_args, distinct_spans_for_expansion, expand_sole_spread_arg,
    spread_element_type, CheckArgsParams,
};
pub(crate) use function::extract_class_docblock;

pub struct CallAnalyzer;

/// An assignment expression in argument position (`f($x = expr)`,
/// `->andReturn($mock = m::mock(...))`) has its value consumed by the call —
/// the write is used even if the variable is never read again.
pub(crate) fn consume_arg_assignment(
    expr: &php_ast::owned::Expr,
    ctx: &mut crate::flow_state::FlowState,
) {
    if let php_ast::owned::ExprKind::Assign(a) = &expr.kind {
        if let php_ast::owned::ExprKind::Variable(name) = &a.target.kind {
            let n = name.trim_start_matches('$');
            ctx.read_vars.insert(mir_types::Name::from(n));
            ctx.mark_consumed(n);
        }
    }
}

/// Pre-mark variables passed to by-reference parameters as defined.
///
/// Passing an as-yet-undefined variable to an out-parameter (e.g. `&$matches`
/// in `preg_match`, or a user method's `&$out`) defines it, so it must not be
/// reported as `UndefinedVariable`. Variadic by-ref params (`&...$rest`) cover
/// every argument from their position onward. Must run before the arguments are
/// analyzed so the read side never sees the variable as undefined.
///
/// Binds each arg to its param the same way `call/args/counts.rs::check_counts`
/// does — honoring named arguments that reorder which param a given arg feeds
/// — rather than assuming `args[i]` always feeds `params[i]`. That fuller
/// binder can't be reused directly here since it needs each arg's inferred
/// type, which isn't available yet (premarking must run before args are
/// analyzed).
pub(crate) fn premark_byref_arg_vars(
    params: &[mir_codebase::definitions::DeclaredParam],
    args: &[php_ast::owned::Arg],
    ctx: &mut crate::flow_state::FlowState,
) {
    use php_ast::owned::ExprKind;

    fn premark_one(
        param: &mir_codebase::definitions::DeclaredParam,
        arg: &php_ast::owned::Arg,
        ctx: &mut crate::flow_state::FlowState,
    ) {
        if !param.is_byref {
            return;
        }
        if let ExprKind::Variable(name) = &arg.value.kind {
            let var_name = name.trim_start_matches('$');
            if !ctx.var_is_defined(var_name) {
                // Prefer @param-out type if declared; fall back to declared
                // in-type, then mixed.
                let ty = param
                    .out_ty
                    .as_ref()
                    .or(param.ty.as_ref())
                    .map(|t| (**t).clone())
                    .unwrap_or_else(mir_types::Type::mixed);
                ctx.set_var(var_name, ty);
            }
        }
    }

    let variadic_index = params.iter().position(|p| p.is_variadic);
    let max_positional = variadic_index.unwrap_or(params.len());
    let mut positional = 0usize;

    for arg in args {
        // A spread (`...$rest`) can't be statically bound to a single param.
        if arg.unpack {
            break;
        }
        if let Some(name) = &arg.name {
            let name = crate::parser::name_to_string_owned(name);
            if let Some(param) = params.iter().find(|p| p.name.as_ref() == name.as_str()) {
                premark_one(param, arg, ctx);
            } else if let Some(vi) = variadic_index {
                premark_one(&params[vi], arg, ctx);
            }
            continue;
        }
        let param = if positional < max_positional {
            params.get(positional)
        } else {
            variadic_index.map(|vi| &params[vi])
        };
        if let Some(param) = param {
            premark_one(param, arg, ctx);
        }
        positional += 1;
    }
}

// Reusable per-thread buffer for arg_types collection. The Option lets
// reentrant calls (foo(bar(baz()))) detect they can't borrow the same buffer
// and fall back to a fresh allocation.
thread_local! {
    pub(crate) static ARG_TYPES_BUF: std::cell::RefCell<Option<Vec<mir_types::Type>>> =
        const { std::cell::RefCell::new(Some(Vec::new())) };
}
