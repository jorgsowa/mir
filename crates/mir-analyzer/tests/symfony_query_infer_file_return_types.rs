mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{infer_file_return_types, inferred_method_return_type_demand, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_infer_file_return_types() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let inferred = infer_file_return_types(&db, file);
    assert_eq!(
        inferred.functions.len(),
        0,
        "RequestStack should not infer standalone function return types"
    );
    assert_eq!(
        inferred.methods.len(),
        0,
        "RequestStack methods already declare return types, so the inference map should stay empty"
    );
    assert_eq!(
        inferred.properties.len(),
        0,
        "RequestStack should not infer constructor-only property types for this fixture file"
    );
    assert_eq!(
        inferred_method_return_type_demand(
            &db,
            "Symfony\\Component\\HttpFoundation\\RequestStack",
            "pop",
        ),
        None,
        "demand-driven inference should not synthesize RequestStack::pop when a native return type exists"
    );
    assert!(
        inferred_method_return_type_demand(
            &db,
            "Symfony\\Component\\HttpFoundation\\RequestStack",
            "getsession",
        )
        .is_none(),
        "getSession should also resolve from declarations rather than inferred return-type state"
    );
}
