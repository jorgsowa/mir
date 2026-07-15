//! Regression: batch/vendor ingestion must feed the subtype index, not just
//! the single-file `ingest_file` (LSP edit) path.
//!
//! Both `collect_definitions` (the vendor-tree walker used to preload types)
//! and `analyze_paths` (the CLI project-file batch pipeline) collect a
//! `StubSlice` per file but previously never turned it into subtype-index
//! class edges, so `indexed_subtype_classes`/goto-implementation could not
//! see implementors living in a vendor tree or in a batch-only project run
//! that never went through `ingest_file`.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::create_temp_dir;

#[test]
fn vendor_tree_implementor_reaches_subtype_index() {
    let dir = create_temp_dir("vendor_subtype_index_vendor");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("vendor/acme/lib/src")).unwrap();

    let shape = root.join("src/Shape.php");
    fs::write(&shape, "<?php\nnamespace Shop;\ninterface Shape {}\n").unwrap();

    let vendor_impl = root.join("vendor/acme/lib/src/Impl.php");
    fs::write(
        &vendor_impl,
        "<?php\nnamespace Vendor;\nclass Impl implements \\Shop\\Shape {}\n",
    )
    .unwrap();

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    // Vendor ingestion: type definitions only, no body analysis.
    session.collect_definitions(std::slice::from_ref(&vendor_impl));

    // Project ingestion via the batch pipeline (never touches `ingest_file`).
    session.analyze_paths(std::slice::from_ref(&shape), &BatchOptions::new());

    // Query scope deliberately excludes the vendor file: the on-demand
    // self-heal (`commit_defs_for_matching`) only ever looks at files in this
    // list, so if the vendor implementor shows up here it can only be because
    // `collect_definitions` itself committed its class edges.
    let shape_arc: Arc<str> = Arc::from(shape.to_string_lossy().as_ref());
    let subs = session.indexed_subtype_classes("Shop\\Shape", &[shape_arc], false);
    let fqcns: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert!(
        fqcns.contains(&"Vendor\\Impl"),
        "vendor implementor must reach the subtype index via collect_definitions: {fqcns:?}"
    );
}

#[test]
fn batch_project_implementor_reaches_subtype_index_without_ingest_file() {
    let dir = create_temp_dir("vendor_subtype_index_batch");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();

    let shape = root.join("src/Shape.php");
    fs::write(&shape, "<?php\nnamespace Shop;\ninterface Shape {}\n").unwrap();

    let impl_file = root.join("src/Circle.php");
    fs::write(
        &impl_file,
        "<?php\nnamespace Shop;\nclass Circle implements Shape {}\n",
    )
    .unwrap();

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    // Pure CLI batch pipeline — no `ingest_file` call anywhere.
    session.analyze_paths(&[shape.clone(), impl_file.clone()], &BatchOptions::new());

    // Query scope excludes Circle.php so self-heal cannot independently
    // discover it; only `analyze_paths` committing its edges makes this pass.
    let shape_arc: Arc<str> = Arc::from(shape.to_string_lossy().as_ref());
    let subs = session.indexed_subtype_classes("Shop\\Shape", &[shape_arc], false);
    let fqcns: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert!(
        fqcns.contains(&"Shop\\Circle"),
        "batch-analyzed implementor must reach the subtype index: {fqcns:?}"
    );
}
