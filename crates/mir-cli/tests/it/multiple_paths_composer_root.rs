//! Several explicit CLI paths must share one composer root (PSR-4 resolution)
//! and every path must still be analyzed.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn run(dir: &Path, paths: &[&str]) -> String {
    let out: Output = Command::new(env!("CARGO_BIN_EXE_mir"))
        .current_dir(dir)
        .env_clear()
        .args(["--no-cache", "--no-progress"])
        .args(paths)
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

/// `App\Models\Widget` and `App\Services\Maker` live in sibling dirs; each
/// references the other and `App\Support\Helper`, which sits outside both paths.
fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    write(
        p,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    write(
        p,
        "src/Models/Widget.php",
        "<?php namespace App\\Models;\nclass Widget { public function size(): int { return 1; } }\n",
    );
    write(
        p,
        "src/Support/Helper.php",
        "<?php namespace App\\Support;\nclass Helper { public static function id(): int { return 1; } }\n",
    );
    write(
        p,
        "src/Services/Maker.php",
        "<?php namespace App\\Services;\n\
         use App\\Models\\Widget;\nuse App\\Support\\Helper;\n\
         class Maker { public function make(): Widget { Helper::id(); return new Widget(); } }\n",
    );
    dir
}

#[test]
fn two_directories_resolve_classes_outside_both() {
    let dir = fixture();
    let out = run(dir.path(), &["src/Models", "src/Services"]);
    assert!(!out.contains("UndefinedClass"), "{out}");
}

#[test]
fn file_and_directory_resolve_classes_outside_both() {
    let dir = fixture();
    let out = run(dir.path(), &["src/Models/Widget.php", "src/Services"]);
    assert!(!out.contains("UndefinedClass"), "{out}");
}

#[test]
fn every_path_is_analyzed() {
    let dir = fixture();
    write(
        dir.path(),
        "src/Models/Broken.php",
        "<?php namespace App\\Models;\nclass Broken { public function f(): int { return 'a'; } }\n",
    );
    write(
        dir.path(),
        "src/Services/AlsoBroken.php",
        "<?php namespace App\\Services;\nclass AlsoBroken { public function f(): int { return 'b'; } }\n",
    );
    let out = run(dir.path(), &["src/Models", "src/Services"]);
    assert!(out.contains("Broken.php"), "{out}");
    assert!(out.contains("AlsoBroken.php"), "{out}");
    assert!(
        !out.contains("Helper.php"),
        "unlisted path analyzed:\n{out}"
    );
}

#[test]
fn overlapping_paths_report_each_issue_once() {
    let dir = fixture();
    write(
        dir.path(),
        "src/Models/Broken.php",
        "<?php namespace App\\Models;\nclass Broken { public function f(): int { return 'a'; } }\n",
    );
    let out = run(dir.path(), &["src/Models", "src/Models/Broken.php"]);
    assert_eq!(out.matches("InvalidReturnType").count(), 1, "{out}");
}

#[test]
fn project_root_among_paths_analyzes_whole_project() {
    let dir = fixture();
    write(
        dir.path(),
        "src/Support/Broken.php",
        "<?php namespace App\\Support;\nclass Broken { public function f(): int { return 'a'; } }\n",
    );
    let out = run(dir.path(), &[".", "src/Models"]);
    assert!(!out.contains("UndefinedClass"), "{out}");
    assert!(
        out.replace('\\', "/").contains("Support/Broken.php"),
        "{out}"
    );
}

#[test]
fn paths_without_composer_json_use_plain_flow() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "a/One.php", "<?php class One {}\n");
    write(
        dir.path(),
        "b/Two.php",
        "<?php function f(): One { return new One(); }\n",
    );
    let out = run(dir.path(), &["a", "b"]);
    assert!(!out.contains("UndefinedClass"), "{out}");
}
