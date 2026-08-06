mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::class_array_property_defaults;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_class_array_property_defaults() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let router = fqcn(&db, "Symfony\\Component\\Routing\\Router");
    let defaults = class_array_property_defaults(&db, router);
    let properties: BTreeSet<&str> = defaults.iter().map(|d| d.property.as_ref()).collect();
    assert!(
        properties.is_superset(&BTreeSet::from(["cache", "expressionLanguageProviders", "options"])),
        "Router should expose its array-typed defaulted properties"
    );
    assert!(
        defaults.iter().any(|d| d.property == "cache"),
        "Router should expose its static empty array default"
    );
    let cache = defaults
        .iter()
        .find(|d| d.property == "cache")
        .expect("cache default");
    assert!(cache.entries.is_empty());
    let options = defaults
        .iter()
        .find(|d| d.property == "options")
        .expect("options default");
    assert!(options.entries.is_empty());
}
