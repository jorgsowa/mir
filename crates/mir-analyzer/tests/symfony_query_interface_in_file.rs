mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{interface_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_interface_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.session_interface.as_ref())
        .expect("SessionInterface file");
    let session_interface = fqcn(
        &db,
        "Symfony\\Component\\HttpFoundation\\Session\\SessionInterface",
    );
    let found = interface_in_file(&db, file, session_interface)
        .clone()
        .expect("SessionInterface");
    assert_eq!(
        found.fqcn.as_ref(),
        "Symfony\\Component\\HttpFoundation\\Session\\SessionInterface"
    );
    assert_eq!(found.short_name.as_ref(), "SessionInterface");
    assert!(found.extends.is_empty(), "SessionInterface should not extend other interfaces here");
    assert!(found.own_constants.is_empty(), "SessionInterface should not declare constants");
    assert!(
        found.location.is_some(),
        "SessionInterface should retain a declaration location"
    );
    let methods: BTreeSet<&str> = found.own_methods.keys().map(|k| k.as_ref()).collect();
    assert_eq!(
        methods,
        BTreeSet::from([
            "all",
            "clear",
            "get",
            "getbag",
            "getid",
            "getmetadatabag",
            "getname",
            "has",
            "invalidate",
            "isstarted",
            "migrate",
            "registerbag",
            "remove",
            "replace",
            "save",
            "set",
            "setid",
            "setname",
            "start",
        ]),
        "SessionInterface should expose its exact interface method set"
    );
}
