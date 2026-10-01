//! A host that answers find-references from `AnalysisSnapshot` alone (after
//! `prepare_for_query`) must get what `AnalysisSession::indexed_references_to`
//! returns, whose owner-side warm-up the snapshot form does not run.

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::thread;

use mir_analyzer::{AnalysisSession, Name, PhpVersion, Range, ReferenceIncludes};

use crate::common::create_temp_dir;

type Refs = Vec<(Arc<str>, Range)>;

#[derive(Clone)]
struct Sources {
    classes: HashMap<&'static str, &'static str>,
    files: HashMap<&'static str, &'static str>,
}

impl mir_analyzer::ClassResolver for Sources {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        self.classes.get(fqcn).map(std::path::PathBuf::from)
    }
}

impl mir_analyzer::SourceProvider for Sources {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        self.files.get(path).map(|text| Arc::from(*text))
    }
}

const VENDOR_PATH: &str = "vendor/acme/Base.php";
const VENDOR: &str = "<?php\nnamespace Vendor;\nclass Base { public function run(): void {} }\n";
const CALLER: &str = "<?php\nnamespace App;\nfunction go(\\Vendor\\Base $b): void { $b->run(); }\n";

fn vendor_sources() -> Arc<Sources> {
    Arc::new(Sources {
        classes: [("Vendor\\Base", VENDOR_PATH)].into_iter().collect(),
        files: [(VENDOR_PATH, VENDOR)].into_iter().collect(),
    })
}

fn vendor_session() -> AnalysisSession {
    let sources = vendor_sources();
    AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(sources.clone())
        .with_source_provider(sources)
}

fn files(paths: &[&str]) -> Vec<Arc<str>> {
    paths.iter().copied().map(Arc::from).collect()
}

fn owner_refs(
    session: &mut AnalysisSession,
    symbol: &Name,
    files: &[Arc<str>],
    include_declaration: bool,
) -> Refs {
    session
        .indexed_references_to(
            symbol,
            files,
            include_declaration,
            ReferenceIncludes::Plain,
            &|| false,
        )
        .unwrap()
}

/// The host's read path: owner-locked prelude, then a snapshot query on
/// another thread. `prelude` adds [`AnalysisSession::prepare_references_query`]
/// to the `prepare_for_query` every snapshot needs.
fn snapshot_refs(
    session: &mut AnalysisSession,
    symbol: &Name,
    files: &[Arc<str>],
    include_declaration: bool,
    prelude: bool,
) -> Refs {
    session.prepare_for_query(None);
    if prelude {
        let stale = session
            .snapshot()
            .stale_reference_candidates(symbol, files)
            .unwrap();
        assert!(session.prepare_references_query(symbol, &stale, include_declaration, &|| false));
    }
    let snap = session.snapshot();
    let (symbol, files) = (symbol.clone(), files.to_vec());
    thread::spawn(move || {
        snap.indexed_references_to(
            &symbol,
            &files,
            include_declaration,
            ReferenceIncludes::Plain,
        )
        .unwrap()
    })
    .join()
    .unwrap()
}

/// The owner's answer, the bare snapshot's, and the snapshot's after the
/// prelude — each over its own identically built session, so none sees
/// another's commits.
fn all_forms(
    build: impl Fn() -> AnalysisSession,
    symbol: &Name,
    files: &[Arc<str>],
    include_declaration: bool,
) -> (Refs, Refs, Refs) {
    (
        owner_refs(&mut build(), symbol, files, include_declaration),
        snapshot_refs(&mut build(), symbol, files, include_declaration, false),
        snapshot_refs(&mut build(), symbol, files, include_declaration, true),
    )
}

/// Class loading needs no owner warm-up: the snapshot loads them on demand.
fn assert_classes_need_no_prelude(forms: (Refs, Refs, Refs), expected: usize) {
    let (owner, bare, prepared) = forms;
    assert_eq!(owner.len(), expected, "{owner:?}");
    assert_eq!(bare, owner);
    assert_eq!(prepared, owner);
}

#[test]
fn cold_candidate_calling_an_unloaded_vendor_method() {
    let build = || {
        let mut session = vendor_session();
        session.ingest_file(Arc::from("caller.php"), Arc::from(CALLER));
        session
    };
    let files = files(&["caller.php", VENDOR_PATH]);
    let symbol = Name::method("Vendor\\Base", "run");
    assert_classes_need_no_prelude(all_forms(build, &symbol, &files, true), 2);
}

#[test]
fn just_edited_candidate_gaining_an_unloaded_vendor_call() {
    let build = || {
        let mut session = vendor_session();
        session.ingest_file(
            Arc::from("caller.php"),
            Arc::from("<?php\nnamespace App;\nfunction go(): void {}\n"),
        );
        let files = files(&["caller.php"]);
        let symbol = Name::method("Vendor\\Base", "run");
        // Commit the pre-edit text as fresh and resolved.
        owner_refs(&mut session, &symbol, &files, false);
        session.ingest_file(Arc::from("caller.php"), Arc::from(CALLER));
        session
    };
    let files = files(&["caller.php"]);
    let symbol = Name::method("Vendor\\Base", "run");
    assert_classes_need_no_prelude(all_forms(build, &symbol, &files, false), 1);
}

#[test]
fn candidate_extending_an_unloaded_vendor_class_for_a_hierarchy_query() {
    const CHILD: &str = "<?php\nnamespace App;\nclass Child extends \\Vendor\\Base {}\n";
    const USES_CHILD: &str = "<?php\nnamespace App;\nfunction go(Child $c): void { $c->run(); }\n";
    let build = || {
        let mut session = vendor_session();
        session.ingest_file(Arc::from("child.php"), Arc::from(CHILD));
        session.ingest_file(Arc::from("uses.php"), Arc::from(USES_CHILD));
        session
    };
    let files = files(&["child.php", "uses.php", VENDOR_PATH]);
    let symbol = Name::method("Vendor\\Base", "run");
    assert_classes_need_no_prelude(all_forms(build, &symbol, &files, true), 2);
}

/// A project with one Composer `autoload.files` function, `vendor_helper`.
/// These have no name → path mapping, so only the owner's warm-up
/// (`ensure_vendor_eager_functions`) can index them.
fn autoload_files_session(root: &std::path::Path) -> AnalysisSession {
    let vendor = root.join("vendor");
    fs::create_dir_all(vendor.join("composer")).unwrap();
    fs::create_dir_all(vendor.join("helpers")).unwrap();
    fs::write(
        vendor.join("composer/autoload_files.php"),
        "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'abc123' => $vendorDir . '/helpers/functions.php',\n);\n",
    )
    .unwrap();
    for empty in ["psr4", "classmap", "namespaces"] {
        fs::write(
            vendor.join(format!("composer/autoload_{empty}.php")),
            "<?php\nreturn [];\n",
        )
        .unwrap();
    }
    fs::write(
        vendor.join("helpers/functions.php"),
        "<?php\nfunction vendor_helper(string $s): string { return $s; }\n",
    )
    .unwrap();
    fs::write(
        root.join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).unwrap();
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));
    session.ingest_file(
        Arc::from("consumer.php"),
        Arc::from("<?php\nvendor_helper('x');\n"),
    );
    session
}

#[test]
fn prelude_makes_a_snapshot_see_a_vendor_autoload_files_function() {
    let root = create_temp_dir("snapshot_prelude_autoload");
    let files = files(&["consumer.php"]);
    let symbol = Name::function("vendor_helper");
    let (owner, _, prepared) = all_forms(
        || autoload_files_session(root.path()),
        &symbol,
        &files,
        false,
    );
    assert_eq!(owner.len(), 1, "{owner:?}");
    assert_eq!(prepared, owner);
}

/// Without the prelude the candidate is analyzed before the eager files are
/// indexed, so its call is unresolved and not recorded. The commit stays tied
/// to the old generation, so the next query after the owner's warm-up is right.
#[test]
fn snapshot_alone_misses_an_autoload_files_call_until_the_owner_prepares() {
    let root = create_temp_dir("snapshot_prelude_gap");
    let files = files(&["consumer.php"]);
    let symbol = Name::function("vendor_helper");
    let mut session = autoload_files_session(root.path());

    assert!(snapshot_refs(&mut session, &symbol, &files, false, false).is_empty());

    session.prepare_for_query(Some(&files[0]));
    let healed = snapshot_refs(&mut session, &symbol, &files, false, false);
    assert_eq!(healed.len(), 1, "{healed:?}");
}

/// Only stale candidates are prepared: a fresh workspace pays no warm-up.
#[test]
fn prelude_is_a_no_op_when_every_candidate_is_fresh() {
    let mut session = vendor_session();
    session.ingest_file(Arc::from("caller.php"), Arc::from(CALLER));
    let files = files(&["caller.php", VENDOR_PATH]);
    let symbol = Name::method("Vendor\\Base", "run");
    owner_refs(&mut session, &symbol, &files, true);

    let (revision, generation) = (session.text_revision(), session.index_generation());
    let stale = session
        .snapshot()
        .stale_reference_candidates(&symbol, &files)
        .unwrap();
    assert!(stale.is_empty());
    assert!(session.prepare_references_query(&symbol, &stale, true, &|| false));
    assert_eq!(session.text_revision(), revision);
    assert_eq!(session.index_generation(), generation);
}

#[test]
fn prelude_honours_cancellation() {
    let mut session = vendor_session();
    session.ingest_file(Arc::from("caller.php"), Arc::from(CALLER));
    let files = files(&["caller.php"]);
    let symbol = Name::method("Vendor\\Base", "run");
    assert!(!session.prepare_references_query(&symbol, &files, false, &|| true));
}

/// A host checks staleness off its lock and takes it only when the answer is
/// non-empty.
#[test]
fn stale_check_is_non_empty_until_the_candidates_are_committed() {
    let mut session = vendor_session();
    session.ingest_file(Arc::from("caller.php"), Arc::from(CALLER));
    let files = files(&["caller.php"]);
    let symbol = Name::method("Vendor\\Base", "run");
    let stale = |session: &AnalysisSession| {
        session
            .snapshot()
            .stale_reference_candidates(&symbol, &files)
            .unwrap()
    };

    assert_eq!(stale(&session), files);
    owner_refs(&mut session, &symbol, &files, false);
    assert!(stale(&session).is_empty());
}
