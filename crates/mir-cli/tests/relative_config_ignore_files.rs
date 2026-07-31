//! End-to-end coverage for Sector H2 of the real-world compatibility audit
//! (ROADMAP.md): passing `-c <bare-filename>` (a relative config path with no
//! directory component) must still resolve `<ignoreFiles>`/`<projectFiles>`
//! directory entries relative to the current directory. `Path::parent()` on a
//! bare relative filename returns `Some("")`, not `None`, so a naive
//! `path.parent().unwrap_or(cwd)` silently used an empty config_base instead
//! of falling back to `cwd` — every relative ignore/project directory then
//! failed to resolve, and a supposedly-ignored file was analyzed anyway.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn run_with_relative_config(dir: &Path) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.current_dir(dir);
    cmd.args(["--no-cache", "--no-progress", "-c", "mir.xml", "."]);
    cmd.output().expect("failed to run mir binary")
}

fn stdout_and_stderr(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn fixture_with_ignored_directory() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir_all(dir.path().join("src/Ignored")).unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("mir.xml"),
        r#"<mir>
    <ignoreFiles>
        <directory name="src/Ignored"/>
    </ignoreFiles>
</mir>"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src/Ignored/Broken.php"),
        "<?php class Broken { public function f(): int { return 'not an int'; } }\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/Good.php"), "<?php class Good {}\n").unwrap();
    dir
}

#[test]
fn relative_bare_config_path_still_resolves_ignore_files() {
    let dir = fixture_with_ignored_directory();
    let out = run_with_relative_config(dir.path());
    let combined = stdout_and_stderr(&out);
    assert!(
        !combined.contains("InvalidReturnType"),
        "src/Ignored/Broken.php is under an <ignoreFiles> directory and must not be \
         analyzed even when -c is passed as a bare relative filename, got:\n{combined}"
    );
}
