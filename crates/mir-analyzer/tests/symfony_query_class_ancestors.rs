mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::class_ancestors;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_class_ancestors() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let input_bag = fqcn(&db, "Symfony\\Component\\HttpFoundation\\InputBag");
    let ancestors = class_ancestors(&db, input_bag);
    let names: Vec<&str> = ancestors.0.iter().map(|a| a.as_ref()).collect();
    assert_eq!(
        names,
        vec![
            "Symfony\\Component\\HttpFoundation\\ParameterBag",
            "IteratorAggregate",
            "Traversable",
            "iterable",
            "Countable",
        ],
        "InputBag ancestor expansion should resolve the exact inherited surface without including self"
    );
}
