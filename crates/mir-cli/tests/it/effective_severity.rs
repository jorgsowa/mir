use std::process::{Command, Output};

use tempfile::TempDir;

fn run(php: &str, handlers: &str, format: &str) -> Output {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/a.php"), php).unwrap();
    std::fs::write(
        dir.path().join("mir.xml"),
        format!(
            r#"<?xml version="1.0"?>
<psalm>
  <projectFiles><directory name="src"/></projectFiles>
  <issueHandlers>{handlers}</issueHandlers>
</psalm>
"#
        ),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir.path())
        .args(["--no-cache", "--no-progress", "-c", "mir.xml"])
        .args(["--format", format, "src"])
        .output()
        .expect("failed to run mir binary")
}

const EVAL: &str = "<?php\neval($_GET['x']);\n";
const POSSIBLY_UNDEFINED: &str =
    "<?php\nfunction f(bool $c): void {\n  if ($c) { $a = 1; }\n  echo $a;\n}\n";

#[test]
fn a_lowered_error_prints_as_a_warning_and_passes() {
    let out = run(EVAL, r#"<TaintedEval errorLevel="warning"/>"#, "text");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("warning[MIR0813]"), "{stdout}");
    assert_eq!(out.status.code(), Some(0), "{stdout}");
}

#[test]
fn json_reports_the_lowered_severity() {
    let out = run(EVAL, r#"<TaintedEval errorLevel="warning"/>"#, "json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let severity = issues[0]["severity"].as_str().unwrap();
    assert!(severity.eq_ignore_ascii_case("warning"), "{stdout}");
}

#[test]
fn a_raised_warning_prints_as_an_error_and_fails() {
    let out = run(
        POSSIBLY_UNDEFINED,
        r#"<PossiblyUndefinedVariable errorLevel="error"/>"#,
        "text",
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("error[MIR0008]"), "{stdout}");
    assert_eq!(out.status.code(), Some(1), "{stdout}");
}
