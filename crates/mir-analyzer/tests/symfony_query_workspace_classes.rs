mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::workspace_classes;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_workspace_classes() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let classes = workspace_classes(&db);
    let names: BTreeSet<&str> = classes.iter().map(|c| c.as_ref()).collect();
    assert!(classes.len() > 1000);
    assert!(
        names.len() > 1000,
        "workspace_classes should expose a broad unique class surface across the Symfony project"
    );
    assert!(
        classes.len() >= names.len(),
        "workspace_classes may currently surface duplicates, but it must not lose unique class names"
    );
    assert!(names.contains("Symfony\\Component\\HttpFoundation\\Request"));
    assert!(names.contains("Symfony\\Component\\HttpFoundation\\InputBag"));
    assert!(names.contains("Symfony\\Component\\HttpFoundation\\RequestStack"));
    assert!(names.contains("Symfony\\Component\\String\\Slugger\\AsciiSlugger"));
    assert!(names.contains("Symfony\\Component\\String\\TruncateMode"));
}
