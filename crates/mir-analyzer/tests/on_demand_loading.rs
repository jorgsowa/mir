//! Files a snapshot loads on demand answer every lookup the way the owner
//! does once it has indexed them, and the owner adopts them without losing
//! them from the symbol index, the dependency graph or reference queries.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use mir_analyzer::{AnalysisSession, Issue, LoadOutcome, Name, PhpVersion, ReferenceIncludes};

/// Resolves exactly the listed class names to paths and serves their text
/// from memory.
#[derive(Clone)]
struct Sources {
    classes: HashMap<&'static str, &'static str>,
    files: HashMap<&'static str, &'static str>,
}

impl Sources {
    fn new(
        classes: &[(&'static str, &'static str)],
        files: &[(&'static str, &'static str)],
    ) -> Self {
        Self {
            classes: classes.iter().copied().collect(),
            files: files.iter().copied().collect(),
        }
    }
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

fn session_with(sources: Sources, files: &[(&str, &str)]) -> AnalysisSession {
    let sources = Arc::new(sources);
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(sources.clone())
        .with_source_provider(sources);
    for (path, text) in files {
        session.ingest_file(Arc::from(*path), Arc::from(*text));
    }
    session
}

fn on_snapshot<R: Send + 'static>(
    session: &AnalysisSession,
    query: impl FnOnce(mir_analyzer::AnalysisSnapshot) -> R + Send + 'static,
) -> R {
    let snap = session.snapshot();
    thread::spawn(move || query(snap)).join().unwrap()
}

fn declaring_file(session: &AnalysisSession, fqcn: &'static str) -> Arc<str> {
    on_snapshot(session, move |snap| {
        snap.find_class_like(fqcn)
            .unwrap()
            .unwrap_or_else(|| panic!("{fqcn} resolves"))
            .location()
            .expect("declared in a file")
            .file
            .clone()
    })
}

const VENDOR_BASE_PATH: &str = "vendor/acme/Base.php";
const VENDOR_BASE: &str =
    "<?php\nnamespace Vendor;\nclass Base {}\nclass Other { public function run(): void {} }\n";

fn vendor_base() -> Sources {
    Sources::new(
        &[("Vendor\\Base", VENDOR_BASE_PATH)],
        &[(VENDOR_BASE_PATH, VENDOR_BASE)],
    )
}

#[test]
fn rebuilding_the_index_keeps_a_file_loaded_on_demand_before_it_existed() {
    let mut session = session_with(
        vendor_base(),
        &[(
            "child.php",
            "<?php\nnamespace App;\nclass Child extends \\Vendor\\Base {}\n",
        )],
    );
    assert!(!session.workspace_symbol_index_ready());
    declaring_file(&session, "Vendor\\Base");

    session.rebuild_workspace_symbol_index();
    session.prepare_for_query(None);

    assert_eq!(
        session.load_class("Vendor\\Base"),
        LoadOutcome::AlreadyLoaded
    );
    assert_eq!(
        session.load_class("Vendor\\Other"),
        LoadOutcome::AlreadyLoaded
    );
}

#[test]
fn a_resolver_file_outranks_the_native_stub_as_in_the_index() {
    const POLYFILL: &str = "vendor/polyfill/Stringable.php";
    let mut session = session_with(
        Sources::new(
            &[("Stringable", POLYFILL)],
            &[(
                POLYFILL,
                "<?php\ninterface Stringable { public function __toString(): string; }\n",
            )],
        ),
        &[],
    );

    let on_demand = declaring_file(&session, "Stringable");
    assert_eq!(on_demand.as_ref(), POLYFILL);

    session.prepare_for_query(None);
    session.ensure_all_stubs();
    session.prepare_for_query(None);
    assert_eq!(session.load_class("Stringable"), LoadOutcome::AlreadyLoaded);
    assert_eq!(declaring_file(&session, "Stringable"), on_demand);
}

#[test]
fn a_vendor_file_loaded_on_demand_is_registered_as_durable() {
    let mut session = session_with(vendor_base(), &[]);
    declaring_file(&session, "Vendor\\Base");
    session.prepare_for_query(None);

    // Re-registering it as a vendor file has nothing to raise.
    let revision = session.text_revision();
    session.set_vendor_files([(Arc::from(VENDOR_BASE_PATH), Arc::from(VENDOR_BASE))]);
    assert_eq!(session.text_revision(), revision);
}

#[test]
fn registering_an_on_demand_project_file_as_vendor_raises_its_durability() {
    const LIB_PATH: &str = "lib/Base.php";
    const LIB: &str = "<?php\nnamespace Lib;\nclass Base {}\n";
    let mut session = session_with(
        Sources::new(&[("Lib\\Base", LIB_PATH)], &[(LIB_PATH, LIB)]),
        &[],
    );
    declaring_file(&session, "Lib\\Base");
    session.prepare_for_query(None);

    let revision = session.text_revision();
    session.set_vendor_files([(Arc::from(LIB_PATH), Arc::from(LIB))]);
    let raised = session.text_revision();
    assert_ne!(raised, revision, "the durability raise is an input write");

    session.set_vendor_files([(Arc::from(LIB_PATH), Arc::from(LIB))]);
    assert_eq!(session.text_revision(), raised);
}

fn issue_messages(issues: &[Issue]) -> Vec<String> {
    let mut messages: Vec<String> = issues.iter().map(|i| i.kind.message()).collect();
    messages.sort();
    messages
}

#[test]
fn inference_over_an_on_demand_file_matches_the_owner() {
    const MAKER_PATH: &str = "vendor/acme/Maker.php";
    const THING_PATH: &str = "vendor/acme/Sub/Thing.php";
    const CALLER: &str = "<?php\nnamespace App;\nfunction use_maker(\\Vendor\\Maker $m): int { return $m->make(); }\n";
    let sources = Sources::new(
        &[
            ("Vendor\\Maker", MAKER_PATH),
            ("Vendor\\Sub\\Thing", THING_PATH),
        ],
        &[
            (
                MAKER_PATH,
                "<?php\nnamespace Vendor;\nuse Vendor\\Sub\\Thing;\nclass Maker { public function make() { return new Thing(); } }\n",
            ),
            (
                THING_PATH,
                "<?php\nnamespace Vendor\\Sub;\nclass Thing {}\n",
            ),
        ],
    );
    let session = session_with(sources.clone(), &[("caller.php", CALLER)]);

    let on_demand = on_snapshot(&session, |snap| {
        let parsed = php_rs_parser::parse(CALLER);
        snap.analyze(
            Arc::from("caller.php"),
            CALLER,
            &parsed.program,
            &parsed.source_map,
        )
        .unwrap()
        .issues
    });
    let on_demand = issue_messages(&on_demand);
    assert!(
        on_demand.iter().any(|m| m.contains("Vendor\\Sub\\Thing")),
        "the inferred return type resolves through the vendor file's imports: {on_demand:?}"
    );

    let mut owner = session_with(sources, &[]);
    owner.load_class("Vendor\\Maker");
    owner.load_class("Vendor\\Sub\\Thing");
    let indexed = owner.analyze_file_diagnostics("caller.php", CALLER).issues;
    assert_eq!(on_demand, issue_messages(&indexed));
}

#[test]
fn the_dependency_graph_includes_files_loaded_on_demand() {
    let mut session = session_with(
        vendor_base(),
        &[(
            "child.php",
            "<?php\nnamespace App;\nclass Child extends \\Vendor\\Base {}\n",
        )],
    );
    let dependencies = |session: &AnalysisSession| {
        session
            .dependency_graph()
            .dependency_paths_of("child.php")
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
    };

    assert_eq!(dependencies(&session), [VENDOR_BASE_PATH]);
    session.prepare_for_query(None);
    assert_eq!(dependencies(&session), [VENDOR_BASE_PATH]);
}

#[test]
fn owner_reference_query_sees_references_a_pending_adoption_resolves() {
    const CALLER: &str =
        "<?php\nnamespace App;\nfunction go(\\Vendor\\Other $o): void { $o->run(); }\n";
    let mut session = session_with(vendor_base(), &[("caller.php", CALLER)]);
    let run = Name::method("Vendor\\Other", "run");
    let files: Vec<Arc<str>> = vec![Arc::from("caller.php")];

    // `Vendor\Other` is only declared alongside `Vendor\Base`, so nothing
    // resolves it until that file is loaded and indexed.
    let (query, scope) = (run.clone(), files.clone());
    let before = on_snapshot(&session, move |snap| {
        snap.indexed_references_to(&query, &scope, false, ReferenceIncludes::Plain)
            .unwrap()
    });
    assert!(before.is_empty(), "{before:?}");
    declaring_file(&session, "Vendor\\Base");

    let refs = session
        .indexed_references_to(&run, &files, false, ReferenceIncludes::Plain, &|| false)
        .unwrap();
    assert_eq!(refs.len(), 1, "{refs:?}");
}

#[test]
fn attaching_a_resolver_after_a_lookup_retries_its_miss() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    assert!(!session.contains_class("Vendor\\Base"));

    let sources = Arc::new(vendor_base());
    let session = session
        .with_class_resolver(sources.clone())
        .with_source_provider(sources);
    assert!(session.contains_class("Vendor\\Base"));
}

#[test]
fn swapping_the_source_provider_retries_a_miss() {
    let resolver_only = Sources::new(&[("Vendor\\Base", VENDOR_BASE_PATH)], &[]);
    let session = session_with(resolver_only, &[]);
    assert!(!session.contains_class("Vendor\\Base"));

    let session = session.with_source_provider(Arc::new(vendor_base()));
    assert!(session.contains_class("Vendor\\Base"));
}
