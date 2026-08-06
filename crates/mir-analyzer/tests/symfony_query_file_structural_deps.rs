mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{file_structural_deps, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_file_structural_deps() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let deps = file_structural_deps(&db, file);
    let dep_set: BTreeSet<&str> = deps.iter().map(|p| p.as_ref()).collect();
    assert!(deps.len() >= 2);
    assert!(
        dep_set.contains(fx.request.as_ref()),
        "RequestStack should structurally depend on Request"
    );
    assert!(
        dep_set
            .iter()
            .any(|p| p.ends_with("Session/SessionInterface.php")),
        "RequestStack should structurally depend on SessionInterface"
    );
    assert!(
        dep_set
            .iter()
            .any(|p| p.ends_with("Exception/SessionNotFoundException.php")),
        "RequestStack should structurally depend on SessionNotFoundException"
    );
    assert_eq!(
        dep_set.len(),
        deps.len(),
        "structural deps should already be deduplicated"
    );
    assert!(!dep_set.contains(fx.request_stack.as_ref()));
}
