use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const BAD: &str = "<?php\nfunction f(): void { echo $missing; }\n";

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/a.php"), BAD).unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir)
        .args(["--no-cache", "--no-progress"])
        .args(args)
        .output()
        .expect("failed to run mir binary")
}

#[test]
fn set_baseline_refuses_a_php_file() {
    let dir = fixture();
    let out = run(dir.path(), &["--set-baseline", "src/a.php"]);
    assert_eq!(out.status.code(), Some(2));
    let source = std::fs::read_to_string(dir.path().join("src/a.php")).unwrap();
    assert_eq!(source, BAD);
}

#[test]
fn set_baseline_refuses_a_directory() {
    let dir = fixture();
    let out = run(dir.path(), &["--set-baseline", "src"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn a_failed_baseline_write_exits_non_zero() {
    let dir = fixture();
    let out = run(dir.path(), &["--set-baseline", "missing/b.xml", "src"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn update_baseline_without_a_baseline_writes_nothing() {
    let dir = fixture();
    let psalm = dir.path().join("psalm-baseline.xml");
    run(dir.path(), &["--set-baseline", "psalm-baseline.xml", "src"]);
    let before = std::fs::read_to_string(&psalm).unwrap();

    for args in [
        &["--baseline", "typo.xml", "--update-baseline", "src"][..],
        &["--ignore-baseline", "--update-baseline", "src"][..],
    ] {
        let out = run(dir.path(), args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }

    assert!(!dir.path().join("mir-baseline.xml").exists());
    assert!(!dir.path().join("typo.xml").exists());
    assert_eq!(std::fs::read_to_string(&psalm).unwrap(), before);
}
