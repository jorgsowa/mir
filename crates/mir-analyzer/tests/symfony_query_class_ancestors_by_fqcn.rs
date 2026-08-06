mod common_symfony;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::class_ancestors_by_fqcn;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_class_ancestors_by_fqcn() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let input_bag = fqcn(&db, "Symfony\\Component\\HttpFoundation\\InputBag");
    let ancestors = class_ancestors_by_fqcn(&db, input_bag);
    let names: Vec<&str> = ancestors.iter().map(|a| a.as_ref()).collect();
    assert_eq!(
        names,
        vec![
            "Symfony\\Component\\HttpFoundation\\InputBag",
            "Symfony\\Component\\HttpFoundation\\ParameterBag",
            "IteratorAggregate",
            "Traversable",
            "iterable",
            "Countable",
        ],
        "class_ancestors_by_fqcn should include self followed by the full inherited chain"
    );
}
