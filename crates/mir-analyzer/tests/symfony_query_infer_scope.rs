mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{infer_scope, MirDatabase, ScopeKey};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_infer_scope() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let header = infer_scope(&db, file, ScopeKey::FileHeader);
    assert!(
        header.issues.is_empty(),
        "RequestStack header scope should analyze cleanly"
    );
    assert_eq!(header.ref_locs.len(), 2);
    let symbols: BTreeSet<&str> = header.ref_locs.iter().map(|loc| loc.symbol_key.as_ref()).collect();
    assert_eq!(
        symbols,
        BTreeSet::from([
            "use:cls:Symfony\\Component\\HttpFoundation\\Exception\\SessionNotFoundException",
            "use:cls:Symfony\\Component\\HttpFoundation\\Session\\SessionInterface",
        ]),
        "header inference should record the two imported class-like references"
    );
    assert!(
        header
            .ref_locs
            .iter()
            .all(|loc| loc.file.as_ref() == fx.request_stack.as_ref()),
        "header references should point back to the analyzed RequestStack file"
    );
}
