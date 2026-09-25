//! End-to-end coverage for Sector C7 of the real-world compatibility audit
//! (ROADMAP.md): a `require_once`/`include` target reaching outside every
//! composer autoload root must still be indexed when the whole project is
//! analyzed with no explicit CLI path — this is exactly the path
//! `Psr4Map::project_files()` feeds (`analyze.rs::run_composer_flow`), so it
//! must be exercised via the real binary rather than a `.phpt` fixture: the
//! fixture harness (`test_utils.rs`) auto-adds any non-autoload-root file it
//! creates as an explicit path, which would mask this exact bug.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn run(dir: &Path) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.args(["--no-cache", "--no-progress", "--stats"]);
    cmd.arg(dir);
    cmd.output().expect("failed to run mir binary")
}

fn stdout_and_stderr(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// `src/` is the only composer autoload root; `legacy/` sits outside it and
/// is only reachable via a manual `require_once __DIR__ . '/../legacy/...'`
/// from a file inside `src/`.
fn fixture_with_dir_relative_require() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("legacy")).unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src/bootstrap.php"),
        "<?php require_once __DIR__ . '/../legacy/helpers.php';\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("legacy/helpers.php"),
        "<?php function legacy_helper(): string { return 'x'; }\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src/Main.php"),
        "<?php class Main { public function run(): string { return legacy_helper(); } }\n",
    )
    .unwrap();
    dir
}

#[test]
fn require_once_target_outside_autoload_root_is_indexed() {
    let dir = fixture_with_dir_relative_require();
    let out = run(dir.path());
    let combined = stdout_and_stderr(&out);
    assert!(
        !combined.contains("UndefinedFunction"),
        "legacy_helper() reached only via require_once outside every autoload root \
         should still be indexed and callable, got:\n{combined}"
    );
}
