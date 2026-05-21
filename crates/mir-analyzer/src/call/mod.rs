// TODO(owned-migration): These modules are temporarily unreachable while
// call/method.rs, call/function.rs and call/static_call.rs are being migrated
// to owned PHP AST types. Re-enable the allow(dead_code) gates once migration
// is complete.
#[allow(dead_code)]
mod args;
#[allow(dead_code)]
mod callable;
#[allow(dead_code)]
mod function;
#[allow(dead_code)]
mod method;
#[allow(dead_code)]
mod static_call;

pub use args::{check_constructor_args, spread_element_type, CheckArgsParams};
pub(crate) use function::extract_class_docblock;

pub struct CallAnalyzer;
