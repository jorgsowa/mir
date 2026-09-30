//! `errorLevel` (root attribute or `--error-level`) applies Psalm's per-issue levels:
//! an issue whose level is below the configured one is downgraded to info.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const PHP: &str = "<?php function f(bool $c): void { if ($c) { $a = 1; } echo $a; }\n";

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

#[test]
fn unset_level_keeps_mir_severities() {
    let dir = fixture("<mir></mir>");
    assert!(run(dir.path(), &[]).contains("PossiblyUndefinedVariable"));
}

#[test]
fn lenient_level_downgrades_possible_issues_to_info() {
    let dir = fixture(r#"<mir errorLevel="4"></mir>"#);
    assert!(!run(dir.path(), &[]).contains("PossiblyUndefinedVariable"));
    assert!(run(dir.path(), &["--show-info"]).contains("PossiblyUndefinedVariable"));
}

#[test]
fn strict_level_keeps_possible_issues() {
    let dir = fixture(r#"<mir errorLevel="3"></mir>"#);
    assert!(run(dir.path(), &[]).contains("PossiblyUndefinedVariable"));
}

#[test]
fn cli_flag_overrides_config() {
    let dir = fixture(r#"<mir errorLevel="3"></mir>"#);
    assert!(!run(dir.path(), &["--error-level", "5"]).contains("PossiblyUndefinedVariable"));
}

#[test]
fn issue_handler_wins_over_level() {
    let dir = fixture(
        r#"<mir errorLevel="8"><issueHandlers><PossiblyUndefinedVariable errorLevel="error"/></issueHandlers></mir>"#,
    );
    assert!(run(dir.path(), &[]).contains("PossiblyUndefinedVariable"));
}
