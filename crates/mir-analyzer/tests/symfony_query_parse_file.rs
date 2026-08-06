mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{parse_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_parse_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let parsed = parse_file(&db, file);
    assert!(
        parsed.0.errors.is_empty(),
        "RequestStack should parse without syntax errors"
    );
    assert!(!parsed.0.program.stmts.is_empty());
}
