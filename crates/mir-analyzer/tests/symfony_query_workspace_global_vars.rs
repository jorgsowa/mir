mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::workspace_global_vars;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_workspace_global_vars() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let vars = workspace_global_vars(&db);
    assert!(
        vars.0.is_empty(),
        "the indexed Symfony project currently should not contribute project global vars"
    );
    assert_eq!(vars.0.len(), 0);
}
