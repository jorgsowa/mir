mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{global_constant_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_global_constant_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.string_functions.as_ref())
        .expect("string functions file");
    let missing = fqcn(&db, "Symfony\\Component\\String\\MISSING_CONST");
    let fn_u = fqcn(&db, "Symfony\\Component\\String\\u");
    assert!(
        global_constant_in_file(&db, file, missing).is_none(),
        "Symfony String helpers file should not define a project global constant"
    );
    assert!(
        global_constant_in_file(&db, file, fn_u).is_none(),
        "function names must not alias to constants in the file query"
    );
}
