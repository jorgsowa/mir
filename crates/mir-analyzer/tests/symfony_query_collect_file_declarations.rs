mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{collect_file_declarations, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_collect_file_declarations() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db.lookup_source_file(fx.request.as_ref()).expect("Request file");
    let decls = collect_file_declarations(&db, file);
    assert_eq!(decls.class_like.len(), 1, "Request.php should export one class-like symbol");
    assert!(
        decls.functions.is_empty(),
        "Request.php should not export free functions"
    );
    assert!(
        decls.constants.is_empty(),
        "Request.php should not export file-level constants"
    );
    assert_eq!(
        decls.class_like[0].0.as_str(),
        "symfony\\component\\httpfoundation\\request"
    );
    assert_eq!(decls.class_like[0].1.file().path(&db).as_ref(), fx.request.as_ref());
    assert!(
        matches!(
            decls.class_like[0].1,
            mir_analyzer::db::SymbolLoc::Class { idx: 0, .. }
        ),
        "Request.php should register its first declaration as the Request class symbol"
    );
}
