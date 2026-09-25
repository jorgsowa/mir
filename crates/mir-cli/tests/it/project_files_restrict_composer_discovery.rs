//! End-to-end coverage for Sector H3 of the real-world compatibility audit
//! (ROADMAP.md): `<projectFiles>` was parsed into `Config::project_dirs` but
//! never consulted by the composer flow — a whole-project run always
//! analyzed every `Psr4Map::project_files()` entry regardless of which
//! directories `<projectFiles>` actually named. `<projectFiles>` should
//! intersect with (narrow down) the composer autoload roots, not be ignored.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn run(dir: &Path) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    // `mir.xml` auto-discovery walks up from the process's cwd, not from the
    // analyzed path — must run with cwd inside the fixture for `Config::find`
    // to pick up the fixture's own `mir.xml`.
    cmd.current_dir(dir);
    cmd.args(["--no-cache", "--no-progress"]);
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

/// Two autoload roots (`src/`, `legacy/`), but `<projectFiles>` only names `src/`.
fn fixture_with_narrower_project_files() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::create_dir_all(dir.path().join("legacy")).unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/","Legacy\\":"legacy/"}}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("mir.xml"),
        r#"<mir>
    <projectFiles>
        <directory name="src"/>
    </projectFiles>
</mir>"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("src/Good.php"), "<?php class Good {}\n").unwrap();
    std::fs::write(
        dir.path().join("legacy/Broken.php"),
        "<?php namespace Legacy; class Broken { public function f(): int { return 'not an int'; } }\n",
    )
    .unwrap();
    dir
}

#[test]
fn project_files_directory_excludes_other_autoload_roots() {
    let dir = fixture_with_narrower_project_files();
    let out = run(dir.path());
    let combined = stdout_and_stderr(&out);
    assert!(
        !combined.contains("InvalidReturnType"),
        "legacy/Broken.php is a real autoload root but outside the configured \
         <projectFiles> directory (src/ only) and must not be analyzed, got:\n{combined}"
    );
}
