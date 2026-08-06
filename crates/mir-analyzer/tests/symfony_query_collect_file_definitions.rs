mod common_symfony;

use std::collections::BTreeSet;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{collect_file_definitions, MirDatabase};
use mir_codebase::definitions::Visibility;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_collect_file_definitions() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let defs = collect_file_definitions(&db, file);

    assert!(
        defs.issues.is_empty(),
        "collect_file_definitions should not emit collector issues for RequestStack"
    );
    assert_eq!(
        defs.slice.classes.len(),
        1,
        "RequestStack.php should define exactly one class"
    );
    assert!(
        defs.slice.interfaces.is_empty(),
        "RequestStack.php should not define interfaces"
    );
    assert!(
        defs.slice.traits.is_empty(),
        "RequestStack.php should not define traits"
    );
    assert!(
        defs.slice.enums.is_empty(),
        "RequestStack.php should not define enums"
    );
    assert!(
        defs.slice.functions.is_empty(),
        "RequestStack.php should not define free functions"
    );
    assert!(
        defs.slice.constants.is_empty(),
        "RequestStack.php should not define file-level constants"
    );
    assert!(
        defs.slice.global_vars.is_empty(),
        "RequestStack.php should not define global vars"
    );

    let class = &defs.slice.classes[0];
    assert_eq!(
        class.fqcn.as_ref(),
        "Symfony\\Component\\HttpFoundation\\RequestStack"
    );
    assert_eq!(class.short_name.as_ref(), "RequestStack");
    assert_eq!(
        class.parent, None,
        "RequestStack should not extend another class"
    );
    assert!(
        class.interfaces.is_empty(),
        "RequestStack should not declare implemented interfaces"
    );
    assert!(
        class.traits.is_empty(),
        "RequestStack should not use traits"
    );
    assert!(!class.is_abstract, "RequestStack is a concrete class");
    assert!(!class.is_final, "RequestStack should not be final");
    assert!(!class.is_readonly, "RequestStack should not be readonly");
    assert!(
        !class.is_internal,
        "RequestStack should be a userland Symfony class"
    );
    assert!(
        class.location.is_some(),
        "RequestStack should retain its declaration location"
    );

    assert_eq!(
        class.own_properties.len(),
        1,
        "RequestStack should expose only its requests property"
    );
    assert!(
        class.own_properties.contains_key("requests"),
        "RequestStack should collect the requests property"
    );

    let method_names: BTreeSet<&str> = class.own_methods.keys().map(|k| k.as_ref()).collect();
    assert_eq!(
        method_names,
        BTreeSet::from([
            "__construct",
            "getcurrentrequest",
            "getmainrequest",
            "getparentrequest",
            "getsession",
            "pop",
            "push",
            "resetrequestformats",
        ]),
        "RequestStack should expose the full declared method surface"
    );

    let push = class
        .own_methods
        .get("push")
        .expect("RequestStack::push should be present");
    assert_eq!(push.name.as_ref(), "push");
    assert_eq!(push.fqcn.as_ref(), class.fqcn.as_ref());
    assert_eq!(push.visibility, Visibility::Public);
    assert!(!push.is_static, "push should be an instance method");
    assert!(
        !push.is_constructor,
        "push should not be marked as constructor"
    );
    assert_eq!(
        push.params.len(),
        1,
        "push should take a single Request argument"
    );
    assert!(
        push.return_type.is_some(),
        "push should preserve its native void return type"
    );

    let pop = class
        .own_methods
        .get("pop")
        .expect("RequestStack::pop should be present");
    assert_eq!(pop.name.as_ref(), "pop");
    assert_eq!(pop.fqcn.as_ref(), class.fqcn.as_ref());
    assert_eq!(pop.visibility, Visibility::Public);
    assert!(!pop.is_static, "pop should be an instance method");
    assert!(
        !pop.is_constructor,
        "pop should not be marked as constructor"
    );
    assert!(pop.params.is_empty(), "pop should not take any parameters");
    assert!(
        pop.return_type.is_some(),
        "pop should preserve its nullable Request return type"
    );
}
