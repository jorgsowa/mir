//! Tests for the Phase-4 workspace-enumeration tracked queries.
//!
//! `workspace_classes` / `workspace_functions` aggregate FQCNs/FQNs
//! across every registered SourceFile. They power the pull-path
//! equivalent of `MirDb::active_*_fqcns` walks used by class.rs,
//! dead_code.rs, project.rs.

use std::sync::Arc;

use mir_analyzer::db::{workspace_classes, workspace_functions};
use mir_analyzer::{AnalysisSession, PhpVersion};

#[test]
fn workspace_classes_empty_for_empty_session() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let db = session.snapshot_db();
    let classes = workspace_classes(&db);
    assert!(classes.is_empty());
}

#[test]
fn workspace_classes_aggregates_across_files() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/A.php"),
        Arc::from("<?php\nnamespace App;\nclass A {}\ninterface IFoo {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/B.php"),
        Arc::from("<?php\nnamespace App;\nclass B {}\ntrait T {}\nenum E {}\n"),
    );

    let db = session.snapshot_db();
    let classes = workspace_classes(&db);
    let names: Vec<&str> = classes.iter().map(|s| s.as_ref()).collect();
    assert!(names.contains(&"App\\A"));
    assert!(names.contains(&"App\\B"));
    assert!(names.contains(&"App\\IFoo"));
    assert!(names.contains(&"App\\T"));
    assert!(names.contains(&"App\\E"));
}

#[test]
fn workspace_functions_aggregates_across_files() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from(
            "<?php\nnamespace App;\nfunction one(): void {} function two(): int { return 0; }\n",
        ),
    );

    let db = session.snapshot_db();
    let fns = workspace_functions(&db);
    let names: Vec<&str> = fns.iter().map(|s| s.as_ref()).collect();
    assert!(names.contains(&"App\\one"));
    assert!(names.contains(&"App\\two"));
}

#[test]
fn workspace_revision_bumps_on_remove() {
    // workspace_classes must invalidate when a file is removed.
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/A.php"),
        Arc::from("<?php\nnamespace App;\nclass A {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/B.php"),
        Arc::from("<?php\nnamespace App;\nclass B {}\n"),
    );
    let db1 = session.snapshot_db();
    assert_eq!(workspace_classes(&db1).len(), 2);
    drop(db1);

    session.invalidate_file("/proj/B.php");
    let db2 = session.snapshot_db();
    let classes = workspace_classes(&db2);
    let names: Vec<&str> = classes.iter().map(|s| s.as_ref()).collect();
    assert_eq!(names, vec!["App\\A"]);
}
