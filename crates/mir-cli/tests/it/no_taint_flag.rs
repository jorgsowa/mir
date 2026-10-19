//! `--no-taint` skips taint analysis, so no `Tainted*` issue is reported.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const PHP: &str = "<?php
function show(): void {
    echo $_GET['name'];
}
";

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

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("mir.xml"), "<mir></mir>").unwrap();
    std::fs::write(dir.path().join("a.php"), PHP).unwrap();
    dir
}

#[test]
fn taint_is_reported_by_default() {
    let dir = fixture();
    assert!(run(dir.path(), &[]).contains("TaintedHtml"));
}

#[test]
fn no_taint_flag_suppresses_taint_issues() {
    let dir = fixture();
    assert!(!run(dir.path(), &["--no-taint"]).contains("Tainted"));
}
