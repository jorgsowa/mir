mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{enum_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_enum_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.truncate_mode.as_ref())
        .expect("TruncateMode file");
    let truncate_mode = fqcn(&db, "Symfony\\Component\\String\\TruncateMode");
    let found = enum_in_file(&db, file, truncate_mode)
        .clone()
        .expect("TruncateMode enum");
    assert_eq!(found.fqcn.as_ref(), "Symfony\\Component\\String\\TruncateMode");
    assert_eq!(found.short_name.as_ref(), "TruncateMode");
    assert!(found.scalar_type.is_none(), "TruncateMode should be a pure enum");
    assert_eq!(
        found.interfaces.iter().map(|i| i.as_ref()).collect::<Vec<_>>(),
        vec!["UnitEnum"],
        "TruncateMode should expose PHP's implicit UnitEnum contract"
    );
    assert!(found.traits.is_empty(), "TruncateMode should not use traits");
    assert!(found.own_methods.is_empty(), "TruncateMode should not declare methods");
    assert!(found.own_constants.is_empty(), "TruncateMode should not declare extra constants");
    let cases: BTreeSet<&str> = found.cases.keys().map(|c| c.as_ref()).collect();
    assert_eq!(
        cases,
        BTreeSet::from(["Char", "WordAfter", "WordBefore"]),
        "TruncateMode should preserve its exact enum case set"
    );
}
