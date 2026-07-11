//! WS3 write-path guarantees:
//!
//! 1. `ingest_file_prepared` runs the Phase-1 warm-up at write time — the
//!    file's direct class references are lazy-loaded when the text lands,
//!    not serially at the front of the next references / re-analysis read.
//! 2. A session built `without_reference_index()` never takes the `RefIndex`
//!    lock on any edit or read path (counter-asserted; a control session
//!    keeps the counter honest).

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
fn no_ref_index_locks_on_edit_or_read_paths_when_opted_out() {
    let file_a: Arc<str> = Arc::from("hoist_a.php");
    let file_b: Arc<str> = Arc::from("hoist_b.php");
    let src_a = "<?php\nclass HoistBase { public function m(): int { return 1; } }\n";
    let src_b = "<?php\nclass HoistDep extends HoistBase {}\nfunction hb(): int { $x = new HoistBase(); return $x->m(); }\n";

    let run_flows = |session: &AnalysisSession| {
        session.ensure_all_stubs();
        // Edit path: ingest (twice — re-ingest exercises the definition
        // removal branch) + prepared variant.
        session.ingest_file(file_a.clone(), Arc::from(src_a));
        session.ingest_file(file_a.clone(), Arc::from(src_a));
        session.ingest_file_prepared(file_b.clone(), Arc::from(src_b));
        // Analysis/read path: FileAnalyzer (the open-file hover/diagnostics
        // flow) commits pending ref locs at the end — must be gated.
        let parsed = php_rs_parser::parse(src_a);
        let _ = FileAnalyzer::new(session).analyze(
            file_a.clone(),
            src_a,
            &parsed.program,
            &parsed.source_map,
        );
        // Pure references read.
        let refs = session.references_to_in_files(
            &mir_analyzer::Name::method("HoistBase", "m"),
            &[file_a.clone(), file_b.clone()],
        );
        assert!(!refs.is_empty(), "pure references path must still work");
        // Edit sweep (the per-keystroke republish path).
        let _ =
            session.reanalyze_files_cancellable(std::slice::from_ref(&file_b), &IndexCancel::new());
        // Close path.
        session.invalidate_file(file_b.as_ref());
    };

    // Opted-out session: zero RefIndex locks across every flow above. The
    // cache dir matters: it attaches an AnalysisCache, whose reverse-dep
    // upkeep inside ingest_file read the index's forward view until gated.
    let cache_dir = create_temp_dir("hoist_locks");
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_cache_dir(cache_dir.path())
        .without_reference_index();
    run_flows(&session);
    assert_eq!(
        session.ref_index_lock_count(),
        0,
        "RefIndex was locked on an edit/read path despite without_reference_index()"
    );

    // Control: the default session locks it — proves the counter counts.
    let control = AnalysisSession::new(PhpVersion::LATEST);
    run_flows(&control);
    assert!(
        control.ref_index_lock_count() > 0,
        "control session should lock RefIndex; the counter is vacuous"
    );
}
