use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// `app/` is the composer project (with a vendor package); `cfg/` holds only the config.
fn fixture(config_name: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();
    write(
        root,
        "app/composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    write(
        root,
        "app/vendor/composer/installed.json",
        r#"{"packages":[{"name":"acme/lib","install-path":"../acme/lib","autoload":{"psr-4":{"Acme\\":"src/"}}}]}"#,
    );
    write(
        root,
        "app/vendor/acme/lib/src/Greeter.php",
        "<?php namespace Acme; class Greeter { public function hi(): string { return 'hi'; } }\n",
    );
    write(
        root,
        "app/src/Service.php",
        "<?php namespace App; class Service { public function run(): string { return (new \\Acme\\Greeter())->hi(); } }\n",
    );
    write(
        root,
        &format!("cfg/{config_name}"),
        r#"<psalm><projectFiles><directory name="../app/src"/></projectFiles></psalm>"#,
    );
    dir
}

fn run(dir: &TempDir, args: &[&str]) -> String {
    let out: Output = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(dir.path().join("cfg"))
        .args(["--no-cache", "--no-progress"])
        .args(args)
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn psalm_xml_outside_root_resolves_vendor_classes() {
    let dir = fixture("psalm.xml");
    let combined = run(&dir, &[]);
    assert!(
        !combined.contains("UndefinedClass"),
        "vendor class must resolve via the composer root, got:\n{combined}"
    );
}

#[test]
fn mir_xml_outside_root_resolves_vendor_classes() {
    let dir = fixture("mir.xml");
    let combined = run(&dir, &[]);
    assert!(
        !combined.contains("UndefinedClass"),
        "vendor class must resolve via the composer root, got:\n{combined}"
    );
}

#[test]
fn explicit_config_outside_root_resolves_vendor_classes() {
    let dir = fixture("psalm.xml");
    let combined = run(&dir, &["--config", "psalm.xml"]);
    assert!(
        !combined.contains("UndefinedClass"),
        "vendor class must resolve via the composer root, got:\n{combined}"
    );
}
