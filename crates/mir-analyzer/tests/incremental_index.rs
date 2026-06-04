//! Tests for incremental workspace-symbol-index maintenance and the warm-cache
//! (no-churn) guarantee that the eager-static-input model depends on.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

use self::common::create_temp_dir;

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

/// Pointer identity of the workspace symbol index's `class_like` map. Stable
/// across reads unless the singleton input was rewritten — our proxy for "did
/// indexing churn the warm cache".
fn class_like_ptr(session: &AnalysisSession) -> usize {
    session.read(|db| {
        let idx = mir_analyzer::db::workspace_index(db);
        Arc::as_ptr(&idx.class_like) as usize
    })
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
    let inc = make_session(root.path());
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
    let full = make_session(root.path());
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
/// method *body* (declared names unchanged) must NOT rewrite the workspace
/// symbol index singleton — otherwise every keystroke would cascade-invalidate
/// vendor body-analysis memos.
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

    let session = make_session(root.path());
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

    let ptr_before = class_like_ptr(&session);
    assert!(session.contains_class("App\\Svc"));
    assert!(session.contains_class("Vendor\\Dep"));

    // 30 body-only edits (class/method names unchanged).
    for n in 1..=30 {
        session.ingest_file(svc_path.clone(), Arc::from(svc(n).as_str()));
    }
    let ptr_after = class_like_ptr(&session);

    assert_eq!(
        ptr_before, ptr_after,
        "body-only edits must not rewrite the workspace symbol index singleton (warm cache)"
    );
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

    let session = make_session(root.path());
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

    let session = make_session(root.path());
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
