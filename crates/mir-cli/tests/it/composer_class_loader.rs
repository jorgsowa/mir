use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn analyze_src(root: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mir"))
        .env_clear()
        .current_dir(root)
        .args(["--no-cache", "--no-progress", "src"])
        .output()
        .expect("failed to run mir binary");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn project(classmap: Option<&str>) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();
    write(
        root,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    if let Some(classmap) = classmap {
        write(root, "vendor/composer/autoload_classmap.php", classmap);
    }
    write(
        root,
        "vendor/composer/ClassLoader.php",
        "<?php\nnamespace Composer\\Autoload;\nclass ClassLoader {\n    /** @return array<string, list<string>> */\n    public function getPrefixesPsr4(): array { return []; }\n}\n",
    );
    write(
        root,
        "src/Boot.php",
        "<?php\nnamespace App;\nuse Composer\\Autoload\\ClassLoader;\nfinal class Boot {\n    public function __construct(private ClassLoader $loader) {}\n    /** @return array<string, list<string>> */\n    public function prefixes(): array { return $this->loader->getPrefixesPsr4(); }\n    public function missing(): void { $this->loader->nope(); }\n}\n",
    );
    dir
}

/// Composer's `autoload_real.php` requires `ClassLoader.php` directly, so it is never in the classmap.
#[test]
fn class_loader_resolves_without_classmap_entry() {
    let classmap = "<?php\n$vendorDir = dirname(__DIR__);\nreturn array(\n    'Composer\\\\InstalledVersions' => $vendorDir . '/composer/InstalledVersions.php',\n);\n";
    for dir in [project(Some(classmap)), project(None)] {
        let out = analyze_src(dir.path());
        assert!(
            !out.contains("UndefinedClass"),
            "ClassLoader must resolve, got:\n{out}"
        );
        assert!(
            out.contains("Method Composer\\Autoload\\ClassLoader::nope() does not exist"),
            "ClassLoader members must be checked, got:\n{out}"
        );
        assert_eq!(out.matches("UndefinedMethod").count(), 1, "got:\n{out}");
    }
}
