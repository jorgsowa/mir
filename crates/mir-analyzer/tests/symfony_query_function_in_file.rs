mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{function_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_function_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.string_functions.as_ref())
        .expect("string functions file");
    let fn_u = fqcn(&db, "Symfony\\Component\\String\\u");
    let found = function_in_file(&db, file, fn_u)
        .clone()
        .expect("u function");
    assert_eq!(found.fqn.as_ref(), "Symfony\\Component\\String\\u");
    assert_eq!(found.short_name.as_ref(), "u");
    assert!(
        found.location.is_some(),
        "u() should retain a declaration location"
    );
    assert_eq!(found.params.len(), 1);
    assert_eq!(found.params[0].name.as_ref(), "string");
    assert_eq!(
        found.return_type.as_ref().map(|ty| ty.to_string()),
        Some("Symfony\\Component\\String\\UnicodeString".to_string())
    );
    assert!(
        found.inferred_return_type.is_none(),
        "u() should not need a fallback inferred return type when a native declaration exists"
    );
}
