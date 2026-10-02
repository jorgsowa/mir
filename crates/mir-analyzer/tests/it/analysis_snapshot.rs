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

const VENDOR_BASE_PATH: &str = "vendor/Base.php";
const VENDOR_BASE: &str =
    "<?php\nnamespace Vendor;\nclass Base { public function fromVendor(): int { return 1; } }\n";
const VENDOR_CHILD: &str = "<?php\nnamespace App;\nclass VendorChild extends \\Vendor\\Base {}\n";
const VENDOR_CALLER: &str =
    "<?php\nnamespace App;\nfunction use_vendor(\\Vendor\\Base $b): int { return $b->fromVendor(); }\n";

/// Maps every `Vendor\` class to [`VENDOR_BASE_PATH`], served from memory.
struct VendorSources;

impl mir_analyzer::ClassResolver for VendorSources {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        fqcn.starts_with("Vendor\\")
            .then(|| std::path::PathBuf::from(VENDOR_BASE_PATH))
    }
}

impl mir_analyzer::SourceProvider for VendorSources {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        (path == VENDOR_BASE_PATH).then(|| Arc::from(VENDOR_BASE))
    }
}

/// The resolver is attached after ingest: ingesting resolves a file's
/// structural dependencies, which would load the vendor file before any
/// snapshot could.
fn vendor_workspace() -> (AnalysisSession, Vec<Arc<str>>) {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    let files: Vec<Arc<str>> = ["child.php", "caller.php"]
        .into_iter()
        .map(Arc::from)
        .collect();
    for (path, text) in files.iter().zip([VENDOR_CHILD, VENDOR_CALLER]) {
        session.ingest_file(path.clone(), Arc::from(text));
    }
    let session = session
        .with_class_resolver(Arc::new(VendorSources))
        .with_source_provider(Arc::new(VendorSources));
    (session, files)
}

#[test]
fn snapshot_loads_an_unindexed_vendor_class_without_an_owner_write() {
    let (mut session, files) = vendor_workspace();
    let tracked_before = session.tracked_file_count();
    let revision_before = session.text_revision();
    let generation_before = session.index_generation();

    let snap = session.snapshot();
    let caller = files[1].clone();
    let (declaring, issues) = thread::spawn(move || {
        let (declaring, _) = snap
            .find_method_in_chain("App\\VendorChild", "fromVendor")
            .unwrap()
            .expect("inherited vendor method resolves on demand");
        let parsed = php_rs_parser::parse(VENDOR_CALLER);
        let analysis = snap
            .analyze(caller, VENDOR_CALLER, &parsed.program, &parsed.source_map)
            .unwrap();
        (declaring, analysis.issues)
    })
    .join()
    .unwrap();

    assert_eq!(declaring.as_ref(), "Vendor\\Base");
    assert!(
        !issues.iter().any(|i| matches!(
            i.kind,
            IssueKind::UndefinedClass { .. } | IssueKind::UndefinedMethod { .. }
        )),
        "{issues:?}"
    );
    assert_eq!(session.text_revision(), revision_before);
    assert_eq!(session.index_generation(), generation_before);
    assert_eq!(session.tracked_file_count(), tracked_before);

    // The owner's next settle adopts the file into its own registry.
    session.prepare_for_query(None);
    assert_eq!(session.tracked_file_count(), tracked_before + 1);
    assert!(session
        .all_classes()
        .iter()
        .any(|(fqcn, _)| fqcn.as_ref() == "Vendor\\Base"));
}

#[test]
fn concurrent_snapshots_loading_one_vendor_file_share_its_input() {
    let (mut session, _files) = vendor_workspace();
    let tracked_before = session.tracked_file_count();
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let snap = session.snapshot();
            thread::spawn(move || {
                snap.find_class_like("Vendor\\Base")
                    .unwrap()
                    .expect("vendor class resolves on demand")
                    .location()
                    .cloned()
                    .expect("vendor class has a location")
            })
        })
        .collect();
    let locations: Vec<_> = readers.into_iter().map(|r| r.join().unwrap()).collect();
    assert!(locations.windows(2).all(|w| w[0] == w[1]), "{locations:?}");

    session.prepare_for_query(None);
    assert_eq!(session.tracked_file_count(), tracked_before + 1);
}

#[test]
fn snapshot_loads_a_builtin_stub_on_demand() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let stubs_before = session.loaded_stub_count();
    let snap = session.snapshot();
    let found = thread::spawn(move || {
        (
            snap.find_class_like("ArrayObject").unwrap().is_some(),
            snap.find_function("str_contains").unwrap().is_some(),
        )
    })
    .join()
    .unwrap();
    assert_eq!(found, (true, true));
    assert_eq!(session.loaded_stub_count(), stubs_before);
}

const POLYFILL_PATH: &str = "vendor/polyfill/Stringable.php";
const POLYFILL: &str = "<?php\ninterface Stringable { public function __toString(): string; }\n";

fn on_snapshot<R: Send + 'static>(
    session: &AnalysisSession,
    query: impl FnOnce(mir_analyzer::AnalysisSnapshot) -> R + Send + 'static,
) -> R {
    let snap = session.snapshot();
    thread::spawn(move || query(snap)).join().unwrap()
}

#[test]
fn is_builtin_class_is_true_only_for_classes_declared_by_a_stub() {
    let (session, _files) = workspace();
    let found = on_snapshot(&session, |snap| {
        [
            "ArrayObject",
            "\\arrayobject",
            "App\\Base",
            "Missing\\Thing",
        ]
        .map(|fqcn| snap.is_builtin_class(fqcn).unwrap())
    });
    assert_eq!(found, [true, true, false, false]);
}

/// Vendor polyfills define stub-named classes; the workspace class wins, so
/// the name-only `stub_path_for_class` gate misclassifies them as built-in.
#[test]
fn workspace_class_shadowing_a_stub_name_is_not_builtin() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    assert!(on_snapshot(&session, |snap| snap
        .is_builtin_class("Stringable")
        .unwrap()));

    session.ingest_file(Arc::from(POLYFILL_PATH), Arc::from(POLYFILL));
    session.prepare_for_query(None);
    let (is_builtin, declared_in) = on_snapshot(&session, |snap| {
        (
            snap.is_builtin_class("Stringable").unwrap(),
            snap.find_class_like("Stringable")
                .unwrap()
                .and_then(|class| class.location().map(|loc| loc.file.clone())),
        )
    });
    assert!(mir_analyzer::stub_path_for_class("Stringable").is_some());
    assert_eq!(declared_in.as_deref(), Some(POLYFILL_PATH));
    assert!(!is_builtin);
}

/// Without the polyfill being indexed yet, a resolver-mapped workspace class
/// still wins over the stub on demand.
#[test]
fn resolver_mapped_class_shadowing_a_stub_name_is_not_builtin() {
    struct Polyfills;
    impl mir_analyzer::ClassResolver for Polyfills {
        fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
            (fqcn == "Stringable").then(|| std::path::PathBuf::from(POLYFILL_PATH))
        }
    }
    impl mir_analyzer::SourceProvider for Polyfills {
        fn read(&self, path: &str) -> Option<Arc<str>> {
            (path == POLYFILL_PATH).then(|| Arc::from(POLYFILL))
        }
    }
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(Arc::new(Polyfills))
        .with_source_provider(Arc::new(Polyfills));
    assert!(!on_snapshot(&session, |snap| snap
        .is_builtin_class("Stringable")
        .unwrap()));
}

fn issue_kinds(results: &[(Arc<str>, mir_analyzer::FileAnalysis)]) -> Vec<(Arc<str>, Vec<String>)> {
    results
        .iter()
        .map(|(file, analysis)| {
            (
                file.clone(),
                analysis
                    .issues
                    .iter()
                    .map(|i| i.kind.name().to_string())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn snapshot_reanalyze_files_matches_the_session_and_commits_references() {
    let (mut session, files) = workspace();
    session.prepare_for_query(None);
    for file in &files {
        session.prepare_file_for_analysis(file);
    }
    let snap = session.snapshot();
    let thread_files = files.clone();
    let from_snapshot = thread::spawn(move || {
        snap.reanalyze_files(&thread_files, &mir_analyzer::IndexCancel::new())
            .unwrap()
            .expect("not cancelled")
    })
    .join()
    .unwrap();

    let (mut other, other_files) = workspace();
    let from_session =
        other.reanalyze_files_cancellable(&other_files, &mir_analyzer::IndexCancel::new());

    assert_eq!(issue_kinds(&from_snapshot), issue_kinds(&from_session));
    assert!(
        from_snapshot
            .iter()
            .any(|(file, analysis)| file.as_ref() == "caller.php"
                && analysis
                    .issues
                    .iter()
                    .any(|i| i.kind.name() == "UndefinedFunction")),
        "{:?}",
        issue_kinds(&from_snapshot)
    );
    assert!(session
        .reference_locations("meth:App\\Base::run")
        .iter()
        .any(|(f, ..)| f.as_ref() == "caller.php"));
}

#[test]
fn snapshot_reanalyze_files_stops_when_cancelled_without_committing() {
    let (mut session, files) = workspace();
    session.prepare_for_query(None);
    let cancel = mir_analyzer::IndexCancel::new();
    cancel.cancel();
    let out = session.snapshot().reanalyze_files(&files, &cancel).unwrap();
    assert!(out.is_none());
    assert!(session
        .reference_locations("meth:App\\Base::run")
        .is_empty());
}

#[test]
fn snapshot_declaration_name_range_matches_the_session() {
    let (mut session, files) = workspace();
    session.prepare_for_query(Some(&files[0]));
    let snap = session.snapshot();
    let run = Name::method("App\\Base", "run");

    let from_snapshot = snap.declaration_name_range_cached(&run).unwrap();
    let from_session = session.declaration_name_range(&run);

    assert_eq!(from_snapshot, from_session);
    let (file, range) = from_snapshot.expect("declaration found");
    assert_eq!(file.as_ref(), "base.php");
    assert_eq!(range.start.line, 3);
}
