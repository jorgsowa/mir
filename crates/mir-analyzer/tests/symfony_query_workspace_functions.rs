mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::workspace_functions;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_workspace_functions() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let functions = workspace_functions(&db);
    let names: BTreeSet<&str> = functions.iter().map(|f| f.as_ref()).collect();
    assert!(functions.len() > 100);
    assert!(
        names.len() > 100,
        "workspace_functions should expose a broad unique function surface across the Symfony project"
    );
    assert!(
        functions.len() >= names.len(),
        "workspace_functions may currently surface duplicates, but it must not lose unique function names"
    );
    assert!(names.contains("Symfony\\Component\\String\\u"));
    assert!(names.contains("Symfony\\Component\\String\\b"));
    assert!(names.contains("Symfony\\Component\\String\\s"));
}
