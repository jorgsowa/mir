//! `<fileExtensions>` in `mir.xml` makes non-`.php` files (`.module`, `.inc`) part of every
//! discovery path: the plain walk, ignored/vendor dirs and the composer project walk.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const MODULE_AND_INC: &str = r#"<mir><fileExtensions>
    <extension name=".php"/><extension name=".module"/><extension name=".inc"/>
</fileExtensions></mir>"#;

fn run(dir: &Path) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.current_dir(dir);
    cmd.args(["--no-cache", "--no-progress"]);
    cmd.arg(dir);
    let out: Output = cmd.output().expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn write(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn plain_walk_analyzes_configured_extensions() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "mir.xml", MODULE_AND_INC);
    write(
        dir.path(),
        "a.module",
        "<?php function a(): int { return b(); }\n",
    );
    write(
        dir.path(),
        "b.inc",
        "<?php function b(): int { return 'x'; }\n",
    );

    let out = run(dir.path());

    assert!(out.contains("b.inc"), "b.inc not analyzed:\n{out}");
    assert!(!out.contains("UndefinedFunction"), "got:\n{out}");
}

#[test]
fn plain_walk_ignores_non_php_without_config() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "a.module",
        "<?php function a(): int { return 'x'; }\n",
    );

    let out = run(dir.path());

    assert!(out.contains("No PHP files found"), "got:\n{out}");
}

#[test]
fn ignored_dirs_are_indexed_with_configured_extensions() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "mir.xml",
        r#"<mir><fileExtensions><extension name="php"/><extension name="inc"/></fileExtensions>
<ignoreFiles><directory name="lib"/></ignoreFiles></mir>"#,
    );
    write(
        dir.path(),
        "main.php",
        "<?php function m(): int { return lib_fn(); }\n",
    );
    write(
        dir.path(),
        "lib/helpers.inc",
        "<?php function lib_fn(): int { return 1; }\n",
    );

    let out = run(dir.path());

    assert!(!out.contains("UndefinedFunction"), "got:\n{out}");
    assert!(
        !out.contains("helpers.inc:"),
        "ignored dir was reported:\n{out}"
    );
}

#[test]
fn composer_walk_analyzes_configured_extensions() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "mir.xml", MODULE_AND_INC);
    write(
        dir.path(),
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    write(
        dir.path(),
        "src/Main.php",
        "<?php namespace App; class Main {}\n",
    );
    write(
        dir.path(),
        "src/legacy.module",
        "<?php function legacy(): int { return 'x'; }\n",
    );

    let out = run(dir.path());

    assert!(out.contains("legacy.module"), "got:\n{out}");
}
