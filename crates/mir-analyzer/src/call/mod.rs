mod args;
mod function;
mod method;
mod static_call;

pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};

pub struct CallAnalyzer;
