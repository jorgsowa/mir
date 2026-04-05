// crates/mir-analyzer/tests/type_env.rs
use mir_analyzer::ScopeId;
use std::sync::Arc;

#[test]
fn scope_id_function_equality() {
    let a = ScopeId::Function {
        file: Arc::from("foo.php"),
        name: Arc::from("myFn"),
    };
    let b = ScopeId::Function {
        file: Arc::from("foo.php"),
        name: Arc::from("myFn"),
    };
    assert_eq!(a, b);
}
