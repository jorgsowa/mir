mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{file_scopes, MirDatabase, ScopeKey};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_file_scopes() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let scopes = file_scopes(&db, file);
    assert_eq!(scopes.len(), 1, "RequestStack should contribute one class-like scope");
    assert!(
        scopes.iter().any(|scope| matches!(
            scope,
            ScopeKey::ClassLike(name, _) if name.as_ref()
                == "Symfony\\Component\\HttpFoundation\\RequestStack"
        )),
        "RequestStack class scope should be discoverable"
    );
    assert!(
        !scopes.iter().any(|scope| matches!(scope, ScopeKey::FileHeader)),
        "file_scopes should not duplicate the file header as a class-like scope entry"
    );
}
