// Integration tests for cross-file cache invalidation (mir#61).
//
// When file B changes, dependents of B (files that extend/implement/use it)
// must have their cache entries evicted so Pass 2 re-analyzes them.

use std::sync::Arc;

use mir_analyzer::cache::hash_content;
use mir_analyzer::{
    dead_code_issue_kinds, AnalysisSession, BatchOptions, IndexCancel, IndexParallelism, PhpVersion,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::common::{create_temp_dir, write_file};

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

    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );
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
    let mut session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );
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
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let opts = BatchOptions::new()
        .without_symbols()
        .with_suppressed(dead_code_issue_kinds().iter().copied());
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
    let mut session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let opts2 = BatchOptions::new()
        .without_symbols()
        .with_suppressed(dead_code_issue_kinds().iter().copied());
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
    let opts = BatchOptions::new()
        .without_symbols()
        .with_progress_callback(Arc::new(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        }));
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir);
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
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    session.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );

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

    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    session.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );

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

    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );

    // Pin cache.bin's mtime far in the past: any rewrite during the second run
    // stamps the current time, whatever the filesystem's mtime granularity.
    let cache_bin = cache_dir.path().join("cache.bin");
    let sentinel = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    std::fs::File::options()
        .write(true)
        .open(&cache_bin)
        .expect("cache.bin should exist after first run")
        .set_modified(sentinel)
        .unwrap();

    let mut session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(
        &[base.clone(), child.clone()],
        &BatchOptions::new().without_symbols(),
    );

    let mtime = std::fs::metadata(&cache_bin).unwrap().modified().unwrap();

    assert_eq!(
        mtime, sentinel,
        "an unchanged warm run must not rewrite cache.bin"
    );
    assert_eq!(
        result1.issues.len(),
        result2.issues.len(),
        "warm run must produce the same diagnostics"
    );
}

fn has_undefined_class(session: &mut AnalysisSession, path: &str, source: &str) -> bool {
    session
        .re_analyze_file(path, source, &BatchOptions::new().without_symbols())
        .issues
        .iter()
        .any(|issue| issue.kind.name() == "UndefinedClass")
}

#[test]
fn mirror_registration_evicts_only_negative_and_dependent_entries() {
    let dir = create_temp_dir("mirror selective invalidation");
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let consumer = "<?php\nnew Mage();\n";
    let unrelated = "<?php\nfunction unrelated(): void {}\n";

    assert!(has_undefined_class(
        &mut session,
        "/mirror/app.php",
        consumer
    ));
    assert!(!has_undefined_class(
        &mut session,
        "/mirror/unrelated.php",
        unrelated
    ));
    assert!(session
        .cache()
        .unwrap()
        .is_valid("/mirror/app.php", &hash_content(consumer)));
    assert!(session
        .cache()
        .unwrap()
        .is_valid("/mirror/unrelated.php", &hash_content(unrelated)));

    session.set_workspace_files(vec![(
        Arc::from("/mirror/Mage.php"),
        Arc::from("<?php\nclass Mage {}\n"),
    )]);

    assert!(
        !session
            .cache()
            .unwrap()
            .is_valid("/mirror/app.php", &hash_content(consumer)),
        "negative lookup must be evicted when its missing class is registered"
    );
    assert!(
        session
            .cache()
            .unwrap()
            .is_valid("/mirror/unrelated.php", &hash_content(unrelated)),
        "unrelated resolved cache entry must survive workspace growth"
    );
    assert!(
        !has_undefined_class(&mut session, "/mirror/app.php", consumer),
        "reanalysis after registration must resolve Mage"
    );
}

#[test]
fn body_only_mirror_edit_preserves_unrelated_cache_entry() {
    let dir = create_temp_dir("mirror body-only invalidation");
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.set_file_text(
        Arc::from("/mirror/edited.php"),
        Arc::from("<?php class Existing { function a(): void {} }"),
    );
    session.rebuild_workspace_symbol_index();
    let unrelated = "<?php\nfunction unrelated(): void {}\n";
    assert!(!has_undefined_class(
        &mut session,
        "/mirror/unrelated.php",
        unrelated
    ));

    session.set_file_text(
        Arc::from("/mirror/edited.php"),
        Arc::from("<?php class Existing { function b(): void {} }"),
    );
    session.settle_workspace_index();
    assert!(
        session
            .cache()
            .unwrap()
            .is_valid("/mirror/unrelated.php", &hash_content(unrelated)),
        "body-only mirror edits must not clear unrelated cache entries"
    );
}

#[test]
fn index_batch_registration_evicts_negative_but_keeps_unrelated_cache() {
    let dir = create_temp_dir("mirror index batch invalidation");
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let consumer = "<?php\nnew Mage();\n";
    let unrelated = "<?php\nfunction unrelated(): void {}\n";
    assert!(has_undefined_class(
        &mut session,
        "/mirror/app.php",
        consumer
    ));
    assert!(!has_undefined_class(
        &mut session,
        "/mirror/unrelated.php",
        unrelated
    ));

    session.index_batch(
        &[(
            Arc::from("/mirror/Mage.php"),
            Arc::from("<?php\nclass Mage {}\n"),
        )],
        IndexParallelism::Sequential,
        &IndexCancel::new(),
    );

    let cache = session.cache().unwrap();
    assert!(!cache.is_valid("/mirror/app.php", &hash_content(consumer)));
    assert!(cache.is_valid("/mirror/unrelated.php", &hash_content(unrelated)));
    assert!(!has_undefined_class(
        &mut session,
        "/mirror/app.php",
        consumer
    ));
}

#[test]
fn direct_input_registration_invalidates_persisted_negative_lookup() {
    let dir = create_temp_dir("mirror direct input invalidation");
    let consumer = "<?php\nnew Mage();\n";
    {
        let mut seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
        assert!(has_undefined_class(&mut seed, "/mirror/app.php", consumer));
        seed.flush_analysis_cache();
    }

    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    session.upsert_source_file(
        Arc::from("/mirror/Mage.php"),
        Arc::from("<?php\nclass Mage {}\n"),
        salsa::Durability::LOW,
    );
    assert!(!session
        .cache()
        .unwrap()
        .is_valid("/mirror/app.php", &hash_content(consumer)));
    assert!(!has_undefined_class(
        &mut session,
        "/mirror/app.php",
        consumer
    ));
}

#[test]
fn shadowed_definition_evicts_dependents_of_old_owner() {
    let dir = create_temp_dir("mirror shadow invalidation");
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    let old_owner = "/mirror/old_mage.php";
    let consumer_path = "/mirror/consumer.php";
    let consumer = "<?php\n(new Mage())->spell();\n";
    session.set_file_text(
        Arc::from(old_owner),
        Arc::from("<?php class Mage { function spell(): void {} }"),
    );
    session.rebuild_workspace_symbol_index();
    assert!(!has_undefined_class(&mut session, consumer_path, consumer));
    assert!(session
        .cache()
        .unwrap()
        .is_valid(consumer_path, &hash_content(consumer)));
    let mut reverse_deps = FxHashMap::default();
    reverse_deps.insert(
        old_owner.to_string(),
        FxHashSet::from_iter([consumer_path.to_string()]),
    );
    session.cache().unwrap().set_reverse_deps(reverse_deps);

    session.set_file_text(
        Arc::from("/mirror/new_mage.php"),
        Arc::from("<?php class Mage {}"),
    );
    session.settle_workspace_index();
    assert!(
        !session
            .cache()
            .unwrap()
            .is_valid(consumer_path, &hash_content(consumer)),
        "dependents of a shadowed definition must be invalidated"
    );
}

#[test]
fn matching_warm_cache_entries_survive_bulk_registration() {
    let dir = create_temp_dir("mirror matching cache");
    let files = [
        ("/mirror/one.php", "<?php function one(): void {}"),
        ("/mirror/two.php", "<?php function two(): void {}"),
    ];
    {
        let mut seed = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
        for (path, source) in files {
            assert!(!has_undefined_class(&mut seed, path, source));
        }
        seed.flush_analysis_cache();
    }
    let mut warm = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(dir.path());
    warm.set_workspace_files(
        files
            .into_iter()
            .map(|(path, source)| (Arc::from(path), Arc::from(source)))
            .collect::<Vec<_>>(),
    );
    for (path, source) in files {
        assert!(
            warm.cache().unwrap().is_valid(path, &hash_content(source)),
            "matching warm cache entry for {path} must survive registration"
        );
    }
}
