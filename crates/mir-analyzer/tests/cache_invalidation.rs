// Integration tests for cross-file cache invalidation (mir#61).
//
// When file B changes, dependents of B (files that extend/implement/use it)
// must have their cache entries evicted so Pass 2 re-analyzes them.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn dependent_file_is_reanalyzed_when_base_changes() {
    let src_dir = TempDir::new().unwrap();
    let cache_dir = TempDir::new().unwrap();

    // --- First run: Base defines method foo(), Child calls it — no issues ---
    let base = write(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function foo(): void {}\n}\n",
    );
    let child = write(
        &src_dir,
        "Child.php",
        "<?php\nclass Child extends Base {}\nfunction test(): void {\n    $c = new Child();\n    $c->foo();\n}\n",
    );

    let analyzer = ProjectAnalyzer::with_cache(cache_dir.path());
    let result1 = analyzer.analyze(&[base.clone(), child.clone()]);
    let undefined_method_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();
    assert_eq!(undefined_method_count, 0, "first run: no issues expected");

    // --- Modify Base: remove foo() ---
    write(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    // foo() removed\n}\n",
    );

    // Second run with a fresh analyzer (simulates a new CLI invocation) but same cache.
    let analyzer2 = ProjectAnalyzer::with_cache(cache_dir.path());
    let result2 = analyzer2.analyze(&[base.clone(), child.clone()]);
    let undefined_method_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();

    assert_eq!(
        undefined_method_count2, 1,
        "second run: Child must be re-analyzed and report UndefinedMethod for foo()"
    );
}

#[test]
fn unrelated_file_cache_entry_survives() {
    let src_dir = TempDir::new().unwrap();
    let cache_dir = TempDir::new().unwrap();

    let base = write(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function foo(): void {}\n}\n",
    );
    let unrelated = write(
        &src_dir,
        "Unrelated.php",
        "<?php\nfunction helper(): void {}\n",
    );

    // First run — populate cache for both files.
    let analyzer = ProjectAnalyzer::with_cache(cache_dir.path());
    analyzer.analyze(&[base.clone(), unrelated.clone()]);

    // Modify only Base.
    write(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function bar(): void {}\n}\n",
    );

    // Second run — Unrelated.php did not change and has no dependency on Base.
    // Its cache entry should survive (we cannot observe this directly from the
    // public API, but we verify no issues are raised for it and the run succeeds).
    let analyzer2 = ProjectAnalyzer::with_cache(cache_dir.path());
    let result = analyzer2.analyze(&[base.clone(), unrelated.clone()]);

    let unrelated_str = unrelated.to_string_lossy();
    let issues_for_unrelated: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.location.file.as_ref() == unrelated_str.as_ref())
        .collect();
    assert!(
        issues_for_unrelated.is_empty(),
        "unrelated file should produce no issues: {issues_for_unrelated:?}"
    );
}
