//! WS3 write-path guarantees:
//!
//! 1. `ingest_file_prepared` runs the Phase-1 warm-up at write time — the
//!    file's direct class references are lazy-loaded when the text lands,
//!    not serially at the front of the next references / re-analysis read.
//! 2. The reference index is maintained with replace semantics on every
//!    edit path, and a warm repeat of `indexed_references_to` is a pure
//!    posting lookup: no re-analysis, bounded `RefIndex` locks.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, IndexCancel, PhpVersion};

use self::common::create_temp_dir;

fn write_fixture(root: &std::path::Path) {
    fs::write(
        root.join("composer.json"),
        r#"{
  "autoload": {
    "psr-4": {
      "App\\": "src/",
      "Vendor\\": "vendor/VendorLib/src/"
    }
  }
}"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("vendor/VendorLib/src")).unwrap();
    fs::write(
        root.join("vendor/VendorLib/src/Dep.php"),
        "<?php\nnamespace Vendor;\nclass Dep { public function run(): int { return 1; } }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/Consumer.php"),
        "<?php\nnamespace App;\nuse Vendor\\Dep;\nclass Consumer { public function go(Dep $d): int { return $d->run(); } }\n",
    )
    .unwrap();
}

fn make_session(root: &std::path::Path) -> AnalysisSession {
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4 map");
    AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4))
}

#[test]
fn ingest_file_prepared_faults_in_direct_references_at_write_time() {
    let dir = create_temp_dir("hoist_prepared");
    let root = dir.path();
    write_fixture(root);

    let consumer: Arc<str> = Arc::from(root.join("src/Consumer.php").to_string_lossy().as_ref());
    let src: Arc<str> = Arc::from(fs::read_to_string(consumer.as_ref()).unwrap().as_str());

    // Plain ingest_file must NOT chase references (that's what keeps
    // load_class cascades one file wide).
    let session = make_session(root);
    session.ingest_file(consumer.clone(), src.clone());
    assert!(
        !session.contains_class("Vendor\\Dep"),
        "plain ingest_file must not lazy-load referenced classes"
    );

    // The prepared variant faults them in at write time.
    let session = make_session(root);
    session.ingest_file_prepared(consumer.clone(), src.clone());
    assert!(
        session.contains_class("Vendor\\Dep"),
        "ingest_file_prepared must lazy-load the file's direct references"
    );

    drop(dir);
}

#[test]
fn indexed_references_warm_repeat_is_pure_lookup() {
    let file_a: Arc<str> = Arc::from("hoist_a.php");
    let file_b: Arc<str> = Arc::from("hoist_b.php");
    let src_a = "<?php\nclass HoistBase { public function m(): int { return 1; } }\n";
    let src_b = "<?php\nclass HoistDep extends HoistBase {}\nfunction hb(): int { $x = new HoistBase(); return $x->m(); }\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    // Re-ingest exercises the definition-removal branch; both files land
    // through the ordinary edit path.
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file_prepared(file_b.clone(), Arc::from(src_b));

    let files = [file_a.clone(), file_b.clone()];
    let sym = mir_analyzer::Name::method("HoistBase", "m");
    let refs = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "expected the single $x->m() call site, got {refs:?}"
    );
    assert_eq!(refs[0].0, file_b);

    // Warm repeat: both files' postings are committed and fresh, so the
    // query must not re-analyze anything — bounded RefIndex locks (the
    // posting lookup itself) and no prepared-file churn.
    let locks_before = session.ref_index_lock_count();
    let warm = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(warm.len(), 1);
    // One lock per posting key (target class + hierarchy + name fallback) —
    // bounded by the key-set size, never by candidate-file count.
    let locks_taken = session.ref_index_lock_count() - locks_before;
    assert!(
        locks_taken <= 8,
        "warm repeat should be a bounded posting lookup, took {locks_taken} RefIndex locks"
    );

    // An edit sweep replaces (never appends) a file's postings: re-running
    // the sweep and the query must not duplicate results.
    let _ = session.reanalyze_files_cancellable(std::slice::from_ref(&file_b), &IndexCancel::new());
    let after_sweep = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(after_sweep.len(), 1, "replace semantics must hold");

    // Closing a file drops its postings.
    session.invalidate_file(file_b.as_ref());
    let after_close = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert!(
        after_close.is_empty(),
        "invalidated file's postings must be gone, got {after_close:?}"
    );

    // FileAnalyzer (the open-file flow) also commits with replace semantics.
    let parsed = php_rs_parser::parse(src_a);
    let _ = FileAnalyzer::new(&session).analyze(
        file_a.clone(),
        src_a,
        &parsed.program,
        &parsed.source_map,
    );
}

/// A fully-resolved commit survives workspace growth: registering an
/// unrelated file bumps the generation, but files whose analysis resolved
/// every name keep their postings — the warm repeat stays a bounded
/// posting lookup instead of re-verifying the whole candidate set.
#[test]
fn warm_repeat_stays_pure_lookup_after_unrelated_file_add() {
    let file_a: Arc<str> = Arc::from("immune_a.php");
    let file_b: Arc<str> = Arc::from("immune_b.php");
    let src_a = "<?php\nclass ImmuneBase { public function m(): int { return 1; } }\n";
    let src_b = "<?php\nfunction ib(): int { $x = new ImmuneBase(); return $x->m(); }\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file(file_b.clone(), Arc::from(src_b));

    let files = [file_a.clone(), file_b.clone()];
    let sym = mir_analyzer::Name::method("ImmuneBase", "m");
    let refs = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(refs.len(), 1);

    // Workspace grows: a brand-new unrelated file advances the generation.
    session.ingest_file(
        Arc::from("immune_unrelated.php"),
        Arc::from("<?php\nclass ImmuneUnrelated {}\n"),
    );

    let locks_before = session.ref_index_lock_count();
    let warm = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(warm.len(), 1);
    let locks_taken = session.ref_index_lock_count() - locks_before;
    assert!(
        locks_taken <= 8,
        "fully-resolved commits must survive the generation bump; \
         took {locks_taken} RefIndex locks"
    );
}

/// A byte-for-byte repeat query against unchanged state hits the memoization
/// cache — the freshness scan itself (not just the underlying re-analysis)
/// is skipped, not just made cheap. `ref_index_lock_count` alone can't prove
/// this: it stays flat either way, since a fully-committed candidate takes
/// no lock even on a cache miss.
#[test]
fn indexed_references_repeat_query_hits_cache() {
    let file_a: Arc<str> = Arc::from("cache_a.php");
    let file_b: Arc<str> = Arc::from("cache_b.php");
    let src_a = "<?php\nclass CacheBase { public function m(): int { return 1; } }\n";
    let src_b = "<?php\nfunction cb(): int { $x = new CacheBase(); return $x->m(); }\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file_prepared(file_b.clone(), Arc::from(src_b));

    let files = [file_a.clone(), file_b.clone()];
    let sym = mir_analyzer::Name::method("CacheBase", "m");
    let hits_before = session.ref_query_cache_hits();
    let first = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(first.len(), 1);
    assert_eq!(
        session.ref_query_cache_hits(),
        hits_before,
        "first call must populate the cache, not hit it"
    );

    let second = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(second, first);
    assert_eq!(
        session.ref_query_cache_hits(),
        hits_before + 1,
        "identical repeat query must hit the memoization cache"
    );
}

/// The cache must bound total memory by content size, not entry count: a
/// per-entry cap (e.g. "10,000 entries") gives no fixed byte ceiling when
/// one entry's `Vec` can be a handful of locations or many thousands for a
/// hot symbol. Three distinct symbols with deliberately different result
/// sizes (1, 2, 3 locations) prove the tracked total is the *sum of result
/// lengths*, not the entry count (which would read 3 after all three).
#[test]
fn indexed_references_cache_tracks_total_locations_not_entry_count() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let mut expected_total = 0usize;
    for (base, n_callers) in [("SizeA", 1usize), ("SizeB", 2), ("SizeC", 3)] {
        let base_file: Arc<str> = Arc::from(format!("{base}_base.php"));
        session.ingest_file(
            base_file.clone(),
            Arc::from(format!(
                "<?php\nclass {base} {{ public function m(): int {{ return 1; }} }}\n"
            )),
        );
        let mut files = vec![base_file];
        for c in 0..n_callers {
            let caller_file: Arc<str> = Arc::from(format!("{base}_caller{c}.php"));
            session.ingest_file_prepared(
                caller_file.clone(),
                Arc::from(format!(
                    "<?php\nfunction {base}_f{c}(): int {{ $x = new {base}(); return $x->m(); }}\n"
                )),
            );
            files.push(caller_file);
        }

        let sym = mir_analyzer::Name::method(base, "m");
        let refs = session
            .indexed_references_to(&sym, &files, false, &|| false)
            .expect("query not cancelled");
        assert_eq!(refs.len(), n_callers, "one call site per caller file");
        expected_total += refs.len();

        assert_eq!(
            session.ref_query_cache_locations(),
            expected_total,
            "tracked total must be the sum of result lengths so far, not the entry count"
        );
    }
}

/// [`AnalysisSession::files_mentioning_class`] answers from the persistent
/// class-mention index instead of a fresh scan on a repeat call — the same
/// public-API surfacing of `indexed_references_to`'s own internal gate
/// mechanism, exposed so a host can reuse it instead of maintaining an
/// equivalent from-scratch text scanner. `class_mention_stats().scans_recorded`
/// (bumped only on an actual per-file scan) proves the repeat call is a
/// pure lookup.
#[test]
fn files_mentioning_class_repeat_query_is_pure_lookup() {
    let file_a: Arc<str> = Arc::from("mention_a.php");
    let file_b: Arc<str> = Arc::from("mention_b.php");
    let src_a = "<?php\nclass MentionOwner { public function m(): int { return 1; } }\n";
    let src_b = "<?php\nfunction mb(): int { $x = new MentionOwner(); return $x->m(); }\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file(file_b.clone(), Arc::from(src_b));

    let files = [file_a.clone(), file_b.clone()];
    let first = session.files_mentioning_class(&files, "MentionOwner");
    assert_eq!(first.len(), 2, "both files textually mention MentionOwner");

    let scans_before = session.class_mention_stats().scans_recorded;
    assert!(scans_before > 0, "cold query must scan at least once");

    let second = session.files_mentioning_class(&files, "MentionOwner");
    assert_eq!(second.len(), 2);
    assert_eq!(
        session.class_mention_stats().scans_recorded,
        scans_before,
        "identical repeat query must not re-scan any file's text"
    );
}

/// A class name the universe has never seen (not declared anywhere in the
/// workspace) conservatively returns every candidate file — the index has
/// no basis to rule any of them out, so it must never under-report.
#[test]
fn files_mentioning_class_unknown_needle_returns_every_candidate() {
    let file_a: Arc<str> = Arc::from("unknown_a.php");
    let file_b: Arc<str> = Arc::from("unknown_b.php");

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(
        file_a.clone(),
        Arc::from("<?php\nfunction a(): int { return 1; }\n"),
    );
    session.ingest_file(
        file_b.clone(),
        Arc::from("<?php\nfunction b(): int { return 2; }\n"),
    );

    let files = [file_a, file_b];
    let result = session.files_mentioning_class(&files, "NeverDeclaredAnywhere");
    assert_eq!(
        result.len(),
        2,
        "a needle never admitted to the universe must conservatively return every file"
    );
}

/// A body-only edit — one that changes which lines reference the symbol
/// without touching any declaration — must invalidate the memoization
/// cache. `index_generation` (`workspace_revision`) deliberately does NOT
/// bump on a body-only edit (see `bump_workspace_revision`'s doc comment),
/// so a cache keyed on it alone would serve a stale answer here; this is
/// exactly why the cache keys on `text_revision` instead.
#[test]
fn indexed_references_cache_invalidates_on_body_only_edit() {
    let file_a: Arc<str> = Arc::from("invalidate_a.php");
    let file_b: Arc<str> = Arc::from("invalidate_b.php");
    let src_a = "<?php\nclass InvalidateBase { public function m(): int { return 1; } }\n";
    let src_b_v1 = "<?php\nfunction ib(): int { $x = new InvalidateBase(); return $x->m(); }\n";
    let src_b_v2 =
        "<?php\nfunction ib(): int { $x = new InvalidateBase(); $x->m(); return $x->m(); }\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file_prepared(file_b.clone(), Arc::from(src_b_v1));

    let files = [file_a.clone(), file_b.clone()];
    let sym = mir_analyzer::Name::method("InvalidateBase", "m");
    let gen_before = session.index_generation();
    let before_edit = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(
        before_edit.len(),
        1,
        "one $x->m() call site before the edit"
    );

    // Body-only edit: no declaration added/removed, so `index_generation`
    // must NOT advance — this is the case the coarser epoch would miss.
    session.ingest_file_prepared(file_b.clone(), Arc::from(src_b_v2));
    assert_eq!(
        session.index_generation(),
        gen_before,
        "a body-only edit must not bump the workspace generation"
    );

    let after_edit = session
        .indexed_references_to(&sym, &files, false, &|| false)
        .expect("query not cancelled");
    assert_eq!(
        after_edit.len(),
        2,
        "the second $x->m() call site must be picked up, not served from a stale cache entry"
    );
}
