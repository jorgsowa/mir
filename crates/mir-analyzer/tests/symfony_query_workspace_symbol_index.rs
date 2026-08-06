mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{workspace_symbol_index, MirDatabase};
use mir_types::Name;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_workspace_symbol_index() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let index = workspace_symbol_index(&db);
    assert!(index.class_like.len() > 1000);
    assert!(index.functions.len() > 100);
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
    assert!(
        index
            .class_like_by_short_name
            .contains_key(&Name::new("request")),
        "workspace index must expose the Request short-name bucket"
    );
    assert!(
        index
            .class_like_by_short_name
            .get(&Name::new("request"))
            .is_some_and(|bucket| bucket.iter().any(|name| {
                name.as_str() == "symfony\\component\\httpfoundation\\request"
            })),
        "the Request short-name bucket should resolve to Symfony\\Component\\HttpFoundation\\Request"
    );
    assert!(
        index.class_like.contains_key(
            &Name::new("Symfony\\Component\\HttpFoundation\\Request").ascii_lowercase()
        ),
        "workspace class-like index should contain Request by its lowered FQCN key"
    );
    assert!(
        index
            .functions
            .contains_key(&Name::new("Symfony\\Component\\String\\u").ascii_lowercase()),
        "workspace function index should contain the u() helper by its lowered FQN key"
    );
}
