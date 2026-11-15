//! Opt-in kinds are reported only once an `<issueHandlers>` entry gives them a level.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const PHP: &str = "<?php
function load(string $stored): mixed {
    return unserialize($stored);
}
";

fn fixture(config: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("mir.xml"), config).unwrap();
    std::fs::write(dir.path().join("a.php"), PHP).unwrap();
    dir
}

fn run(dir: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir)
        .args(["--no-cache", "--no-progress", "-c", "mir.xml", "."])
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn hidden_by_default() {
    let dir = fixture("<mir></mir>");
    assert!(!run(dir.path()).contains("UnsafeUnserialize"));
}

#[test]
fn hidden_when_suppressed() {
    let dir = fixture(
        r#"<mir><issueHandlers><UnsafeUnserialize errorLevel="suppress"/></issueHandlers></mir>"#,
    );
    assert!(!run(dir.path()).contains("UnsafeUnserialize"));
}

#[test]
fn reported_when_given_a_level() {
    let dir = fixture(
        r#"<mir><issueHandlers><UnsafeUnserialize errorLevel="warning"/></issueHandlers></mir>"#,
    );
    assert!(run(dir.path()).contains("UnsafeUnserialize"));
}
