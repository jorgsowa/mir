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

/// Write `lib_file` under `src/` (PSR-4 root), write `consumer_file` at the
/// temp root, wire up PSR-4 for `App\\` → `src/`, and run analysis on the
/// consumer only.  The library must be discovered lazily.
fn analyze_with_psr4(
    lib_file: &str,
    lib_src: &str,
    consumer_file: &str,
    consumer_src: &str,
) -> mir_analyzer::project::AnalysisResult {
    let root = TempDir::new().unwrap();
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(root.path().join("src").join(lib_file), lib_src).unwrap();
    let consumer_path = write(&root, consumer_file, consumer_src);
    let psr4 = make_psr4(&root, "App\\\\", "src");
    let mut analyzer = ProjectAnalyzer::new();
    analyzer.psr4 = Some(psr4);
    analyzer.analyze(&[consumer_path])
}

#[test]
fn lazy_loads_parent_class_from_psr4() {
    let result = analyze_with_psr4(
        "Base.php",
        "<?php\nnamespace App;\nclass Base {\n    public function hello(): void {}\n}\n",
        // Use FQCN directly to avoid relying on `use`-alias resolution in the collector.
        "Child.php",
        "<?php\nclass Child extends \\App\\Base {}\nfunction test(): void {\n    $c = new Child();\n    $c->hello();\n}\n",
    );

    let undefined_method: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .collect();

    assert!(
        undefined_method.is_empty(),
        "hello() should be found after lazy-loading Base.php; got: {undefined_method:?}"
    );
}

#[test]
fn lazy_loads_interface_from_psr4() {
    let result = analyze_with_psr4(
        "Greetable.php",
        "<?php\nnamespace App;\ninterface Greetable {\n    public function greet(): string;\n}\n",
        "Greeter.php",
        "<?php\nuse App\\Greetable;\nclass Greeter implements Greetable {\n    public function greet(): string { return 'hi'; }\n}\n",
    );

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
    fs::create_dir_all(root.path().join("src")).unwrap();

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
}

#[test]
fn lazy_loads_enum_used_as_type_hint_from_psr4() {
    let result = analyze_with_psr4(
        "Status.php",
        "<?php\nnamespace App;\nenum Status: string {\n    case Active = 'active';\n    case Inactive = 'inactive';\n}\n",
        "Service.php",
        "<?php\nuse App\\Status;\nfunction getStatus(): Status { return Status::Active; }\nfunction check(Status $s): bool { return $s === Status::Active; }\n",
    );

    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "App\\Status enum should be found via lazy loading; got: {undefined_class:?}"
    );
}

#[test]
fn lazy_loads_class_used_only_in_static_call_from_psr4() {
    let result = analyze_with_psr4(
        "Helper.php",
        "<?php\nnamespace App;\nclass Helper {\n    public static function run(): string { return 'ok'; }\n}\n",
        "Consumer.php",
        "<?php\nuse App\\Helper;\nfunction doWork(): string { return Helper::run(); }\n",
    );

    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "App\\Helper class should be found via lazy loading; got: {undefined_class:?}"
    );
}

#[test]
fn lazy_loads_trait_used_in_class_from_psr4() {
    let result = analyze_with_psr4(
        "Greetable.php",
        "<?php\nnamespace App;\ntrait Greetable {\n    public function greet(): string { return 'hi'; }\n}\n",
        "Greeter.php",
        "<?php\nuse App\\Greetable;\nclass Greeter {\n    use Greetable;\n}\nfunction test(): void {\n    $g = new Greeter();\n    $g->greet();\n}\n",
    );

    let undefined: Vec<_> = result
        .issues
        .iter()
        .filter(|i| matches!(i.kind.name(), "UndefinedClass" | "UndefinedMethod"))
        .collect();

    assert!(
        undefined.is_empty(),
        "App\\Greetable trait should be found via lazy loading; got: {undefined:?}"
    );
}

#[test]
fn lazy_loads_interface_extended_by_interface_from_psr4() {
    let result = analyze_with_psr4(
        "Countable.php",
        "<?php\nnamespace App;\ninterface Countable {\n    public function count(): int;\n}\n",
        "ExtendedCountable.php",
        "<?php\nuse App\\Countable;\ninterface ExtendedCountable extends Countable {\n    public function isEmpty(): bool;\n}\nclass MyList implements ExtendedCountable {\n    public function count(): int { return 0; }\n    public function isEmpty(): bool { return true; }\n}\n",
    );

    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "App\\Countable interface should be found via lazy loading; got: {undefined_class:?}"
    );
}

#[test]
fn lazy_loads_interface_implemented_by_enum_from_psr4() {
    let result = analyze_with_psr4(
        "HasLabel.php",
        "<?php\nnamespace App;\ninterface HasLabel {\n    public function label(): string;\n}\n",
        "Status.php",
        "<?php\nuse App\\HasLabel;\nenum Status: string implements HasLabel {\n    case Active = 'active';\n    public function label(): string { return $this->value; }\n}\n",
    );

    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "App\\HasLabel interface should be found via lazy loading; got: {undefined_class:?}"
    );
}

// FQCNs used directly in function bodies (no `use` import) are only discovered
// during Pass 2 body analysis, after lazy loading has already run.  Fixing this
// requires a post-Pass-2 lazy-load phase: collect UndefinedClass diagnostics →
// PSR-4 resolve → load missing files → re-analyze affected files.
#[ignore = "known gap: FQCN-without-use requires post-Pass-2 lazy loading"]
#[test]
fn lazy_loads_fqcn_used_directly_without_use_import_from_psr4() {
    let result = analyze_with_psr4(
        "Helper.php",
        "<?php\nnamespace App;\nclass Helper {\n    public static function run(): string { return 'ok'; }\n}\n",
        "Consumer.php",
        "<?php\nfunction doWork(): string { return \\App\\Helper::run(); }\n",
    );

    let undefined_class: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();

    assert!(
        undefined_class.is_empty(),
        "App\\Helper should be found via lazy loading when used as FQCN; got: {undefined_class:?}"
    );
}
