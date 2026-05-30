//! Regression test: inline suppression must survive the disk-cache hit path.
//!
//! `re_analyze_file` short-circuits on a content-hash cache hit and replays the
//! file's reference locations *without* re-registering the `SourceFile` input.
//! A fresh process (new session reading a previous run's disk cache) therefore
//! has no source to recompute suppression marks from — so the marks must be
//! baked into the cached issues at `cache.put` time. This test fails if a
//! cache hit ever returns an un-suppressed issue that the source silenced.

mod common;

use std::sync::Arc;

use mir_analyzer::cache::AnalysisCache;
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::create_temp_dir;

const SOURCE: &str =
    "<?php\nfunction f(): void {\n    new NoSuchClass(); // @mir-ignore UndefinedClass\n}\n";

fn undefined_class_issues(result: &mir_analyzer::AnalysisResult) -> (usize, usize) {
    let total = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .count();
    let visible = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass" && !i.suppressed)
        .count();
    (total, visible)
}

#[test]
fn inline_suppression_survives_cache_hit_in_fresh_session() {
    let cache_dir = create_temp_dir("inline_suppression_cache: cache");

    // First session: cold cache → miss path analyzes, bakes the suppression
    // mark, and writes the cached entry. The issue must be present but silenced.
    let cache1 = Arc::new(AnalysisCache::open(cache_dir.path()));
    let session1 = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache1.clone());
    let result1 = session1.re_analyze_file("test.php", SOURCE, &BatchOptions::new());
    let (total1, visible1) = undefined_class_issues(&result1);
    assert_eq!(total1, 1, "first run should detect the UndefinedClass");
    assert_eq!(visible1, 0, "first run: @mir-ignore should suppress it");

    // Persist to disk so a brand-new session can read it back.
    cache1.flush();

    // Second session: a fresh db (no SourceFile registered) reading the same
    // disk cache. Identical content → cache hit, which replays without
    // registering source. The baked mark must keep the issue suppressed.
    let cache2 = Arc::new(AnalysisCache::open(cache_dir.path()));
    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache2.clone());
    let result2 = session2.re_analyze_file("test.php", SOURCE, &BatchOptions::new());
    let (total2, visible2) = undefined_class_issues(&result2);
    assert_eq!(
        total2, 1,
        "second run should hit the cache and return the cached issue"
    );
    assert_eq!(
        visible2, 0,
        "cache hit in a fresh session must keep @mir-ignore suppression"
    );
}
