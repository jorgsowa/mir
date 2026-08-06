mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{class_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_class_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db.lookup_source_file(fx.request.as_ref()).expect("Request file");
    let request = fqcn(&db, "Symfony\\Component\\HttpFoundation\\Request");
    let found = class_in_file(&db, file, request)
        .clone()
        .expect("Request class");
    assert_eq!(found.fqcn.as_ref(), "Symfony\\Component\\HttpFoundation\\Request");
    assert_eq!(found.short_name.as_ref(), "Request");
    assert!(found.parent.is_none(), "Request should not extend another class");
    assert!(
        found.interfaces.is_empty(),
        "Request should not declare implemented interfaces directly"
    );
    assert!(!found.is_abstract, "Request should be a concrete class");
    assert!(!found.is_final, "Request should not be final in this Symfony fixture");
    assert!(
        found.own_properties.contains_key("request"),
        "Request should expose the request input bag property"
    );
    assert!(
        found.own_properties.contains_key("query"),
        "Request should expose the query input bag property"
    );
    assert!(
        found.own_properties.contains_key("session"),
        "Request should expose the session storage property"
    );
    assert!(found.own_methods.contains_key("getsession"));
    assert!(found.own_methods.contains_key("hassession"));
    assert!(found.own_methods.contains_key("getmethod"));
    assert!(found.own_constants.contains_key("METHOD_GET"));
    assert!(found.own_constants.contains_key("METHOD_POST"));
    assert!(found.own_constants.contains_key("HEADER_FORWARDED"));
    assert!(
        found.own_methods.len() > 50,
        "Request should preserve its broad method surface in the collected class definition"
    );
    assert!(
        found.own_constants.len() > 10,
        "Request should preserve its HTTP method and header constant set"
    );
}
