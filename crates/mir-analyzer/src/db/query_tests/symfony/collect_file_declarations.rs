use super::load_full_symfony_fixture;
use crate::db::{collect_file_declarations, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_collect_file_declarations() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request.as_ref())
        .expect("Request file");
    let decls = collect_file_declarations(&db, file);
    assert_eq!(
        decls.class_like().count(),
        1,
        "Request.php should export one class-like symbol"
    );
    assert!(
        decls.functions().count() == 0,
        "Request.php should not export free functions"
    );
    assert!(
        decls.constants().count() == 0,
        "Request.php should not export file-level constants"
    );
    let request = decls.class_like_at(0).expect("first class-like decl");
    assert_eq!(
        request.lookup_key().as_str(),
        "symfony\\component\\httpfoundation\\request"
    );
    assert_eq!(request.loc.file().path(&db).as_ref(), fx.request.as_ref());
    assert!(
        matches!(request.loc, crate::db::SymbolLoc::Class { idx: 0, .. }),
        "Request.php should register its first declaration as the Request class symbol"
    );
}
