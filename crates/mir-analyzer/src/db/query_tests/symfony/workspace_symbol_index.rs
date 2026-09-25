use super::load_full_symfony_fixture;
use crate::db::MirDatabase;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_workspace_symbol_index() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    assert_eq!(
        db.symbol_defining_file("Symfony\\Component\\HttpFoundation\\Request")
            .as_deref(),
        Some(fx.request.as_ref())
    );
    assert_eq!(
        db.symbol_defining_file("Symfony\\Component\\String\\u")
            .as_deref(),
        Some(fx.string_functions.as_ref())
    );
}
