//! Integration tests for the Pass 1 definition cache.
use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

/// Helper: run analysis on `src` with a cache dir.
/// Returns issues so they can be inspected.
fn analyze_with_cache(src: &str, cache_dir: &TempDir) -> Vec<mir_analyzer::Issue> {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.php");
    std::fs::write(&file, src).unwrap();
    let mut analyzer = ProjectAnalyzer::new();
    analyzer.enable_cache(cache_dir.path());
    analyzer.analyze(&[file]).issues
}

#[test]
fn second_run_produces_identical_issues() {
    // A file that has an undefined-method error (must be inside a function body
    // so the analyzer walks Pass 2 for that scope).
    let src = r#"<?php
class Foo {}
function test(): void {
    $f = new Foo();
    $f->missing();
}
"#;
    let cache_dir = TempDir::new().unwrap();
    let first = analyze_with_cache(src, &cache_dir);
    let second = analyze_with_cache(src, &cache_dir);

    assert!(
        !first.is_empty(),
        "expected at least one issue on first run"
    );
    assert_eq!(
        first.len(),
        second.len(),
        "second run (cache hit) must produce the same number of issues"
    );
}

#[test]
fn file_level_constants_survive_cache_hit() {
    // Two files: one defines a constant, the other uses it.
    // On the second run, the first file is a cache hit; the constant must still
    // be present for the second file's analysis to succeed.
    let dir = TempDir::new().unwrap();
    let defines = dir.path().join("defines.php");
    let uses = dir.path().join("uses.php");
    std::fs::write(&defines, "<?php\nconst MY_CONST = 42;\n").unwrap();
    std::fs::write(&uses, "<?php\n/** @var int $x */\n$x = MY_CONST;\n").unwrap();

    let cache_dir = TempDir::new().unwrap();

    // First run: both files are cache misses; fills the cache.
    let first = {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(&[defines.clone(), uses.clone()]).issues
    };

    // Second run: defines.php is a cache hit.
    let second = {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(&[defines.clone(), uses.clone()]).issues
    };

    assert_eq!(
        first.len(),
        second.len(),
        "cache hit must not introduce new issues (constants must survive replay)"
    );
}

#[test]
fn changed_file_invalidates_snapshot() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.php");
    let cache_dir = TempDir::new().unwrap();

    std::fs::write(&file, "<?php\nclass A {}\n").unwrap();

    // First run: populates cache.
    {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(std::slice::from_ref(&file));
    }

    // Change the file.
    std::fs::write(&file, "<?php\nclass B {}\n").unwrap();

    // Second run: stale snapshot must be rejected; class B must be known.
    let mut analyzer = ProjectAnalyzer::new();
    analyzer.enable_cache(cache_dir.path());
    analyzer.analyze(std::slice::from_ref(&file));

    assert!(
        analyzer.codebase().classes.contains_key("B"),
        "class B must be in codebase after file change"
    );
    assert!(
        !analyzer.codebase().classes.contains_key("A"),
        "stale class A must not appear after file change"
    );
}

#[test]
fn vendor_types_survive_cache_hit() {
    // Simulate a vendor file that defines a class; project file references it.
    // Second run: vendor file is a collect_types_only cache hit.
    let dir = TempDir::new().unwrap();
    let vendor = dir.path().join("vendor.php");
    let project = dir.path().join("project.php");
    let cache_dir = TempDir::new().unwrap();

    std::fs::write(
        &vendor,
        "<?php\nclass VendorClass { public function hello(): void {} }\n",
    )
    .unwrap();
    std::fs::write(&project, "<?php\n$v = new VendorClass();\n$v->hello();\n").unwrap();

    let run = |cache_dir: &TempDir| {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.collect_types_only(std::slice::from_ref(&vendor));
        analyzer.analyze(std::slice::from_ref(&project)).issues
    };

    let first = run(&cache_dir);
    let second = run(&cache_dir);

    assert_eq!(
        first.len(),
        second.len(),
        "vendor cache hit must not lose class definitions"
    );
}

#[test]
fn re_analyze_file_snapshot_used_on_next_run() {
    // Populate the cache with class A, then re_analyze the same file to class B.
    // The snapshot written by re_analyze_file must be used as a cache hit on the
    // next analyze() run, producing class B (not stale class A) in the codebase.
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("subject.php");
    let cache_dir = TempDir::new().unwrap();

    std::fs::write(&file, "<?php\nclass A {}\n").unwrap();

    // First run: cold miss, populates cache with class A.
    {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(std::slice::from_ref(&file));
    }

    // Incremental edit: re-analyze in memory with class B content.
    // This rebuilds the snapshot so the cache entry now describes class B.
    let new_content = "<?php\nclass B {}\n";
    {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(std::slice::from_ref(&file)); // warm up codebase
        analyzer.re_analyze_file(file.to_str().unwrap(), new_content);
    }

    // Write the new content to disk so the hash matches the re_analyze snapshot.
    std::fs::write(&file, new_content).unwrap();

    // Next analyze() run: should be a cache hit for the B snapshot written above.
    let mut analyzer = ProjectAnalyzer::new();
    analyzer.enable_cache(cache_dir.path());
    analyzer.analyze(std::slice::from_ref(&file));

    assert!(
        analyzer.codebase().classes.contains_key("B"),
        "class B must be in codebase (snapshot written by re_analyze_file)"
    );
    assert!(
        !analyzer.codebase().classes.contains_key("A"),
        "stale class A must not appear after re_analyze_file"
    );
}

#[test]
fn vendor_namespace_survives_cache_hit() {
    // Vendor file declares a namespaced class. On the second run it is a cache
    // hit; the namespace must be replayed so the class is accessible by its FQN.
    let dir = TempDir::new().unwrap();
    let vendor = dir.path().join("vendor.php");
    let project = dir.path().join("project.php");
    let cache_dir = TempDir::new().unwrap();

    std::fs::write(
        &vendor,
        "<?php\nnamespace Acme;\nclass Widget { public function render(): void {} }\n",
    )
    .unwrap();
    // Reference the class by its FQN so we can verify it is accessible.
    std::fs::write(
        &project,
        "<?php\n$w = new \\Acme\\Widget();\n$w->render();\n",
    )
    .unwrap();

    let run = |cache_dir: &TempDir| {
        let mut analyzer = ProjectAnalyzer::new();
        analyzer.enable_cache(cache_dir.path());
        analyzer.collect_types_only(std::slice::from_ref(&vendor));
        analyzer.analyze(std::slice::from_ref(&project)).issues
    };

    let first = run(&cache_dir);
    let second = run(&cache_dir);

    assert_eq!(
        first.len(),
        second.len(),
        "vendor cache hit must not lose namespaced class (namespace must survive replay)"
    );
}

#[test]
fn lazy_loaded_classes_survive_cache_hit() {
    // Set up a minimal PSR-4 project: a project file references App\Service,
    // which lives in src/Service.php and is NOT in the initial analyzed file list.
    // lazy_load_missing_classes discovers and caches it on the first run.
    // The second run must use the cached snapshot and still see App\Service.
    let root = TempDir::new().unwrap();

    // composer.json
    std::fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();

    // src/Service.php — NOT included in the analyzed file list; discovered lazily.
    let src_dir = root.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("Service.php"),
        "<?php\nnamespace App;\nclass Service { public function run(): void {} }\n",
    )
    .unwrap();

    // Project file that references App\Service.
    let project_file = root.path().join("main.php");
    std::fs::write(
        &project_file,
        "<?php\nuse App\\Service;\n$s = new Service();\n$s->run();\n",
    )
    .unwrap();

    let cache_dir = TempDir::new().unwrap();

    let run = |cache_dir: &TempDir| {
        let (mut analyzer, _map) =
            ProjectAnalyzer::from_composer(root.path()).expect("composer setup");
        analyzer.enable_cache(cache_dir.path());
        analyzer.analyze(std::slice::from_ref(&project_file)).issues
    };

    let first = run(&cache_dir);
    let second = run(&cache_dir);

    assert_eq!(
        first.len(),
        second.len(),
        "lazy-loaded class must survive cache hit (App\\Service must be known on second run)"
    );
}
