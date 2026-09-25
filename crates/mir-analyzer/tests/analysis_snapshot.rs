//! `AnalysisSnapshot` contracts: queries run off the owner's thread, match
//! the session's own answers, share commits and memos with the owner, and
//! unwind with `Cancelled` when the owner writes.

use std::sync::{mpsc, Arc};
use std::thread;

use mir_analyzer::{AnalysisSession, IssueKind, Name, PhpVersion, ReferenceIncludes};

const BASE: &str = "<?php\nnamespace App;\nclass Base { public function run(): void {} }\n";
const CHILD: &str = "<?php\nnamespace App;\nclass Child extends Base {}\n";
const CALLER: &str =
    "<?php\nnamespace App;\nfunction go(Base $b): void { $b->run(); missing(); }\n";

fn workspace() -> (AnalysisSession, Vec<Arc<str>>) {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    let files: Vec<Arc<str>> = ["base.php", "child.php", "caller.php"]
        .into_iter()
        .map(Arc::from)
        .collect();
    for (path, text) in files.iter().zip([BASE, CHILD, CALLER]) {
        session.ingest_file(path.clone(), Arc::from(text));
    }
    (session, files)
}

#[test]
fn snapshot_queries_on_another_thread_match_the_session() {
    let (mut session, files) = workspace();
    let run = Name::method("App\\Base", "run");

    session.prepare_for_query(Some(&files[2]));
    let snap = session.snapshot();
    let thread_files = files.clone();
    let thread_run = run.clone();
    let (refs, subs, name, class_found) = thread::spawn(move || {
        let refs = snap
            .indexed_references_to(&thread_run, &thread_files, true, ReferenceIncludes::Plain)
            .unwrap();
        let subs = snap
            .indexed_subtype_classes("App\\Base", &thread_files, false)
            .unwrap();
        let call_offset = CALLER.find("run()").unwrap() as u32;
        let name = snap.name_at(&thread_files[2], call_offset).unwrap();
        let class_found = snap.find_class_like("App\\Child").unwrap().is_some();
        (refs, subs, name, class_found)
    })
    .join()
    .unwrap();

    let hits_before = session.ref_query_cache_hits();
    let session_refs = session
        .indexed_references_to(&run, &files, true, ReferenceIncludes::Plain, &|| false)
        .unwrap();
    assert_eq!(refs, session_refs);
    assert_eq!(
        session.ref_query_cache_hits(),
        hits_before + 1,
        "the snapshot's memoized answer should serve the owner's repeat query"
    );
    assert_eq!(refs.len(), 2, "declaration + call site: {refs:?}");

    let sub_names: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert_eq!(sub_names, ["App\\Child"]);
    assert_eq!(name, Some(run));
    assert!(class_found);
}

#[test]
fn snapshot_analyze_commits_references_for_the_owner() {
    let (mut session, files) = workspace();
    let run_key = "meth:App\\Base::run";
    assert!(session.reference_locations(run_key).is_empty());
    session.prepare_for_query(Some(&files[2]));
    let snap = session.snapshot();
    let file = files[2].clone();
    let analysis = thread::spawn(move || {
        let parsed = php_rs_parser::parse(CALLER);
        snap.analyze(file, CALLER, &parsed.program, &parsed.source_map)
            .unwrap()
    })
    .join()
    .unwrap();

    assert!(
        analysis
            .issues
            .iter()
            .any(|i| matches!(i.kind, IssueKind::UndefinedFunction { .. })),
        "{:?}",
        analysis.issues
    );
    let locs = session.reference_locations(run_key);
    assert!(
        locs.iter().any(|(f, ..)| f.as_ref() == "caller.php"),
        "snapshot commit should land in the owner's reference index: {locs:?}"
    );
}

#[test]
fn snapshot_warm_files_commits_references_for_the_owner() {
    let (mut session, files) = workspace();
    let run_key = "meth:App\\Base::run";
    session.prepare_for_query(Some(&files[2]));
    let snap = session.snapshot();

    let cancelled = mir_analyzer::IndexCancel::new();
    cancelled.cancel();
    assert!(!snap.warm_files(&files, &cancelled).unwrap());
    assert!(session.reference_locations(run_key).is_empty());

    let thread_files = files.clone();
    let warmed = thread::spawn(move || {
        snap.warm_files(&thread_files, &mir_analyzer::IndexCancel::new())
            .unwrap()
    })
    .join()
    .unwrap();
    assert!(warmed);
    let locs = session.reference_locations(run_key);
    assert!(
        locs.iter().any(|(f, ..)| f.as_ref() == "caller.php"),
        "warm commit should land in the owner's reference index: {locs:?}"
    );
}

#[test]
fn snapshot_class_and_collector_issues_match_the_session() {
    let (mut session, files) = workspace();
    session.prepare_for_query(None);
    let snap = session.snapshot();
    let thread_files = files.clone();
    let (class_issues, collector_issues) = thread::spawn(move || {
        (
            snap.class_issues(&thread_files).unwrap(),
            snap.collector_issues(&thread_files).unwrap(),
        )
    })
    .join()
    .unwrap();
    assert_eq!(class_issues.len(), session.class_issues(&files).len());
    assert_eq!(
        collector_issues.len(),
        session.collector_issues(&files).len()
    );
}

#[test]
fn owner_write_cancels_an_in_flight_snapshot_query() {
    let (mut session, files) = workspace();
    let snap = session.snapshot();
    let (started_tx, started_rx) = mpsc::channel();
    let thread_files = files.clone();
    let reader = thread::spawn(move || loop {
        match snap.collector_issues(&thread_files) {
            Ok(_) => {
                let _ = started_tx.send(());
            }
            Err(cancelled) => return cancelled,
        }
    });
    started_rx.recv().unwrap();

    // Blocks until the reader observes cancellation and drops its snapshot.
    session.ingest_file(
        files[0].clone(),
        Arc::from(
            "<?php\nnamespace App;\nclass Base { public function run(): int { return 1; } }\n",
        ),
    );
    let _cancelled: salsa::Cancelled = reader.join().unwrap();

    let decl = session
        .definition_of_cached(&Name::method("App\\Base", "run"))
        .unwrap();
    assert_eq!(decl.file.as_ref(), "base.php");
}
