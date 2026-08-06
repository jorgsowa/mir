mod common_symfony;

use common_symfony::load_full_symfony_fixture;
use mir_analyzer::db::{analyze_file, MirDatabase};
use mir_analyzer::{Issue, IssueKind};
use mir_types::Location;

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn symfony_query_analyze_file() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();
    let file = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let analyzed = analyze_file(&db, file);
    assert!(
        !analyzed.ref_locs.is_empty(),
        "RequestStack analysis should record symbol references"
    );

    let string_functions = db
        .lookup_source_file(fx.string_functions.as_ref())
        .expect("string functions file");
    let string_out = analyze_file(&db, string_functions);
    assert_eq!(
        string_out.issues.len(),
        0,
        "Symfony string helper functions should analyze cleanly"
    );
    assert_eq!(string_out.ref_locs.len(), 15);
}

#[test]
#[ignore = "known false positives in Symfony fixture analysis"]
fn symfony_query_analyze_file_false_positives() {
    let Some(fx) = load_full_symfony_fixture() else {
        eprintln!("skipping: Symfony fixture not available");
        return;
    };

    let db = fx.session.snapshot_db();

    let request_stack = db
        .lookup_source_file(fx.request_stack.as_ref())
        .expect("RequestStack file");
    let request_stack_out = analyze_file(&db, request_stack);
    assert_eq!(
        request_stack_out.issues.as_ref(),
        &[
            Issue::new(
                IssueKind::MixedAssignment {
                    var: "resetRequestFormats".to_string(),
                },
                Location::new(fx.request_stack.clone(), 121, 121, 8, 108),
            )
            .with_snippet(
                "$resetRequestFormats ??= \\Closure::bind(static fn () => self::$formats = null, null, Request::class)",
            ),
            Issue::new(
                IssueKind::MixedFunctionCall,
                Location::new(fx.request_stack.clone(), 122, 122, 8, 30),
            )
            .with_snippet("$resetRequestFormats()"),
        ],
        "RequestStack false positives should stay stable until the underlying analysis improves"
    );

    let uri_signer = db
        .lookup_source_file(fx.uri_signer.as_ref())
        .expect("UriSigner file");
    let uri_out = analyze_file(&db, uri_signer);
    assert!(
        uri_out.issues.contains(
            &Issue::new(
                IssueKind::UndefinedClass {
                    name: "Psr\\Clock\\ClockInterface".to_string(),
                },
                Location::new(fx.uri_signer.clone(), 42, 42, 17, 31),
            )
            .with_snippet("ClockInterface")
        ),
        "UriSigner currently reports the missing external ClockInterface dependency"
    );
    assert!(
        uri_out.issues.contains(
            &Issue::new(
                IssueKind::UndefinedFunction {
                    name: "trigger_deprecation".to_string(),
                },
                Location::new(fx.uri_signer.clone(), 75, 75, 12, 279),
            )
            .with_snippet("trigger_deprecation('symfony/http-foundation', '8.2', 'Not passing an expiration to \"%s::sign()\" is deprecated and will be required in 9.0; pass one explicitly, or set a default via the \"$defaultExpiration\" argument of \"%s::__construct()\".', self::class, self::class)")
        ),
        "UriSigner currently reports the missing external trigger_deprecation helper"
    );

    let ascii_slugger = db
        .lookup_source_file(fx.ascii_slugger.as_ref())
        .expect("AsciiSlugger file");
    let ascii_out = analyze_file(&db, ascii_slugger);
    assert!(
        ascii_out.issues.contains(
            &Issue::new(
                IssueKind::UndefinedClass {
                    name: "Symfony\\Contracts\\Translation\\LocaleAwareInterface".to_string(),
                },
                Location::new(fx.ascii_slugger.clone(), 26, 26, 48, 68),
            )
            .with_snippet("LocaleAwareInterface")
        ),
        "AsciiSlugger currently reports the missing external LocaleAwareInterface dependency"
    );

    let router = db
        .lookup_source_file(fx.router.as_ref())
        .expect("Router file");
    let router_out = analyze_file(&db, router);
    assert!(
        router_out.issues.contains(
            &Issue::new(
                IssueKind::UndefinedClass {
                    name: "Psr\\Log\\LoggerInterface".to_string(),
                },
                Location::new(fx.router.clone(), 60, 60, 19, 34),
            )
            .with_snippet("LoggerInterface")
        ),
        "Router currently reports the missing external LoggerInterface dependency"
    );

    let parameter_bag = db
        .lookup_source_file(fx.parameter_bag.as_ref())
        .expect("ParameterBag file");
    let parameter_bag_out = analyze_file(&db, parameter_bag);
    assert!(
        parameter_bag_out.issues.contains(
            &Issue::new(
                IssueKind::UndefinedClass {
                    name: "T".to_string(),
                },
                Location::new(fx.parameter_bag.clone(), 200, 200, 19, 25),
            )
            .with_snippet("$class")
        ),
        "ParameterBag currently reports a template-docblock false positive for T"
    );
}
