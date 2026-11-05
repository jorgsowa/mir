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
