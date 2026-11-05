//! Psalm's path-scoped `<errorLevel type="..."><directory/></errorLevel>` blocks.

use std::process::Command;

use tempfile::TempDir;

const PHP: &str = "<?php function f(bool $c): void { if ($c) { $a = 1; } echo $a; }\n";

#[test]
fn scoped_suppress_applies_only_inside_its_directory() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(root.join("src/a.php"), PHP).unwrap();
    std::fs::write(root.join("tests/b.php"), PHP).unwrap();
    std::fs::write(
        root.join("mir.xml"),
        r#"<mir><issueHandlers><PossiblyUndefinedVariable>
            <errorLevel type="suppress"><directory name="tests"/></errorLevel>
        </PossiblyUndefinedVariable></issueHandlers></mir>"#,
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(root)
        .args([
            "--no-cache",
            "--no-progress",
            "-c",
            "mir.xml",
            "src",
            "tests",
        ])
        .output()
        .expect("failed to run mir binary");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("src/a.php") && combined.contains("PossiblyUndefinedVariable"),
        "issue outside the scope must still be reported, got:\n{combined}"
    );
    assert!(
        !combined.contains("tests/b.php"),
        "issue inside the suppressed directory must be hidden, got:\n{combined}"
    );
}

#[test]
fn referenced_filters_match_the_symbol_an_issue_names() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/a.php"),
        "<?php
new App\\Hidden();
new App\\Shown();
hidden_fn();
shown_fn();
(new stdClass())->hiddenMethod();
",
    )
    .unwrap();
    std::fs::write(
        root.join("mir.xml"),
        r#"<mir><issueHandlers>
            <UndefinedClass><errorLevel type="suppress"><referencedClass name="\app\Hid*"/></errorLevel></UndefinedClass>
            <UndefinedFunction><errorLevel type="suppress"><referencedFunction name="hidden_fn"/></errorLevel></UndefinedFunction>
            <UndefinedMethod><errorLevel type="suppress"><referencedMethod name="stdClass::hiddenMethod"/></errorLevel></UndefinedMethod>
        </issueHandlers></mir>"#,
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(root)
        .args(["--no-cache", "--no-progress", "-c", "mir.xml", "src"])
        .output()
        .expect("failed to run mir binary");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for shown in ["App\\Shown", "shown_fn"] {
        assert!(combined.contains(shown), "{shown} hidden:\n{combined}");
    }
    for hidden in ["App\\Hidden", "hidden_fn", "hiddenMethod"] {
        assert!(!combined.contains(hidden), "{hidden} shown:\n{combined}");
    }
}
