mod args;
mod callable;
mod function;
mod method;
mod static_call;

pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};
pub(crate) use function::extract_class_docblock;

pub struct CallAnalyzer;

// Reusable per-thread buffer for arg_types collection. The Option lets
// reentrant calls (foo(bar(baz()))) detect they can't borrow the same buffer
// and fall back to a fresh allocation.
thread_local! {
    pub(crate) static ARG_TYPES_BUF: std::cell::RefCell<Option<Vec<mir_types::Union>>> =
        const { std::cell::RefCell::new(Some(Vec::new())) };
}
