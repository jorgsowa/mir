use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const BAD: &str = "<?php\nfunction f(): void { echo $missing; }\n";

fn fixture(with_composer: bool) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir(dir.path().join("src")).unwrap();
    if with_composer {
        std::fs::write(
            dir.path().join("composer.json"),
            r#"{"autoload": {"classmap": ["src/"]}}"#,
        )
        .unwrap();
    }
    std::fs::write(dir.path().join("src/a.php"), BAD).unwrap();
    std::fs::write(dir.path().join("src/b.php"), BAD).unwrap();
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<files>
  <file src="src/a.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
    </UndefinedVariable>
  </file>
  <file src="src/b.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
    </UndefinedVariable>
  </file>
  <file src="src/gone.php">
    <UndefinedVariable>
      <code><![CDATA[$old]]></code>
    </UndefinedVariable>
  </file>
</files>
"#,
    )
    .unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir)
        .args(["--no-cache", "--no-progress", "--format", "json"])
        .args(args)
        .output()
        .expect("failed to run mir binary")
}

fn assert_partial_run_keeps_other_files(with_composer: bool) {
    let dir = fixture(with_composer);

    let out = run(dir.path(), &["--report-stale-baseline", "src/a.php"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("1 stale baseline issue(s)") && stderr.contains("src/gone.php"),
        "only the deleted file's entry is stale:\n{stderr}"
    );
    assert!(!stderr.contains("src/b.php"), "{stderr}");

    run(dir.path(), &["--update-baseline", "src/a.php"]);
    let updated = std::fs::read_to_string(dir.path().join("psalm-baseline.xml")).unwrap();
    assert!(updated.contains(r#"<file src="src/a.php">"#), "{updated}");
    assert!(updated.contains(r#"<file src="src/b.php">"#), "{updated}");
    assert!(!updated.contains("src/gone.php"), "{updated}");

    let out = run(dir.path(), &["--report-stale-baseline", "src/a.php"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn partial_run_keeps_baseline_entries_of_unanalyzed_files() {
    assert_partial_run_keeps_other_files(false);
}

#[test]
fn partial_composer_run_keeps_baseline_entries_of_unanalyzed_files() {
    assert_partial_run_keeps_other_files(true);
}

#[test]
fn full_run_still_reports_entries_of_clean_files_stale() {
    let dir = fixture(false);
    std::fs::write(dir.path().join("src/b.php"), "<?php\n").unwrap();

    for target in ["src", "."] {
        let out = run(dir.path(), &["--report-stale-baseline", target]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{target}: {stderr}");
        assert!(
            stderr.contains("2 stale baseline issue(s)"),
            "{target}: {stderr}"
        );
        assert!(stderr.contains("src/b.php"), "{target}: {stderr}");
    }
}

#[test]
fn dot_target_writes_baseline_keys_without_dot_prefix() {
    let dir = fixture(false);
    run(dir.path(), &["--update-baseline", "."]);
    let updated = std::fs::read_to_string(dir.path().join("psalm-baseline.xml")).unwrap();
    assert!(updated.contains(r#"<file src="src/a.php">"#), "{updated}");
    assert!(!updated.contains("./src"), "{updated}");

    let out = run(dir.path(), &["--report-stale-baseline", "."]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const WORKDIR_KEYS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<files>
  <file src="src/a.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
    </UndefinedVariable>
  </file>
  <file src="src/b.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
    </UndefinedVariable>
  </file>
</files>
"#;

#[test]
fn baseline_above_the_workdir_matches_workdir_relative_keys() {
    let root = TempDir::new().expect("failed to create temp dir");
    let proj = root.path().join("proj");
    std::fs::create_dir_all(proj.join("src")).unwrap();
    std::fs::write(proj.join("src/a.php"), BAD).unwrap();
    std::fs::write(proj.join("src/b.php"), BAD).unwrap();
    let baseline = root.path().join("b.xml");
    std::fs::write(&baseline, WORKDIR_KEYS).unwrap();

    let out = run(&proj, &["--baseline", "../b.xml", "src"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );

    run(&proj, &["--baseline", "../b.xml", "--update-baseline", "src"]);
    let updated = std::fs::read_to_string(&baseline).unwrap();
    assert_eq!(updated.matches("<code>").count(), 2, "{updated}");

    let out = run(&proj, &["--baseline", "../b.xml", "src"]);
    assert!(out.status.success(), "{updated}");
}

#[test]
fn partial_update_keeps_workdir_relative_keys_of_a_baseline_elsewhere() {
    let dir = fixture(false);
    std::fs::create_dir(dir.path().join("build")).unwrap();
    let baseline = dir.path().join("build/b.xml");
    std::fs::write(&baseline, WORKDIR_KEYS).unwrap();

    let out = run(
        dir.path(),
        &[
            "--baseline",
            "build/b.xml",
            "--report-stale-baseline",
            "src/a.php",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    run(
        dir.path(),
        &["--baseline", "build/b.xml", "--update-baseline", "src/a.php"],
    );
    let updated = std::fs::read_to_string(&baseline).unwrap();
    assert!(updated.contains(r#"<file src="src/a.php">"#), "{updated}");
    assert!(updated.contains(r#"<file src="src/b.php">"#), "{updated}");
}
