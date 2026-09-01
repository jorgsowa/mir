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
    assert!(
        !inferred.methods.is_empty(),
        "inference narrows RequestStack's method bodies even though they already declare native \
         return types — the narrowed type feeds callers that want more than the declared signature"
    );
    assert_eq!(
        inferred.properties.len(),
        0,
        "RequestStack should not infer constructor-only property types for this fixture file"
    );
    assert!(
        inferred_method_return_type_demand(
            &db,
            "Symfony\\Component\\HttpFoundation\\RequestStack",
            "pop",
        )
        .is_some(),
        "demand-driven inference should surface RequestStack::pop's narrowed body-derived type"
    );
    assert!(
        inferred_method_return_type_demand(
            &db,
            "Symfony\\Component\\HttpFoundation\\RequestStack",
            "getsession",
        )
        .is_some(),
        "demand-driven inference should surface RequestStack::getSession's narrowed body-derived type"
    );
}
