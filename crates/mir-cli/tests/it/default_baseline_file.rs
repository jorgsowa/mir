use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const PSALM_BASELINE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<files>
  <file src="./bad.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
    </UndefinedVariable>
  </file>
</files>
"#;

fn fixture_with_psalm_baseline() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("bad.php"),
        "<?php\nfunction f(): void { echo $missing; }\n",
    )
    .expect("failed to write PHP fixture");
    std::fs::write(dir.path().join("psalm-baseline.xml"), PSALM_BASELINE)
        .expect("failed to write baseline");
    dir
}

fn run(dir: &Path, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.current_dir(dir);
    cmd.args(["--no-cache", "--no-progress", "."]);
    cmd.args(extra_args);
    cmd.output().expect("failed to run mir binary")
}

#[test]
fn set_baseline_without_file_replaces_psalm_baseline() {
    let dir = fixture_with_psalm_baseline();

    let out = run(dir.path(), &["--set-baseline"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(!dir.path().join("psalm-baseline.xml").exists());
    let written = std::fs::read_to_string(dir.path().join("mir-baseline.xml")).unwrap();
    assert!(written.contains("<UndefinedVariable>"), "{written}");

    let out = run(dir.path(), &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn set_baseline_with_explicit_file_keeps_psalm_baseline() {
    let dir = fixture_with_psalm_baseline();

    let out = run(dir.path(), &["--set-baseline", "custom.xml"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(dir.path().join("custom.xml").exists());
    assert!(dir.path().join("psalm-baseline.xml").exists());
}

#[test]
fn mir_baseline_takes_precedence_over_psalm_baseline() {
    let dir = fixture_with_psalm_baseline();
    std::fs::write(
        dir.path().join("mir-baseline.xml"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files/>\n",
    )
    .unwrap();

    let out = run(dir.path(), &[]);

    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("mir-baseline.xml"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
