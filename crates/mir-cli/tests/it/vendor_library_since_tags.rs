use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// `@since`/`@removed` in a vendor package are the library's versions, not PHP's.
#[test]
fn vendor_since_and_removed_tags_do_not_hide_symbols() {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();
    write(
        root,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    write(
        root,
        "vendor/composer/installed.json",
        r#"{"packages":[{"name":"acme/lib","install-path":"../acme/lib","autoload":{"psr-4":{"Acme\\":"src/"},"files":["src/functions.php"]}}]}"#,
    );
    write(
        root,
        "vendor/acme/lib/src/Client.php",
        "<?php\nnamespace Acme;\nclass Client {\n    /** @since 8.4 */\n    public function send(): int { return 1; }\n}\n",
    );
    write(
        root,
        "vendor/acme/lib/src/functions.php",
        "<?php\nnamespace Acme;\n/** @removed 5.0 */\nfunction helper(): int { return 1; }\n",
    );
    write(
        root,
        "src/Use.php",
        "<?php\nnamespace App;\nfunction run(\\Acme\\Client $c): int { return $c->send() + \\Acme\\helper(); }\n",
    );

    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(root)
        .args(["--no-cache", "--no-progress", "--php-version", "8.0", "src"])
        .output()
        .expect("failed to run mir binary");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("UndefinedMethod") && !combined.contains("UndefinedFunction"),
        "vendor symbols must resolve regardless of library @since/@removed, got:\n{combined}"
    );
}
