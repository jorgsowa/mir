//! Workspace-wide enumeration: `all_classes` / `all_functions` aggregate
//! declarations across every registered file.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion};

fn class_names(session: &AnalysisSession) -> Vec<String> {
    let mut names: Vec<String> = session
        .all_classes()
        .into_iter()
        .map(|(fqcn, _)| fqcn.to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn all_classes_empty_for_empty_session() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    assert!(session.all_classes().is_empty());
}

#[test]
fn all_classes_aggregates_across_files() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/A.php"),
        Arc::from("<?php\nnamespace App;\nclass A {}\ninterface IFoo {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/B.php"),
        Arc::from("<?php\nnamespace App;\nclass B {}\ntrait T {}\nenum E {}\n"),
    );

    assert_eq!(
        class_names(&session),
        vec!["App\\A", "App\\B", "App\\E", "App\\IFoo", "App\\T"]
    );
}

#[test]
fn all_functions_aggregates_across_files() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from(
            "<?php\nnamespace App;\nfunction one(): void {} function two(): int { return 0; }\n",
        ),
    );

    let fns = session.all_functions();
    let names: Vec<&str> = fns.iter().map(|(fqn, _)| fqn.as_ref()).collect();
    assert!(names.contains(&"App\\one"));
    assert!(names.contains(&"App\\two"));
}

#[test]
fn all_classes_drops_removed_file() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/A.php"),
        Arc::from("<?php\nnamespace App;\nclass A {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/B.php"),
        Arc::from("<?php\nnamespace App;\nclass B {}\n"),
    );
    assert_eq!(class_names(&session), vec!["App\\A", "App\\B"]);

    session.invalidate_file("/proj/B.php");
    assert_eq!(class_names(&session), vec!["App\\A"]);
}
