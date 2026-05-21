mod args;
mod callable;
mod function;
mod method;
mod static_call;

pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};
pub(crate) use function::extract_class_docblock;

pub struct CallAnalyzer;
