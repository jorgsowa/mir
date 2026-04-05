// crates/mir-analyzer/tests/type_env.rs
use mir_analyzer::{ProjectAnalyzer, ScopeId};
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

#[test]
fn analyze_result_has_type_envs_field() {
    let result = ProjectAnalyzer::analyze_source("<?php\n");
    // just verifying the field exists and is empty for a trivial source
    assert!(result.type_envs.is_empty());
}
