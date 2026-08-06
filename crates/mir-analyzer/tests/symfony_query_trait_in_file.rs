mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::{fqcn, load_full_symfony_fixture};
use mir_analyzer::db::{trait_in_file, MirDatabase};

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_trait_in_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.compiled_url_matcher_trait.as_ref())
        .expect("CompiledUrlMatcherTrait file");
    let trait_fqcn = fqcn(
        &db,
        "Symfony\\Component\\Routing\\Matcher\\Dumper\\CompiledUrlMatcherTrait",
    );
    let found = trait_in_file(&db, file, trait_fqcn)
        .clone()
        .expect("CompiledUrlMatcherTrait");
    assert_eq!(
        found.fqcn.as_ref(),
        "Symfony\\Component\\Routing\\Matcher\\Dumper\\CompiledUrlMatcherTrait"
    );
    assert_eq!(found.short_name.as_ref(), "CompiledUrlMatcherTrait");
    assert!(found.traits.is_empty(), "CompiledUrlMatcherTrait should not use nested traits");
    assert_eq!(
        found.own_properties.len(),
        6,
        "CompiledUrlMatcherTrait should retain its native and docblock-declared helper properties"
    );
    let properties: BTreeSet<&str> = found.own_properties.keys().map(|k| k.as_ref()).collect();
    assert_eq!(
        properties,
        BTreeSet::from([
            "checkCondition",
            "context",
            "dynamicRoutes",
            "matchHost",
            "regexpList",
            "staticRoutes",
        ]),
        "CompiledUrlMatcherTrait should expose the exact property surface, including its docblock context property"
    );
    let methods: BTreeSet<&str> = found.own_methods.keys().map(|k| k.as_ref()).collect();
    assert_eq!(
        methods,
        BTreeSet::from(["domatch", "match"]),
        "CompiledUrlMatcherTrait should expose exactly its two declared methods"
    );
}
