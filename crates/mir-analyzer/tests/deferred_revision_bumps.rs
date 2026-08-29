// Pins the deferred-bump batching on the LSP warm-up paths: a pass that
// lazy-loads N classes must advance the workspace generation once (one salsa
// input write, one reader-cancellation window), not once per class.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

use self::common::create_temp_dir;

const LIB_CLASSES: usize = 10;

/// PSR-4 workspace mirroring the LSP-host shape: only the consumer file is
/// registered up front; the `LIB_CLASSES` classes it references live on disk
/// only (vendor-style) and must be resolver-loaded — each load registers a
/// NEW SourceFile input, the path that bumps the workspace revision.
/// Returns (session, registered query-scope paths).
fn workspace_with_lazy_classes() -> (AnalysisSession, Vec<Arc<str>>) {
    let root = create_temp_dir("deferred-bumps");
    fs::create_dir_all(root.path().join("src")).unwrap();

    for i in 0..LIB_CLASSES {
        let path = root.path().join(format!("src/Lib{i}.php"));
        let src =
            format!("<?php\nnamespace App;\nclass Lib{i} {{ public function m(): void {{}} }}\n");
        fs::write(&path, &src).unwrap();
    }
    let uses: String = (0..LIB_CLASSES)
        .map(|i| format!("$v{i} = new \\App\\Lib{i}();\n"))
        .collect();
    let main = root.path().join("main.php");
    let main_src = format!("<?php\n{uses}");
    fs::write(&main, &main_src).unwrap();

    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    let psr4 =
        Arc::new(mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map"));

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let main_path: Arc<str> = Arc::from(main.to_string_lossy().as_ref());
    session.set_workspace_files(vec![(main_path.clone(), Arc::from(main_src.as_str()))]);
    // The batching only engages while the symbol-index singleton exists
    // (without one, class lookups fall back to the revision-keyed tracked
    // walk, which must stay fresh per load) — build it, as an LSP host does
    // at the end of its scan.
    session.rebuild_workspace_symbol_index();
    // Keep the temp dir alive for the session's resolver reads.
    std::mem::forget(root);
    (session, vec![main_path])
}

#[test]
fn cold_references_query_bumps_generation_once_for_all_lazy_loads() {
    let (session, paths) = workspace_with_lazy_classes();
    let target = Name::class("App\\Lib0");

    let gen_before = session.index_generation();
    let refs = session
        .indexed_references_to(
            &target,
            &paths,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    let bumps = session.index_generation() - gen_before;

    assert!(
        !refs.is_empty(),
        "the `new \\App\\Lib0()` site must be found"
    );
    // Phase 1 lazily loads all LIB_CLASSES classes referenced by main.php;
    // batched, that is one flush — not one bump per loaded class.
    assert!(
        bumps <= 2,
        "expected the warm-up's lazy loads to coalesce into one revision bump, got {bumps}"
    );
    // The loads themselves must still have landed.
    for i in 0..LIB_CLASSES {
        assert!(
            session.contains_class(&format!("App\\Lib{i}")),
            "App\\Lib{i} must be loaded after the query's warm-up"
        );
    }

    // Warm repeat: fully committed, no loads, no bumps, same result.
    let gen_before = session.index_generation();
    let warm = session
        .indexed_references_to(
            &target,
            &paths,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(warm.len(), refs.len());
    assert_eq!(
        session.index_generation(),
        gen_before,
        "a warm repeat must not bump the generation"
    );
}

#[test]
fn bulk_registration_bumps_generation_once_per_batch() {
    let (session, _paths) = workspace_with_lazy_classes();

    let batch: Vec<(Arc<str>, Arc<str>)> = (0..20)
        .map(|i| {
            (
                Arc::from(format!("/virtual/extra{i}.php").as_str()),
                Arc::from(format!("<?php\nclass Extra{i} {{}}\n").as_str()),
            )
        })
        .collect();

    let gen_before = session.index_generation();
    session.set_workspace_files(batch);
    let bumps = session.index_generation() - gen_before;
    assert!(
        bumps <= 1,
        "registering a batch of new files must bump the generation once, got {bumps}"
    );
    assert!(
        bumps >= 1,
        "adding files must still advance the generation for host progress tracking"
    );
}

#[test]
fn prefetch_imports_batches_generation_bumps() {
    let root = create_temp_dir("prefetch-deferred-bumps");
    fs::create_dir_all(root.path().join("src")).unwrap();

    for i in 0..LIB_CLASSES {
        let path = root.path().join(format!("src/Dep{i}.php"));
        let src =
            format!("<?php\nnamespace App;\nclass Dep{i} {{ public function m(): void {{}} }}\n");
        fs::write(&path, &src).unwrap();
    }

    let uses: String = (0..LIB_CLASSES)
        .map(|i| format!("use App\\Dep{i};\n"))
        .collect();
    let params: String = (0..LIB_CLASSES)
        .map(|i| format!("Dep{i} $d{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let body: String = (0..LIB_CLASSES).map(|i| format!("$d{i}->m();\n")).collect();
    let opened_src = format!(
        "<?php\n{uses}class Caller {{ public function go({params}): void {{\n{body}}} }}\n"
    );

    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    let psr4 =
        Arc::new(mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map"));

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    let opened: Arc<str> = Arc::from("opened.php");
    session.ingest_file(opened.clone(), Arc::from(opened_src.as_str()));
    session.rebuild_workspace_symbol_index();

    let gen_before = session.index_generation();
    let loaded = session.prefetch_imports(opened.as_ref());
    let bumps = session.index_generation() - gen_before;

    assert_eq!(
        loaded, LIB_CLASSES,
        "prefetch must load every unresolved imported class"
    );
    assert!(
        bumps <= 2,
        "expected prefetch_imports to coalesce lazy loads into one revision bump, got {bumps}"
    );
    for i in 0..LIB_CLASSES {
        assert!(
            session.contains_class(&format!("App\\Dep{i}")),
            "App\\Dep{i} must be loaded after prefetch"
        );
    }
}
