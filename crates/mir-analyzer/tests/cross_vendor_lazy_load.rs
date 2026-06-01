// Regression tests for cross-vendor lazy class loading (mir#vendor-root bug).
//
// Root cause: find_composer_root_for_path used to stop at the first ancestor
// composer.json, which for a path inside vendor/ is the package's own manifest,
// not the project root. Psr4Map built from a sub-package manifest has no
// vendor/composer/installed.json context, so all cross-package references fire
// as UndefinedClass.
//
// These tests exercise the full pipeline:
//   Psr4Map::from_composer (project root) → AnalysisSession::with_psr4
//   → analyze_paths → lazy_load_from_body_issues → psr4.resolve(fqcn)
//
// They also include a negative-case counterpart that confirms the bug *would*
// occur when Psr4Map is loaded from the sub-package root (the old behavior),
// proving the fix is what makes the positive tests pass.

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};
use tempfile::TempDir;

use self::common::create_temp_dir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write(dir: &TempDir, rel: &str, content: &str) -> PathBuf {
    let path = dir.path().join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
    path
}

/// Build a minimal project root:
///   composer.json          – `App\\ → src/`
///   vendor/composer/installed.json – two packages:
///     laravel/serializable-closure  `Laravel\SerializableClosure\\ → src/`
///     nesbot/carbon                 `Carbon\\ → src/Carbon/`
///   vendor/laravel/framework/composer.json  – the sub-package's own manifest
///     (its PSR-4 maps Illuminate\\ → src/)
/// Returns the temp dir (dropped ⇒ cleaned up) and the absolute path to the
/// file inside vendor/laravel/framework that references cross-package FQCNs.
fn build_laravel_project(name: &str, consumer_src: &str) -> (TempDir, PathBuf) {
    let root = create_temp_dir(name);

    // --- project root composer.json ---
    write(
        &root,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );

    // --- vendor/composer/installed.json: three packages ---
    write(
        &root,
        "vendor/composer/installed.json",
        r#"{
            "packages": [
                {
                    "name": "laravel/framework",
                    "autoload": {"psr-4": {"Illuminate\\": "src/"}}
                },
                {
                    "name": "laravel/serializable-closure",
                    "autoload": {"psr-4": {"Laravel\\SerializableClosure\\": "src/"}}
                },
                {
                    "name": "nesbot/carbon",
                    "autoload": {"psr-4": {"Carbon\\": "src/Carbon/"}}
                }
            ]
        }"#,
    );

    // --- vendor/laravel/framework: package-own composer.json + consumer file ---
    // This composer.json must be skipped by find_composer_root_for_path; it only
    // knows about Illuminate\\ and has no installed.json for sibling packages.
    write(
        &root,
        "vendor/laravel/framework/composer.json",
        r#"{"name":"laravel/framework","autoload":{"psr-4":{"Illuminate\\":"src/"}}}"#,
    );

    let consumer_path = write(
        &root,
        "vendor/laravel/framework/src/Illuminate/Queue/Job.php",
        consumer_src,
    );

    // --- vendor/laravel/serializable-closure ---
    write(
        &root,
        "vendor/laravel/serializable-closure/src/SerializableClosure.php",
        "<?php\nnamespace Laravel\\SerializableClosure;\nclass SerializableClosure {\n    public function __construct(callable $closure) {}\n}\n",
    );

    // --- vendor/nesbot/carbon ---
    write(
        &root,
        "vendor/nesbot/carbon/src/Carbon/CarbonInterval.php",
        "<?php\nnamespace Carbon;\nclass CarbonInterval {\n    public function __construct(string $spec) {}\n    public function totalDays(): int { return 0; }\n}\n",
    );

    (root, consumer_path)
}

fn undefined_class_names(result: &mir_analyzer::AnalysisResult) -> Vec<String> {
    result
        .issues
        .iter()
        .filter_map(|i| {
            if let mir_issues::IssueKind::UndefinedClass { name } = &i.kind {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Positive tests: PSR4 loaded from the project root → no UndefinedClass
// ---------------------------------------------------------------------------

/// Analyzing a file inside vendor/laravel/framework/ that instantiates a class
/// from a sibling package (laravel/serializable-closure) must not emit
/// UndefinedClass when Psr4Map is loaded from the project root.
#[test]
fn cross_vendor_psr4_resolves_sibling_package_class() {
    let (root, consumer_path) = build_laravel_project(
        "cross_vendor_psr4",
        "<?php\nnamespace Illuminate\\Queue;\nuse Laravel\\SerializableClosure\\SerializableClosure;\nfunction pack(): SerializableClosure {\n    return new SerializableClosure(fn() => null);\n}\n",
    );

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(root.path())
            .expect("Psr4Map from project root"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let result = session.analyze_paths(&[consumer_path], &BatchOptions::new());

    let undefined = undefined_class_names(&result);
    assert!(
        undefined.is_empty(),
        "SerializableClosure should resolve via lazy loading from project-root PSR4 map; UndefinedClass: {undefined:?}"
    );
}

/// Same but using the FQCN directly (no `use` import) and a different sibling
/// package (nesbot/carbon).
#[test]
fn cross_vendor_psr4_resolves_fqcn_without_use_import() {
    let (root, consumer_path) = build_laravel_project(
        "cross_vendor_fqcn",
        "<?php\nnamespace Illuminate\\Support;\nfunction makeInterval(): \\Carbon\\CarbonInterval {\n    return new \\Carbon\\CarbonInterval('P1D');\n}\n",
    );

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(root.path())
            .expect("Psr4Map from project root"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let result = session.analyze_paths(&[consumer_path], &BatchOptions::new());

    let undefined = undefined_class_names(&result);
    assert!(
        undefined.is_empty(),
        "Carbon\\CarbonInterval should resolve via lazy loading from project-root PSR4 map; UndefinedClass: {undefined:?}"
    );
}

/// Both cross-vendor references in the same file.
#[test]
fn cross_vendor_psr4_resolves_multiple_sibling_packages_in_one_file() {
    let (root, consumer_path) = build_laravel_project(
        "cross_vendor_multi",
        "<?php\nnamespace Illuminate\\Queue;\nuse Laravel\\SerializableClosure\\SerializableClosure;\nfunction packAndSchedule(string $spec): void {\n    $c = new SerializableClosure(fn() => null);\n    $i = new \\Carbon\\CarbonInterval($spec);\n}\n",
    );

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(root.path())
            .expect("Psr4Map from project root"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let result = session.analyze_paths(&[consumer_path], &BatchOptions::new());

    let undefined = undefined_class_names(&result);
    assert!(
        undefined.is_empty(),
        "All sibling-package classes should resolve lazily; UndefinedClass: {undefined:?}"
    );
}

// ---------------------------------------------------------------------------
// Negative test: PSR4 loaded from the sub-package root → UndefinedClass fires
//
// This confirms the old behavior and proves that using the project-root Psr4Map
// (not the sub-package one) is what eliminates the false positives.
// ---------------------------------------------------------------------------

/// When Psr4Map is loaded from vendor/laravel/framework/ (as the old code did),
/// sibling-package classes cannot be resolved and UndefinedClass must fire.
/// This is the pre-fix behavior — if this assertion starts failing, the lazy
/// loader has somehow learned to resolve without a proper installed.json, which
/// would be a separate concern to investigate.
#[test]
fn cross_vendor_sub_package_root_produces_undefined_class() {
    let (root, consumer_path) = build_laravel_project(
        "cross_vendor_neg",
        "<?php\nnamespace Illuminate\\Queue;\nuse Laravel\\SerializableClosure\\SerializableClosure;\nfunction pack(): SerializableClosure {\n    return new SerializableClosure(fn() => null);\n}\n",
    );

    // Intentionally load from the sub-package root, simulating the old (buggy)
    // find_composer_root_for_path behavior.
    let sub_pkg_root = root.path().join("vendor/laravel/framework");
    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(&sub_pkg_root)
            .expect("Psr4Map from sub-package root"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let result = session.analyze_paths(&[consumer_path], &BatchOptions::new());

    let undefined = undefined_class_names(&result);
    assert!(
        !undefined.is_empty(),
        "Loading PSR4 from the sub-package root should leave sibling classes unresolvable \
         (pre-fix behavior); expected UndefinedClass but got none"
    );
    assert!(
        undefined.iter().any(|n| n.contains("SerializableClosure")),
        "expected UndefinedClass for SerializableClosure; got: {undefined:?}"
    );
}

// ---------------------------------------------------------------------------
// Classmap cross-vendor resolution
// ---------------------------------------------------------------------------

/// A vendor package that uses classmap (not PSR-4) autoloading is also resolved
/// correctly when the project-root Psr4Map is used — the classmap is read from
/// vendor/composer/autoload_classmap.php.
#[test]
fn cross_vendor_classmap_resolves_non_psr4_package() {
    let root = create_temp_dir("cross_vendor_classmap");

    write(
        &root,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );

    // autoload_classmap.php pointing to a non-PSR-4 legacy class.
    // The parse_composer_autoload_array format: 'FQCN' => $vendorDir . '/path'
    write(
        &root,
        "vendor/composer/autoload_classmap.php",
        "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'Legacy\\\\Widget' => $vendorDir . '/legacy/src/Widget.php',\n);\n",
    );

    // installed.json with no packages (classmap pkg is not PSR-4, just classmap).
    write(
        &root,
        "vendor/composer/installed.json",
        r#"{"packages":[]}"#,
    );

    write(
        &root,
        "vendor/legacy/src/Widget.php",
        "<?php\nnamespace Legacy;\nclass Widget {\n    public function render(): string { return ''; }\n}\n",
    );

    let consumer_path = write(
        &root,
        "src/Page.php",
        "<?php\nnamespace App;\nuse Legacy\\Widget;\nfunction buildPage(): string {\n    return (new Widget())->render();\n}\n",
    );

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(root.path())
            .expect("Psr4Map from project root"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let result = session.analyze_paths(&[consumer_path], &BatchOptions::new());

    let undefined = undefined_class_names(&result);
    assert!(
        undefined.is_empty(),
        "Legacy\\Widget should resolve via classmap lazy loading; UndefinedClass: {undefined:?}"
    );
}
