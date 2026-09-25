//! Regression test for the 0.17.1 publishing bug.
//!
//! The crate was shipped to crates.io with an empty `STUB_FILES` because the
//! `stubs/` directory lived at the workspace root, outside the package
//! directory, and `cargo package` excluded it. Downstream consumers
//! (`php-lsp`) saw every PHP built-in reported as `UndefinedFunction` /
//! `UndefinedClass`.
//!
//! This test runs `cargo package --list` and asserts the resulting file
//! manifest includes `stubs/` entries — i.e. the artifact actually published
//! to crates.io will contain the stub source.

use std::process::Command;

#[test]
fn cargo_package_includes_stubs_directory() {
    let manifest_path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");

    let output = Command::new(env!("CARGO"))
        .args([
            "package",
            "--list",
            "--manifest-path",
            manifest_path,
            "--allow-dirty",
            "--no-verify",
        ])
        .output()
        .expect("running `cargo package --list` failed");

    assert!(
        output.status.success(),
        "`cargo package --list` exited non-zero.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let listed = String::from_utf8_lossy(&output.stdout);

    // A canonical built-in stub file. If this entry is missing the published
    // crate will silently emit an empty `STUB_FILES` and every downstream
    // consumer will lose all built-in symbol resolution.
    let canary = "stubs/Core/Core.php";
    assert!(
        listed.lines().any(|line| line.trim() == canary),
        "`cargo package --list` did not include `{canary}` — \
         the published crate would ship without built-in stubs, \
         repeating the 0.17.1 regression.\n\nFull listing:\n{listed}",
    );

    let stub_php_count = listed
        .lines()
        .filter(|line| line.starts_with("stubs/") && line.ends_with(".php"))
        .count();
    assert!(
        stub_php_count >= 100,
        "expected >=100 `stubs/**/*.php` entries in the package manifest, \
         got {stub_php_count}. The directory is present but largely empty — \
         either the move is partial or build.rs filters too aggressively.\n\n\
         Full listing:\n{listed}"
    );
}
