use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn fixture_with_stale_baseline() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("bad.php"),
        "<?php\nfunction f(): void { echo $missing; }\n",
    )
    .expect("failed to write PHP fixture");
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<files>
  <file src="./bad.php">
    <UndefinedVariable>
      <code><![CDATA[$missing]]></code>
      <code><![CDATA[$old]]></code>
    </UndefinedVariable>
  </file>
</files>
"#,
    )
    .expect("failed to write baseline");
    dir
}

fn fixture_with_hidden_info_baseline() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("info.php"),
        "<?php\nfunction f($unused): void {}\n",
    )
    .expect("failed to write PHP fixture");
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<files>
  <file src="./info.php">
    <UnusedParam>
      <code><![CDATA[$unused]]></code>
    </UnusedParam>
  </file>
</files>
"#,
    )
    .expect("failed to write baseline");
    dir
}

fn run(dir: &Path, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.current_dir(dir);
    cmd.args(["--no-cache", "--no-progress"]);
    cmd.args(extra_args);
    cmd.arg(".");
    cmd.output().expect("failed to run mir binary")
}

#[test]
fn report_stale_baseline_fails_when_baseline_entry_no_longer_matches() {
    let dir = fixture_with_stale_baseline();

    let out = run(
        dir.path(),
        &[
            "--baseline",
            "psalm-baseline.xml",
            "--report-stale-baseline",
            "--format",
            "json",
        ],
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "stale baseline entries should fail the run\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "[]",
        "the live issue is still suppressed by the matching baseline entry"
    );
    assert!(
        stderr.contains("1 stale baseline issue(s) no longer emitted"),
        "stale baseline count should be reported, got:\n{stderr}"
    );
    assert!(
        stderr.contains("./bad.php: UndefinedVariable ($old)"),
        "stale baseline entry should identify file, kind, and snippet, got:\n{stderr}"
    );
    assert!(
        !stderr.contains("$missing"),
        "the consumed baseline entry must not be reported as stale, got:\n{stderr}"
    );
}

#[test]
fn stale_baseline_reporting_is_opt_in() {
    let dir = fixture_with_stale_baseline();

    let out = run(
        dir.path(),
        &["--baseline", "psalm-baseline.xml", "--format", "json"],
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "stale baseline entries should not fail without --report-stale-baseline\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout.trim(), "[]");
    assert!(
        !stderr.contains("stale baseline"),
        "stale reporting should be opt-in, got:\n{stderr}"
    );
}

#[test]
fn hidden_info_issues_do_not_keep_default_baselines_fresh() {
    let dir = fixture_with_hidden_info_baseline();

    let out = run(
        dir.path(),
        &[
            "--baseline",
            "psalm-baseline.xml",
            "--report-stale-baseline",
            "--format",
            "json",
        ],
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "hidden info-level issues should not consume default-output baselines\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "[]",
        "the info-level issue is hidden in the default output"
    );
    assert!(
        stderr.contains("1 stale baseline issue(s) no longer emitted"),
        "hidden info baseline entry should be reported stale, got:\n{stderr}"
    );
    assert!(
        stderr.contains("./info.php: UnusedParam ($unused)"),
        "stale hidden info entry should identify file, kind, and snippet, got:\n{stderr}"
    );
}
