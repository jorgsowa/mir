//! A composer.json without an `autoload` section falls back to analyzing the given paths directly.

use std::process::Command;

use tempfile::TempDir;

#[test]
fn composer_json_without_autoload_analyzes_paths() {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("composer.json"), r#"{"name":"x/y"}"#).unwrap();
    std::fs::write(
        dir.path().join("a.php"),
        "<?php\nfunction f(): int { return undefined_fn(); }\n",
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir.path())
        .args(["--no-cache", "--no-progress"])
        .output()
        .expect("failed to run mir binary");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert_ne!(out.status.code(), Some(2), "{all}");
    assert!(all.contains("UndefinedFunction"), "{all}");
}
