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

#[test]
fn returns_type_env_for_function_scope() {
    let src = "<?php\nfunction myFn(): void {\n    $greeting = 'hello';\n}\n";
    let result = ProjectAnalyzer::analyze_source(src);
    let scope = result.type_envs.iter().find(|(k, _)| {
        matches!(k, mir_analyzer::ScopeId::Function { name, .. } if name.as_ref() == "myFn")
    });
    assert!(scope.is_some(), "Expected a TypeEnv for function myFn");
    let env = scope.unwrap().1;
    assert!(env.get_var("greeting").is_some(), "Expected $greeting in TypeEnv");
}

#[test]
fn returns_type_env_for_method_scope() {
    let src = "<?php\nclass MyClass {\n    public function handle(): void {\n        $result = 42;\n    }\n}\n";
    let result = ProjectAnalyzer::analyze_source(src);
    let scope = result.type_envs.iter().find(|(k, _)| {
        matches!(k, mir_analyzer::ScopeId::Method { method, .. } if method.as_ref() == "handle")
    });
    assert!(scope.is_some(), "Expected a TypeEnv for method handle");
    let env = scope.unwrap().1;
    assert!(env.get_var("result").is_some(), "Expected $result in TypeEnv");
}

#[test]
fn get_var_returns_none_for_unknown_variable() {
    let src = "<?php\nfunction f(): void {\n    $x = 1;\n}\n";
    let result = ProjectAnalyzer::analyze_source(src);
    let env = result.type_envs.values().next().unwrap();
    assert!(env.get_var("nonexistent").is_none());
}

#[test]
fn var_names_lists_all_variables_in_scope() {
    let src = "<?php\nfunction f(): void {\n    $a = 1;\n    $b = 'hello';\n}\n";
    let result = ProjectAnalyzer::analyze_source(src);
    let env = result.type_envs.values().next().unwrap();
    let names: Vec<&str> = env.var_names().collect();
    assert!(names.contains(&"a"), "Expected 'a' in var_names");
    assert!(names.contains(&"b"), "Expected 'b' in var_names");
}
