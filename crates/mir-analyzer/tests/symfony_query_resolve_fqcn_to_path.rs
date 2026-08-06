mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::resolve_fqcn_to_path;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_resolve_fqcn_to_path() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let request = fqcn(&db, "Symfony\\Component\\HttpFoundation\\Request");
    let request_stack = fqcn(&db, "Symfony\\Component\\HttpFoundation\\RequestStack");
    let ascii_slugger = fqcn(&db, "Symfony\\Component\\String\\Slugger\\AsciiSlugger");
    let session_interface = fqcn(
        &db,
        "Symfony\\Component\\HttpFoundation\\Session\\SessionInterface",
    );
    let missing = fqcn(&db, "Symfony\\Component\\HttpFoundation\\DefinitelyMissing");
    assert_eq!(
        resolve_fqcn_to_path(&db, request).as_deref(),
        Some(fx.request.as_ref())
    );
    assert_eq!(
        resolve_fqcn_to_path(&db, request_stack).as_deref(),
        Some(fx.request_stack.as_ref())
    );
    assert_eq!(
        resolve_fqcn_to_path(&db, ascii_slugger).as_deref(),
        Some(fx.ascii_slugger.as_ref())
    );
    assert_eq!(
        resolve_fqcn_to_path(&db, session_interface).as_deref(),
        Some(fx.session_interface.as_ref())
    );
    assert!(resolve_fqcn_to_path(&db, missing).is_none());
}
