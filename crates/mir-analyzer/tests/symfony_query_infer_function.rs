mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{infer_function, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_infer_function() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.string_functions.as_ref())
        .expect("string functions file");
    let fn_u = fqcn(&db, "Symfony\\Component\\String\\u");
    let inferred = infer_function(&db, file, fn_u.name(&db).as_str().into())
        .clone()
        .expect("infer_function should find u()");
    assert!(inferred.issues.is_empty());
    assert!(
        inferred.ref_locs.len() >= 2,
        "u() inference should record its constructor call and input-symbol references"
    );
    assert!(
        inferred.return_type.is_some(),
        "infer_function should surface u()'s return type"
    );
    assert_eq!(
        inferred.return_type.as_ref().map(|t| t.to_string()),
        Some("Symfony\\Component\\String\\UnicodeString".to_string())
    );
    assert!(
        inferred
            .ref_locs
            .iter()
            .any(|loc| loc.symbol_key.as_ref() == "cls:Symfony\\Component\\String\\UnicodeString"),
        "u() should reference the UnicodeString constructor target"
    );
    assert!(
        inferred
            .ref_locs
            .iter()
            .all(|loc| loc.file.as_ref() == fx.string_functions.as_ref()),
        "u() reference locations should point back to the Symfony string helper file"
    );
}
