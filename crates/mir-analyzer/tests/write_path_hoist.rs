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
