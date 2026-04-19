// Integration tests for the stub snapshot cache (Phase 2 of stub injection).
//
// Verifies that:
// 1. A warm cache hit skips PHP parsing and produces identical analysis results.
// 2. The cache is invalidated when the cache key changes (e.g. a user stub file changes).

use std::fs;
use std::path::PathBuf;

use mir_analyzer::{cache::AnalysisCache, ProjectAnalyzer};
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

fn analyzer_with_cache(cache_dir: &TempDir) -> ProjectAnalyzer {
    let mut a = ProjectAnalyzer::new();
    a.cache = Some(AnalysisCache::open(cache_dir.path()));
    a
}

#[test]
fn stub_cache_hit_produces_same_results() {
    let cache_dir = TempDir::new().unwrap();
    let src_dir = TempDir::new().unwrap();

    let php = write(&src_dir, "main.php", "<?php\n$s = strlen('hello');\n");

    // First run — cold cache, stubs parsed from PHP.
    let result1 = {
        let a = analyzer_with_cache(&cache_dir);
        a.load_stubs();
        a.analyze(std::slice::from_ref(&php)).issues
    };

    // Second run — warm cache, stubs restored from snapshot.
    let result2 = {
        let a = analyzer_with_cache(&cache_dir);
        a.load_stubs();
        a.analyze(std::slice::from_ref(&php)).issues
    };

    assert_eq!(
        result1.len(),
        result2.len(),
        "warm cache should produce identical issue count"
    );

    // Verify the stub cache file exists.
    assert!(
        cache_dir.path().join("stub-cache.json").exists(),
        "stub-cache.json should be written after first run"
    );
}

#[test]
fn stub_cache_invalidated_on_user_stub_change() {
    let cache_dir = TempDir::new().unwrap();
    let stubs_dir = TempDir::new().unwrap();

    // Write a user stub file.
    let stub_file = write(
        &stubs_dir,
        "MyStubs.php",
        "<?php\nfunction my_stub_fn(): string { return ''; }\n",
    );

    // First run — builds cache with my_stub_fn defined.
    {
        let mut a = analyzer_with_cache(&cache_dir);
        a.stub_files = vec![stub_file.clone()];
        a.load_stubs();
        assert!(
            a.codebase.functions.contains_key("my_stub_fn"),
            "stub function must be registered after first load"
        );
    }

    // Modify the stub file content — this must invalidate the cache.
    fs::write(
        &stub_file,
        "<?php\nfunction my_stub_fn_v2(): int { return 0; }\n",
    )
    .unwrap();

    // Second run — cache key changes, stubs re-parsed.
    {
        let mut a = analyzer_with_cache(&cache_dir);
        a.stub_files = vec![stub_file.clone()];
        a.load_stubs();
        assert!(
            a.codebase.functions.contains_key("my_stub_fn_v2"),
            "updated stub function must be registered after cache invalidation"
        );
        assert!(
            !a.codebase.functions.contains_key("my_stub_fn"),
            "old stub function must not appear after cache invalidation"
        );
    }
}
