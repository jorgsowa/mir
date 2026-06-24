// Integration tests for cross-file cache invalidation (mir#61).
//
// When file B changes, dependents of B (files that extend/implement/use it)
// must have their cache entries evicted so Pass 2 re-analyzes them.

mod common;

use mir_analyzer::{dead_code_issue_kinds, AnalysisSession, BatchOptions, PhpVersion};

use self::common::{create_temp_dir, write_file};

#[test]
fn dependent_file_is_reanalyzed_when_base_changes() {
    let src_dir = create_temp_dir("cache_invalidation: source files");
    let cache_dir = create_temp_dir("cache_invalidation: cache");

    // --- First run: Base defines method foo(), Child calls it — no issues ---
    let base = write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function foo(): void {}\n}\n",
    );
    let child = write_file(
        &src_dir,
        "Child.php",
        "<?php\nclass Child extends Base {}\nfunction test(): void {\n    $c = new Child();\n    $c->foo();\n}\n",
    );

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());
    let undefined_method_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();
    assert_eq!(undefined_method_count, 0, "first run: no issues expected");

    // --- Modify Base: remove foo() ---
    write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    // foo() removed\n}\n",
    );

    // Second run with a fresh analyzer (simulates a new CLI invocation) but same cache.
    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());
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
    let src_dir = create_temp_dir("unrelated_file: source files");
    let cache_dir = create_temp_dir("unrelated_file: cache");

    let base = write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function foo(): void {}\n}\n",
    );
    let unrelated = write_file(
        &src_dir,
        "Unrelated.php",
        "<?php\nfunction helper(): void {}\n",
    );

    // First run — populate cache for both files. Suppress the dead-code
    // group so the bare `helper()` function in Unrelated.php doesn't
    // surface as `UnusedFunction` in the assertions below.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let opts = BatchOptions::new().with_suppressed(dead_code_issue_kinds().iter().copied());
    session.analyze_paths(&[base.clone(), unrelated.clone()], &opts);

    // Modify only Base.
    write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function bar(): void {}\n}\n",
    );

    // Second run — Unrelated.php did not change and has no dependency on Base.
    // Its cache entry should survive (we cannot observe this directly from the
    // public API, but we verify no issues are raised for it and the run succeeds).
    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let opts2 = BatchOptions::new().with_suppressed(dead_code_issue_kinds().iter().copied());
    let result = session2.analyze_paths(&[base.clone(), unrelated.clone()], &opts2);

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

/// Count how many files the body pass actually (re)analyzes on `paths`.
/// `on_file_done` fires once per analyzed file; cache hits return before it,
/// so this measures real re-analysis, not cache replays.
fn reanalyzed_count(cache_dir: &std::path::Path, paths: &[std::path::PathBuf]) -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let n = Arc::new(AtomicUsize::new(0));
    let counter = n.clone();
    let opts = BatchOptions::new().with_progress_callback(Arc::new(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    }));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir);
    session.analyze_paths(paths, &opts);
    n.load(Ordering::Relaxed)
}

#[test]
fn body_only_change_to_base_does_not_reanalyze_dependent() {
    // The firewall: editing the *body* of a declared-return method in Base
    // (signature unchanged) must re-analyze Base but leave Child's cached
    // result in place.
    let src_dir = create_temp_dir("firewall: src");
    let cache_dir = create_temp_dir("firewall: cache");

    let base = write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base { public function foo(): int { return 1; } }\n",
    );
    let child = write_file(
        &src_dir,
        "Child.php",
        "<?php\nclass Child extends Base {\n    public function bar(): int { return $this->foo(); }\n}\n",
    );

    // Cold run populates the cache for both files.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    session.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());

    // Body-only edit to Base::foo — declared return type `int` is unchanged.
    write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base { public function foo(): int { $x = 41; return $x + 1; } }\n",
    );

    let reanalyzed = reanalyzed_count(cache_dir.path(), &[base.clone(), child.clone()]);
    assert_eq!(
        reanalyzed, 1,
        "only Base should be re-analyzed; Child's cached result must survive a body-only change"
    );
}

#[test]
fn signature_change_to_base_reanalyzes_dependent() {
    // Control for the firewall: changing Base's *signature* (return type) must
    // cascade to Child, so both are re-analyzed.
    let src_dir = create_temp_dir("firewall_control: src");
    let cache_dir = create_temp_dir("firewall_control: cache");

    let base = write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base { public function foo(): int { return 1; } }\n",
    );
    let child = write_file(
        &src_dir,
        "Child.php",
        "<?php\nclass Child extends Base {\n    public function bar(): int { return $this->foo(); }\n}\n",
    );

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    session.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());

    // Signature change: foo(): int -> foo(): string.
    write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base { public function foo(): string { return 'x'; } }\n",
    );

    let reanalyzed = reanalyzed_count(cache_dir.path(), &[base.clone(), child.clone()]);
    assert_eq!(
        reanalyzed, 2,
        "a signature change to Base must cascade re-analysis to Child"
    );
}

#[test]
fn warm_run_without_changes_does_not_rewrite_cache() {
    // A re-run over an unchanged file set must not recompute the reverse-dep
    // graph or rewrite cache.bin: every file hits the cache, so the on-disk
    // graph is already accurate. We assert the cache file's mtime is unchanged
    // across the second run, and that results stay correct.
    let src_dir = create_temp_dir("warm_run: source files");
    let cache_dir = create_temp_dir("warm_run: cache");

    let base = write_file(
        &src_dir,
        "Base.php",
        "<?php\nclass Base {\n    public function foo(): void {}\n}\n",
    );
    let child = write_file(
        &src_dir,
        "Child.php",
        "<?php\nclass Child extends Base {}\nfunction test(): void {\n    (new Child())->foo();\n}\n",
    );

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());

    let cache_bin = cache_dir.path().join("cache.bin");
    let mtime1 = std::fs::metadata(&cache_bin)
        .expect("cache.bin should exist after first run")
        .modified()
        .unwrap();

    // Advance the clock past filesystem mtime granularity so a real rewrite
    // during the second run would be observable.
    std::thread::sleep(std::time::Duration::from_millis(20));

    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(&[base.clone(), child.clone()], &BatchOptions::new());

    let mtime2 = std::fs::metadata(&cache_bin).unwrap().modified().unwrap();

    assert_eq!(
        mtime1, mtime2,
        "an unchanged warm run must not rewrite cache.bin"
    );
    assert_eq!(
        result1.issues.len(),
        result2.issues.len(),
        "warm run must produce the same diagnostics"
    );
}
