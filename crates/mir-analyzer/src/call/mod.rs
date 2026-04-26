mod args;
mod function;
mod method;
mod static_call;

pub(crate) use args::expr_can_be_passed_by_reference;
pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};

pub struct CallAnalyzer;
