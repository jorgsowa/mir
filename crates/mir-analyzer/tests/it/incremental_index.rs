//! Tests for incremental workspace-symbol-index maintenance and the warm-cache
//! (no-churn) guarantee that the eager-static-input model depends on.

use std::fs;
use std::sync::Arc;

use mir_analyzer::db::{analyze_file, MirDatabase, Work};
use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

use crate::common::create_temp_dir;

fn make_session(root: &std::path::Path) -> AnalysisSession {
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4 map");
    AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4))
}

fn write_composer(root: &std::path::Path) {
    fs::write(
        root.join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/","Vendor\\":"vendor/VendorLib/src/"}}}"#,
    )
    .unwrap();
}

/// Every PHP file to index for these fixtures (project + vendor union — the
/// fixtures declare `Vendor\` directly in composer autoload, so it lands in
/// project entries without a generated `installed.json`).
fn indexable_files(root: &std::path::Path) -> Vec<(Arc<str>, Arc<str>)> {
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4 map");
    let mut paths = psr4.project_files();
    paths.extend(psr4.all_vendor_files());
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter_map(|p| {
            let t = fs::read_to_string(&p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(t.as_str()),
            ))
        })
        .collect()
}

// ─── incremental merge == full rebuild ────────────────────────────────────────

/// Indexing vendor files in bounded chunks (out of order) via `index_batch`
/// must produce the same resolvable class set as a single full rebuild
/// (`finalize_index`), and every class must resolve through the incrementally
/// merged singleton even before finalize.
#[test]
fn incremental_index_matches_full_rebuild() {
    let root = create_temp_dir("incr_matches_full");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    fs::create_dir_all(&vendor_src).unwrap();
    write_composer(root.path());

    for i in 0..20usize {
        fs::write(
            vendor_src.join(format!("C{i}.php")),
            format!("<?php\nnamespace Vendor;\nclass C{i} {{ public function m(): void {{}} }}\n"),
        )
        .unwrap();
    }

    let mut files = indexable_files(root.path());
    // Reverse so chunks arrive "out of order" relative to declaration order.
    files.reverse();

    // Incremental: chunks of 3, no finalize.
    let mut inc = make_session(root.path());
    let cancel = IndexCancel::new();
    for chunk in files.chunks(3) {
        inc.index_batch(chunk, IndexParallelism::Sequential, &cancel);
    }
    for i in 0..20usize {
        assert!(
            inc.contains_class(&format!("Vendor\\C{i}")),
            "incremental index missing Vendor\\C{i} before finalize"
        );
    }

    // Full rebuild in a fresh session.
    let mut full = make_session(root.path());
    for chunk in files.chunks(3) {
        full.index_batch(chunk, IndexParallelism::Sequential, &cancel);
    }
    full.finalize_index();

    for i in 0..20usize {
        assert_eq!(
            inc.contains_class(&format!("Vendor\\C{i}")),
            full.contains_class(&format!("Vendor\\C{i}")),
            "incremental vs full rebuild disagree on Vendor\\C{i}"
        );
    }

    // Finalize the incremental session — must remain complete (idempotent).
    inc.finalize_index();
    for i in 0..20usize {
        assert!(inc.contains_class(&format!("Vendor\\C{i}")));
    }
}

// ─── warm cache: body-only edits don't churn the index ────────────────────────

/// The headline guarantee: after the index is built, editing a project file's
/// method *body* (declared names unchanged) keeps the workspace symbol index
/// resolving both project and vendor declarations.
#[test]
fn body_only_edits_do_not_churn_workspace_index() {
    let root = create_temp_dir("no_churn");
    let app_src = root.path().join("src");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    fs::create_dir_all(&app_src).unwrap();
    fs::create_dir_all(&vendor_src).unwrap();
    write_composer(root.path());

    fs::write(
        vendor_src.join("Dep.php"),
        "<?php\nnamespace Vendor;\nclass Dep { public function go(): int { return 1; } }\n",
    )
    .unwrap();

    let mut session = make_session(root.path());
    let cancel = IndexCancel::new();
    let vfiles = indexable_files(root.path());
    session.index_batch(&vfiles, IndexParallelism::Sequential, &cancel);
    session.finalize_index();

    // Ingest a project file so the singleton includes it.
    let svc_path: Arc<str> = Arc::from(app_src.join("Svc.php").to_string_lossy().as_ref());
    let svc = |n: i32| {
        format!(
            "<?php\nnamespace App;\nclass Svc {{ public function run(): int {{ return {n}; }} }}\n"
        )
    };
    session.ingest_file(svc_path.clone(), Arc::from(svc(0).as_str()));

    assert!(session.contains_class("App\\Svc"));
    assert!(session.contains_class("Vendor\\Dep"));

    // 30 body-only edits (class/method names unchanged).
    for n in 1..=30 {
        session.ingest_file(svc_path.clone(), Arc::from(svc(n).as_str()));
    }
    assert!(session.contains_class("App\\Svc"));
    assert!(session.contains_class("Vendor\\Dep"));
}

/// A declaration-changing edit (renaming the class) DOES update the index —
/// the old name stops resolving and the new one starts. Verifies the
/// incremental subtract+add path stays correct.
#[test]
fn declaration_change_updates_index_incrementally() {
    let root = create_temp_dir("decl_change");
    let app_src = root.path().join("src");
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    let mut session = make_session(root.path());
    session.ensure_all_stubs();
    session.finalize_index();

    let path: Arc<str> = Arc::from(app_src.join("A.php").to_string_lossy().as_ref());
    session.ingest_file(
        path.clone(),
        Arc::from("<?php\nnamespace App;\nclass Alpha {}\n"),
    );
    assert!(session.contains_class("App\\Alpha"));

    // Rename Alpha → Beta.
    session.ingest_file(
        path.clone(),
        Arc::from("<?php\nnamespace App;\nclass Beta {}\n"),
    );
    assert!(
        session.contains_class("App\\Beta"),
        "renamed class must resolve after incremental update"
    );
    assert!(
        !session.contains_class("App\\Alpha"),
        "old class name must stop resolving after rename (incremental subtract)"
    );
}

/// When an edited file currently owns a duplicated symbol, the incremental
/// subtract path must detect the ambiguity and fall back to a rebuild instead
/// of dropping the surviving declaration from the workspace index.
#[test]
fn duplicate_winner_rename_preserves_surviving_symbol() {
    let root = create_temp_dir("decl_ambiguity");
    let app_src = root.path().join("src");
    fs::create_dir_all(&app_src).unwrap();
    write_composer(root.path());

    let mut session = make_session(root.path());
    session.ensure_all_stubs();
    session.finalize_index();

    let first: Arc<str> = Arc::from(app_src.join("First.php").to_string_lossy().as_ref());
    let second: Arc<str> = Arc::from(app_src.join("Second.php").to_string_lossy().as_ref());

    session.ingest_file(
        first.clone(),
        Arc::from("<?php\nnamespace App;\nclass Dup {}\n"),
    );
    session.ingest_file(
        second.clone(),
        Arc::from("<?php\nnamespace App;\nclass Dup {}\n"),
    );
    assert!(session.contains_class("App\\Dup"));

    session.ingest_file(
        second.clone(),
        Arc::from("<?php\nnamespace App;\nclass Unique {}\n"),
    );

    assert!(
        session.contains_class("App\\Dup"),
        "fallback rebuild should preserve the surviving duplicate declaration"
    );
    assert!(
        session.contains_class("App\\Unique"),
        "edited file's replacement declaration must resolve after rebuild"
    );
}

/// Adding or removing a class re-runs only the scopes whose symbol lookups it changes.
#[test]
fn declaration_edit_reruns_only_scopes_whose_lookups_changed() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from("/proj/Dep.php"),
        Arc::from("<?php\nnamespace App;\nclass Dep { public function go(): int { return 1; } }\n"),
    );
    session.ingest_file(
        Arc::from("/proj/User.php"),
        Arc::from(
            "<?php\nnamespace App;\n\
            function useDep(Dep $d): int { return $d->go(); }\n\
            function useFresh(): object { return new Fresh(); }\n",
        ),
    );
    let undefined_class_count = |session: &AnalysisSession| {
        let db = session.snapshot_db();
        let file = db.lookup_source_file("/proj/User.php").unwrap();
        analyze_file(&db, file)
            .issues
            .iter()
            .filter(|i| i.kind.name() == "UndefinedClass")
            .count()
    };
    assert_eq!(undefined_class_count(&session), 1);

    let before = session.work_count(Work::ScopeAnalysis);
    session.ingest_file(
        Arc::from("/proj/Fresh.php"),
        Arc::from("<?php\nnamespace App;\nclass Fresh {}\n"),
    );
    assert_eq!(undefined_class_count(&session), 0);
    assert_eq!(
        session.work_count(Work::ScopeAnalysis) - before,
        1,
        "only useFresh() looks up the added class"
    );

    session.ingest_file(
        Arc::from("/proj/Fresh.php"),
        Arc::from("<?php\nnamespace App;\nclass Other {}\n"),
    );
    assert_eq!(undefined_class_count(&session), 1);
}

/// Rebuilding an index with unchanged contents leaves the singleton input untouched.
#[test]
fn rebuild_with_unchanged_declarations_does_not_write_the_index() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from("/proj/Dep.php"),
        Arc::from("<?php\nnamespace App;\nclass Dep {}\nfunction dep(): void {}\n"),
    );
    let index_revision = |session: &AnalysisSession| {
        let db = session.snapshot_db();
        let revision = *db.workspace_symbol_index_singleton().unwrap().revision(&db);
        revision
    };
    session.rebuild_workspace_symbol_index();
    let before = index_revision(&session);

    session.rebuild_workspace_symbol_index();
    assert_eq!(index_revision(&session), before);

    session.ingest_file(
        Arc::from("/proj/Extra.php"),
        Arc::from("<?php\nnamespace App;\nclass Extra {}\n"),
    );
    assert_ne!(index_revision(&session), before);
}

// ─── cancellation ─────────────────────────────────────────────────────────────

/// A pre-cancelled token makes `index_batch` a no-op that reports `cancelled`.
#[test]
fn index_batch_honours_cancellation() {
    let root = create_temp_dir("cancel_index");
    let vendor_src = root.path().join("vendor/VendorLib/src");
    fs::create_dir_all(&vendor_src).unwrap();
    write_composer(root.path());
    fs::write(
        vendor_src.join("X.php"),
        "<?php\nnamespace Vendor;\nclass X {}\n",
    )
    .unwrap();

    let mut session = make_session(root.path());
    let files: Vec<(Arc<str>, Arc<str>)> = vec![(
        Arc::from("x"),
        Arc::from("<?php\nnamespace Vendor;\nclass X {}\n"),
    )];

    let cancel = IndexCancel::new();
    cancel.cancel();
    let outcome = session.index_batch(&files, IndexParallelism::Sequential, &cancel);
    assert!(outcome.cancelled);
    assert_eq!(outcome.registered, 0);
}
