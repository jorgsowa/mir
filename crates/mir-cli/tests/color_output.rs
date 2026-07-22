//! End-to-end coverage for `--color` / `NO_COLOR` / `CLICOLOR(_FORCE)` handling
//! (issue #295). Spawns the real `mir` binary so the precedence between the
//! flag, env vars, and TTY auto-detection is verified the way a user or CI
//! pipeline actually experiences it, not just at the owo-colors call site.

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn has_ansi(bytes: &[u8]) -> bool {
    bytes.contains(&0x1b)
}

fn fixture_with_issue() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("bad.php"),
        "<?php\nfunction f() { echo $undefined; }\n",
    )
    .expect("failed to write fixture");
    dir
}

/// Runs the `mir` binary against `dir` with a clean environment (only the
/// vars under test are set), so results don't depend on the ambient shell or
/// CI environment the test suite happens to run in.
fn run(dir: &Path, extra_args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.args(["--no-cache", "--no-progress", "--stats"]);
    cmd.args(extra_args);
    cmd.arg(dir);
    cmd.output().expect("failed to run mir binary")
}

#[test]
fn auto_defaults_to_plain_when_piped() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &[], &[]);
    assert!(
        !has_ansi(&out.stdout),
        "stdout should be plain: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        !has_ansi(&out.stderr),
        "stderr should be plain: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn color_always_forces_ansi_on_both_streams() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &["--color", "always"], &[]);
    assert!(has_ansi(&out.stdout), "stdout should be colored");
    assert!(has_ansi(&out.stderr), "stderr should be colored");
}

#[test]
fn color_never_forces_plain_even_under_clicolor_force() {
    let dir = fixture_with_issue();
    let out = run(
        dir.path(),
        &["--color", "never"],
        &[("CLICOLOR_FORCE", "1")],
    );
    assert!(!has_ansi(&out.stdout));
    assert!(!has_ansi(&out.stderr));
}

#[test]
fn explicit_flag_overrides_no_color_env() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &["--color", "always"], &[("NO_COLOR", "1")]);
    assert!(has_ansi(&out.stdout));
    assert!(has_ansi(&out.stderr));
}

#[test]
fn no_color_env_disables_auto_color() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &[], &[("NO_COLOR", "1")]);
    assert!(!has_ansi(&out.stdout));
    assert!(!has_ansi(&out.stderr));
}

#[test]
fn clicolor_force_enables_auto_color_when_piped() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &[], &[("CLICOLOR_FORCE", "1")]);
    assert!(has_ansi(&out.stdout));
    assert!(has_ansi(&out.stderr));
}

#[test]
fn clicolor_force_wins_over_no_color_in_auto_mode() {
    // Documents actual `supports-color` behavior (ported from the npm package
    // of the same name): FORCE_COLOR/CLICOLOR_FORCE are checked before
    // NO_COLOR, so an explicit force wins even if NO_COLOR is also set.
    let dir = fixture_with_issue();
    let out = run(
        dir.path(),
        &[],
        &[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")],
    );
    assert!(has_ansi(&out.stdout));
    assert!(has_ansi(&out.stderr));
}

#[test]
fn json_format_stdout_never_carries_ansi_even_when_forced() {
    let dir = fixture_with_issue();
    let out = run(dir.path(), &["--color", "always", "--format", "json"], &[]);
    assert!(
        !has_ansi(&out.stdout),
        "JSON payload on stdout must stay machine-readable regardless of --color"
    );
}
