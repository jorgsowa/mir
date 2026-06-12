mod args;
mod callable;
mod function;
mod method;
mod static_call;

pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};
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

// Reusable per-thread buffer for arg_types collection. The Option lets
// reentrant calls (foo(bar(baz()))) detect they can't borrow the same buffer
// and fall back to a fresh allocation.
thread_local! {
    pub(crate) static ARG_TYPES_BUF: std::cell::RefCell<Option<Vec<mir_types::Type>>> =
        const { std::cell::RefCell::new(Some(Vec::new())) };
}
