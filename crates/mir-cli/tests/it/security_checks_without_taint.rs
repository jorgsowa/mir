//! Security checks that read request data and its guards give the same result with `--no-taint`.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn run(php: &str, extra: &[&str]) -> String {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("mir.xml"), "<mir></mir>").unwrap();
    std::fs::write(dir.path().join("a.php"), php).unwrap();
    output(dir.path(), extra)
}

fn output(dir: &Path, extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir)
        .args(["--no-cache", "--no-progress", "-c", "mir.xml"])
        .args(extra)
        .arg(".")
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const BOUNDED_AMOUNT: &str = "<?php
function pad(): string {
    $n = (int) $_GET['n'];
    if ($n > 100) {
        return '';
    }
    return str_repeat('x', $n);
}
";

const UNCHECKED_PASSWORD: &str = "<?php
function login(\\LDAP\\Connection $ldap): bool {
    $password = $_POST['password'];
    return ldap_bind($ldap, 'uid=alice', $password);
}
";

const DENY_LIST: &str = "<?php
function filter(): void {
    $q = $_GET['q'];
    if (preg_match('/<script/iu', $q)) {
        exit;
    }
}
";

#[test]
fn bound_check_holds_without_taint() {
    for flags in [&[][..], &["--no-taint"][..]] {
        assert!(!run(BOUNDED_AMOUNT, flags).contains("ResourceExhaustion"));
    }
}

#[test]
fn ldap_bind_reported_without_taint() {
    for flags in [&[][..], &["--no-taint"][..]] {
        assert!(run(UNCHECKED_PASSWORD, flags).contains("LdapAnonymousBind"));
    }
}

#[test]
fn deny_list_match_reported_without_taint() {
    for flags in [&[][..], &["--no-taint"][..]] {
        assert!(run(DENY_LIST, flags).contains("InsecureCheckResult"));
    }
}
