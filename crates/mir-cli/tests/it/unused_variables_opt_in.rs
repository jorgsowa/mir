//! Unused-variable kinds are reported only under `findUnusedVariablesAndParams`,
//! `findUnusedVariables`, `findUnusedCode`, `errorLevel="1"` or `--find-dead-code`, as in Psalm.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const PHP: &str = "<?php
function f(array $items): int {
    $unused = count($items);
    foreach ($items as $k => $v) {
        echo $k;
    }
    return 1;
}
";

fn fixture(config: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("mir.xml"), config).unwrap();
    std::fs::write(dir.path().join("a.php"), PHP).unwrap();
    dir
}

fn run(dir: &Path, extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir)
        .args(["--no-cache", "--no-progress", "-c", "mir.xml"])
        .args(extra)
        .arg(".")
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn reports_both(output: &str) -> bool {
    output.contains("UnusedVariable") && output.contains("UnusedForeachValue")
}

fn reports_neither(output: &str) -> bool {
    !output.contains("UnusedVariable") && !output.contains("UnusedForeachValue")
}

#[test]
fn hidden_by_default() {
    let dir = fixture("<mir></mir>");
    assert!(reports_neither(&run(dir.path(), &[])));
}

#[test]
fn hidden_when_unused_code_attribute_is_false() {
    let dir = fixture(r#"<psalm findUnusedCode="false"></psalm>"#);
    assert!(reports_neither(&run(dir.path(), &[])));
}

#[test]
fn find_unused_variables_attribute_enables() {
    let dir = fixture(r#"<psalm findUnusedVariables="true"></psalm>"#);
    assert!(reports_both(&run(dir.path(), &[])));
}

#[test]
fn find_unused_variables_element_enables() {
    let dir = fixture("<mir><findUnusedVariables>true</findUnusedVariables></mir>");
    assert!(reports_both(&run(dir.path(), &[])));
}

#[test]
fn find_unused_code_attribute_enables() {
    let dir = fixture(r#"<psalm findUnusedCode="true"></psalm>"#);
    assert!(reports_both(&run(dir.path(), &[])));
}

#[test]
fn find_dead_code_flag_enables() {
    let dir = fixture("<mir></mir>");
    assert!(reports_both(&run(dir.path(), &["--find-dead-code"])));
}

#[test]
fn psalm_find_unused_variables_and_params_attribute_enables() {
    let dir = fixture(r#"<psalm findUnusedVariablesAndParams="true"></psalm>"#);
    assert!(reports_both(&run(dir.path(), &[])));
}

#[test]
fn error_level_one_enables() {
    let dir = fixture(r#"<psalm errorLevel="1"></psalm>"#);
    assert!(reports_both(&run(dir.path(), &[])));
}

#[test]
fn error_level_one_respects_explicit_opt_out() {
    let dir = fixture(r#"<psalm errorLevel="1" findUnusedVariablesAndParams="false"></psalm>"#);
    assert!(reports_neither(&run(dir.path(), &[])));
}
