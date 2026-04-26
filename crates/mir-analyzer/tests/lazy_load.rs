// Integration tests for on-demand PSR-4 lazy class loading (mir#50).
//
// When Pass 2 would encounter an unknown parent class or interface, the
// lazy-loading phase should resolve it via the PSR-4 map, run Pass 1 on
// the discovered file, and re-finalize the codebase so that inheritance
// relationships are visible during Pass 2.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

/// Build a minimal composer.json and return a `Psr4Map`.
fn make_psr4(root: &TempDir, prefix: &str, subdir: &str) -> Arc<mir_analyzer::composer::Psr4Map> {
    fs::write(
        root.path().join("composer.json"),
        format!(r#"{{"autoload":{{"psr-4":{{"{prefix}":"{subdir}/"}}}}}}"#),
    )
    .unwrap();
    let map =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map creation");
    Arc::new(map)
}

#[test]
fn lazy_loads_parent_class_from_psr4() {
    let root = TempDir::new().unwrap();

    // Create the PSR-4 source directory
    let src_dir = root.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Write the base class (NOT in the initial file list — must be discovered lazily)
    fs::write(
        src_dir.join("Base.php"),
        "<?php\nnamespace App;\nclass Base {\n    public function hello(): void {}\n}\n",
    )
    .unwrap();

    // Write the child class (in the initial file list); use FQCN directly to
    // avoid relying on `use`-alias resolution in the collector.
    let child_path = write(
        &root,
        "Child.php",
        "<?php\nclass Child extends \\App\\Base {}\nfunction test(): void {\n    $c = new Child();\n    $c->hello();\n}\n",
    );

    let psr4 = make_psr4(&root, "App\\\\", "src");

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.psr4 = Some(psr4);

    // Only pass Child.php — Base.php must be discovered via PSR-4 lazy loading
    let result = analyzer.analyze(&[child_path]);

    let undefined_method_issues: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .collect();

    assert!(
        undefined_method_issues.is_empty(),
        "hello() should be found after lazy-loading Base.php; got: {undefined_method_issues:?}"
    );
}

#[test]
fn lazy_loads_interface_from_psr4() {
    let root = TempDir::new().unwrap();
    let src_dir = root.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Write the interface (NOT in initial file list)
    fs::write(
        src_dir.join("Greetable.php"),
        "<?php\nnamespace App;\ninterface Greetable {\n    public function greet(): string;\n}\n",
    )
    .unwrap();

    // Write a class that implements the interface (in initial file list)
    let impl_path = write(
        &root,
        "Greeter.php",
        "<?php\nuse App\\Greetable;\nclass Greeter implements Greetable {\n    public function greet(): string { return 'hi'; }\n}\n",
    );

    let psr4 = make_psr4(&root, "App\\\\", "src");

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.psr4 = Some(psr4);

    let result = analyzer.analyze(&[impl_path]);

    // No UndefinedClass for the interface
    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "Greetable interface should be found via lazy loading; got: {undefined_class:?}"
    );
}

#[test]
fn does_not_loop_when_class_has_no_psr4_match() {
    let root = TempDir::new().unwrap();
    let src_dir = root.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Write a class that extends something that does NOT exist on disk
    let child_path = write(
        &root,
        "Orphan.php",
        "<?php\nclass Orphan extends \\NonExistent\\Ghost {}\n",
    );

    let psr4 = make_psr4(&root, "App\\\\", "src");

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.psr4 = Some(psr4);

    // Should terminate without hanging or panicking
    let _result = analyzer.analyze(&[child_path]);
    // The test passing (no panic/hang) is the assertion
}
