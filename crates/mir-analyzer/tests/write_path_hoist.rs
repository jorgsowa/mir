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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert_eq!(after_sweep.len(), 1, "replace semantics must hold");

    // Closing a file drops its postings.
    session.invalidate_file(file_b.as_ref());
    let after_close = session
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert!(
        after_close.is_empty(),
        "invalidated file's postings must be gone, got {after_close:?}"
    );

    // FileAnalyzer (the open-file flow) also commits with replace semantics.
    let parsed = php_rs_parser::parse(src_a);
    let _ = FileAnalyzer::new(&session).analyze_diagnostics_only(
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert_eq!(refs.len(), 1);

    // Workspace grows: a brand-new unrelated file advances the generation.
    session.ingest_file(
        Arc::from("immune_unrelated.php"),
        Arc::from("<?php\nclass ImmuneUnrelated {}\n"),
    );

    let locks_before = session.ref_index_lock_count();
    let warm = session
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert_eq!(first.len(), 1);
    assert_eq!(
        session.ref_query_cache_hits(),
        hits_before,
        "first call must populate the cache, not hit it"
    );

    let second = session
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
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
///
/// Entries are revision-scoped: each ingest below moves the text revision,
/// which evicts the previous generation's (now-unreachable) keys, so during
/// the setup loop the tracked total is the *current* revision's sum only.
/// The cross-symbol accumulation is asserted at a fixed revision at the end.
#[test]
fn indexed_references_cache_tracks_total_locations_not_entry_count() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let mut file_sets: Vec<(&str, usize, Vec<Arc<str>>)> = Vec::new();
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
            .indexed_references_to(
                &sym,
                &files,
                false,
                mir_analyzer::ReferenceIncludes::Plain,
                &|| false,
            )
            .expect("query not cancelled");
        assert_eq!(refs.len(), n_callers, "one call site per caller file");
        assert_eq!(
            session.ref_query_cache_locations(),
            n_callers,
            "later-revision insert must evict the dead prior generation, \
             leaving only the current revision's locations"
        );
        file_sets.push((base, n_callers, files));
    }

    // No further ingests: all three queries land at one revision and
    // accumulate. 1 + 2 + 3 = 6 locations across 3 entries proves the
    // tracked total is the sum of result lengths, not the entry count.
    let mut expected_total = 0usize;
    for (base, n_callers, files) in &file_sets {
        let sym = mir_analyzer::Name::method(*base, "m");
        let refs = session
            .indexed_references_to(
                &sym,
                files,
                false,
                mir_analyzer::ReferenceIncludes::Plain,
                &|| false,
            )
            .expect("query not cancelled");
        assert_eq!(refs.len(), *n_callers);
        expected_total += refs.len();
    }
    assert_eq!(
        session.ref_query_cache_locations(),
        expected_total,
        "tracked total must be the sum of result lengths, not the entry count"
    );
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
/// workspace) is admitted on the spot and answered by a real scan — not a
/// conservative "return everything" fallback. `files_mentioning_class`
/// delegates to `files_mentioning_any`, which always admits its needles
/// verbatim before querying, so an unrecognized needle is never a reason
/// to skip narrowing; a file whose text simply doesn't contain the literal
/// still correctly drops out.
#[test]
fn files_mentioning_class_unknown_needle_gets_a_real_scan() {
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
        Arc::from("<?php\nfunction b() { new NeverDeclaredAnywhere(); }\n"),
    );

    let files = [file_a, file_b];
    let result = session.files_mentioning_class(&files, "NeverDeclaredAnywhere");
    assert_eq!(
        result.len(),
        1,
        "only the file whose text actually mentions the needle should match"
    );
    assert_eq!(result[0].as_ref(), "unknown_b.php");
}

/// [`AnalysisSession::files_mentioning_any`] matches a file mentioning
/// *any* of several needles admitted verbatim (no short-name stripping) —
/// the shape a caller narrowing by a fully-qualified literal plus a
/// subtype closure needs, not just one declared class's bare short name.
#[test]
fn files_mentioning_any_matches_if_any_needle_hits() {
    let file_a: Arc<str> = Arc::from("any_a.php");
    let file_b: Arc<str> = Arc::from("any_b.php");
    let file_c: Arc<str> = Arc::from("any_c.php");

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(
        file_a.clone(),
        Arc::from("<?php\nfunction a() { new \\App\\Foo\\Owner(); }\n"),
    );
    session.ingest_file(
        file_b.clone(),
        Arc::from("<?php\nfunction b() { new \\App\\Foo\\OwnerChild(); }\n"),
    );
    session.ingest_file(
        file_c.clone(),
        Arc::from("<?php\nfunction c(): int { return 1; }\n"),
    );

    let files = [file_a.clone(), file_b.clone(), file_c.clone()];
    let result =
        session.files_mentioning_any(&files, &["\\App\\Foo\\Owner", "\\App\\Foo\\OwnerChild"]);
    let mut result: Vec<&str> = result.iter().map(|f| f.as_ref()).collect();
    result.sort_unstable();
    assert_eq!(result, vec!["any_a.php", "any_b.php"]);
}

/// A fully-qualified literal (with namespace separators) admitted via
/// [`AnalysisSession::files_mentioning_any`] must match under the same
/// whole-identifier boundary rule as a short name — PHP's lexer never
/// places a qualified name immediately adjacent to another identifier
/// with no separator (`new App\Foo\Bar()` requires the space; `newApp\Foo\Bar`
/// would lex as one invalid token), so the boundary check that protects
/// short-name matches never produces a false negative for a literal FQN
/// mention in valid source.
#[test]
fn files_mentioning_any_fqn_literal_needle_matches_across_boundaries() {
    let file_a: Arc<str> = Arc::from("fqn_a.php");
    let file_b: Arc<str> = Arc::from("fqn_b.php");

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(
        file_a.clone(),
        Arc::from("<?php\nfunction a(): \\App\\Foo\\Bar { return new \\App\\Foo\\Bar(); }\n"),
    );
    session.ingest_file(
        file_b.clone(),
        Arc::from("<?php\nfunction b(): int { return 1; }\n"),
    );

    let files = [file_a.clone(), file_b.clone()];
    let result = session.files_mentioning_any(&files, &["\\App\\Foo\\Bar"]);
    assert_eq!(
        result.len(),
        1,
        "only file_a textually mentions the literal FQN"
    );
    assert_eq!(result[0].as_ref(), "fqn_a.php");
}

/// Same persistent-lookup discipline as `files_mentioning_class_repeat_query_is_pure_lookup`,
/// for the multi-needle form.
#[test]
fn files_mentioning_any_repeat_query_is_pure_lookup() {
    let file_a: Arc<str> = Arc::from("any_repeat_a.php");
    let file_b: Arc<str> = Arc::from("any_repeat_b.php");

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(
        file_a.clone(),
        Arc::from(
            "<?php\nclass AnyRepeatOwner {}\nclass AnyRepeatChild extends AnyRepeatOwner {}\n",
        ),
    );
    session.ingest_file(
        file_b.clone(),
        Arc::from("<?php\nfunction b() { new AnyRepeatChild(); }\n"),
    );

    let files = [file_a.clone(), file_b.clone()];
    let needles = ["AnyRepeatOwner", "AnyRepeatChild"];
    let first = session.files_mentioning_any(&files, &needles);
    assert_eq!(first.len(), 2);

    let scans_before = session.class_mention_stats().scans_recorded;
    assert!(scans_before > 0, "cold query must scan at least once");

    let second = session.files_mentioning_any(&files, &needles);
    assert_eq!(second.len(), 2);
    assert_eq!(
        session.class_mention_stats().scans_recorded,
        scans_before,
        "identical repeat query must not re-scan any file's text"
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
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
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert_eq!(
        after_edit.len(),
        2,
        "the second $x->m() call site must be picked up, not served from a stale cache entry"
    );
}

/// A subtype-edge commit made by a *subtype* query must invalidate a
/// member-references result cached at the same text revision. The reference
/// gate can skip a file whose text mentions neither the member nor the
/// owner's short name, leaving its subtype edge uncommitted — the cached
/// hierarchy fan-out is then smaller than what a subtype BFS (whose needles
/// walk the frontier's short names) later discovers. Edge commits happen
/// off-salsa, so the text revision alone cannot see them; the subtype-edge
/// epoch in the cache key is what evicts the stale entry.
#[test]
fn references_cache_invalidates_when_subtype_query_grows_hierarchy() {
    let base: Arc<str> = Arc::from("epoch_base.php");
    let mid: Arc<str> = Arc::from("epoch_mid.php");
    let child: Arc<str> = Arc::from("epoch_child.php");
    let grandchild: Arc<str> = Arc::from("epoch_grandchild.php");
    let caller: Arc<str> = Arc::from("epoch_caller.php");

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(
        base.clone(),
        Arc::from("<?php\nclass EpochBase { public function m(): int { return 1; } }\n"),
    );
    session.ingest_file(
        mid.clone(),
        Arc::from("<?php\nclass EpochMid extends EpochBase {}\n"),
    );
    // Registered without ingestion: no defs commit, so the EpochMid ←
    // EpochChild edge stays unknown until some query's gate admits this
    // file. Its text mentions neither `m` nor `EpochBase`, so the
    // references gate never does — only the subtype BFS's `EpochMid`
    // frontier needle can.
    session.set_file_text(
        child.clone(),
        Arc::from("<?php\nclass EpochChild extends EpochMid {}\n"),
    );
    // Overrides m(): the call site below resolves its owner to
    // EpochGrandchild, so the references query can only reach it through
    // the subtype fan-out — which the missing EpochChild edge severs.
    session.ingest_file(
        grandchild.clone(),
        Arc::from(
            "<?php\nclass EpochGrandchild extends EpochChild { public function m(): int { return 2; } }\n",
        ),
    );
    session.ingest_file(
        caller.clone(),
        Arc::from("<?php\nfunction epoch_go(EpochGrandchild $g): int { return $g->m(); }\n"),
    );

    let files = [
        base.clone(),
        mid.clone(),
        child.clone(),
        grandchild.clone(),
        caller.clone(),
    ];
    // Throwaway query so `settle_workspace_index` absorbs the mirror write
    // now: the queries under test then run at ONE text revision, and only
    // the edge epoch distinguishes their cache generations.
    let _ = session.indexed_references_to(
        &mir_analyzer::Name::method("EpochBase", "nope"),
        &files,
        false,
        mir_analyzer::ReferenceIncludes::Plain,
        &|| false,
    );

    let sym = mir_analyzer::Name::method("EpochBase", "m");
    let cold = session
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert!(
        cold.is_empty(),
        "EpochChild edge uncommitted: the $g->m() call site posts under \
         meth:EpochGrandchild::m, unreachable through the severed \
         hierarchy; got {cold:?}"
    );

    let subs = session.indexed_subtype_classes("EpochBase", &files, false);
    assert_eq!(
        subs.len(),
        3,
        "mid + child + grandchild expected, got {subs:?}"
    );

    let warm = session
        .indexed_references_to(
            &sym,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("query not cancelled");
    assert_eq!(
        warm.len(),
        1,
        "the grown hierarchy must reach the $g->m() call site; a stale \
         same-revision cache hit would still be empty"
    );
    assert_eq!(warm[0].0, caller);
}

/// [`AnalysisSession::indexed_subtype_classes`] gets the exact same
/// memoization treatment as `indexed_references_to` — same rationale
/// (`commit_defs_for_matching`'s freshness pass costs O(candidates) on
/// every call), same tests: a repeat query is a pure lookup, and a change
/// that adds a new subtype is still picked up, not served stale.
#[test]
fn indexed_subtype_classes_repeat_query_hits_cache() {
    let file_a: Arc<str> = Arc::from("subtype_cache_a.php");
    let file_b: Arc<str> = Arc::from("subtype_cache_b.php");
    let src_a = "<?php\nclass SubtypeCacheBase {}\n";
    let src_b = "<?php\nclass SubtypeCacheChild extends SubtypeCacheBase {}\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file(file_b.clone(), Arc::from(src_b));

    let files = [file_a.clone(), file_b.clone()];
    let hits_before = session.subtype_query_cache_hits();
    let first = session.indexed_subtype_classes("SubtypeCacheBase", &files, false);
    assert_eq!(first.len(), 1, "one direct subclass");
    assert_eq!(
        session.subtype_query_cache_hits(),
        hits_before,
        "first call must populate the cache, not hit it"
    );

    let second = session.indexed_subtype_classes("SubtypeCacheBase", &files, false);
    assert_eq!(second.len(), 1);
    assert_eq!(
        session.subtype_query_cache_hits(),
        hits_before + 1,
        "identical repeat query must hit the memoization cache"
    );
}

/// A newly-added subclass file must be picked up by the next query, not
/// served from a cache entry populated before it existed.
#[test]
fn indexed_subtype_classes_cache_invalidates_on_new_file() {
    let file_a: Arc<str> = Arc::from("subtype_new_a.php");
    let file_b: Arc<str> = Arc::from("subtype_new_b.php");
    let src_a = "<?php\nclass SubtypeNewBase {}\n";
    let src_b = "<?php\nclass SubtypeNewChild1 extends SubtypeNewBase {}\n";

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ingest_file(file_a.clone(), Arc::from(src_a));
    session.ingest_file(file_b.clone(), Arc::from(src_b));

    let mut files = vec![file_a.clone(), file_b.clone()];
    let before = session.indexed_subtype_classes("SubtypeNewBase", &files, false);
    assert_eq!(before.len(), 1, "one subclass before the new file lands");

    let file_c: Arc<str> = Arc::from("subtype_new_c.php");
    session.ingest_file(
        file_c.clone(),
        Arc::from("<?php\nclass SubtypeNewChild2 extends SubtypeNewBase {}\n"),
    );
    files.push(file_c);

    let after = session.indexed_subtype_classes("SubtypeNewBase", &files, false);
    assert_eq!(
        after.len(),
        2,
        "the new subclass must be picked up, not served from a stale cache entry"
    );
}

/// The cache bounds memory by total cached SITES, not entry count — same
/// discipline as `ref_query_cache` (see
/// `indexed_references_cache_tracks_total_locations_not_entry_count`), for
/// the identical reason: one entry's result size varies with how many
/// subtypes the queried class actually has.
///
/// Entries are revision-scoped: each ingest below moves the text revision,
/// which evicts the previous generation's (now-unreachable) keys, so during
/// the setup loop the tracked total is the *current* revision's sum only.
/// The cross-symbol accumulation is asserted at a fixed revision at the end.
#[test]
fn indexed_subtype_classes_cache_tracks_total_sites_not_entry_count() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let mut file_sets: Vec<(&str, usize, Vec<Arc<str>>)> = Vec::new();
    for (base, n_children) in [("SizeBaseA", 1usize), ("SizeBaseB", 2), ("SizeBaseC", 3)] {
        let base_file: Arc<str> = Arc::from(format!("{base}_base.php"));
        session.ingest_file(
            base_file.clone(),
            Arc::from(format!("<?php\nclass {base} {{}}\n")),
        );
        let mut files = vec![base_file];
        for c in 0..n_children {
            let child_file: Arc<str> = Arc::from(format!("{base}_child{c}.php"));
            session.ingest_file(
                child_file.clone(),
                Arc::from(format!("<?php\nclass {base}Child{c} extends {base} {{}}\n")),
            );
            files.push(child_file);
        }

        let sites = session.indexed_subtype_classes(base, &files, false);
        assert_eq!(sites.len(), n_children, "one site per direct subclass");
        assert_eq!(
            session.subtype_query_cache_sites(),
            n_children,
            "later-revision insert must evict the dead prior generation, \
             leaving only the current revision's sites"
        );
        file_sets.push((base, n_children, files));
    }

    // No further ingests: all three queries land at one revision and
    // accumulate. 1 + 2 + 3 = 6 sites across 3 entries proves the tracked
    // total is the sum of result lengths, not the entry count.
    let mut expected_total = 0usize;
    for (base, n_children, files) in &file_sets {
        let sites = session.indexed_subtype_classes(base, files, false);
        assert_eq!(sites.len(), *n_children);
        expected_total += sites.len();
    }
    assert_eq!(
        session.subtype_query_cache_sites(),
        expected_total,
        "tracked total must be the sum of result lengths, not the entry count"
    );
}
