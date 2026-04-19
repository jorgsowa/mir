use mir_analyzer::test_utils::check;
use mir_issues::IssueKind;

/// Verify that both parts of Iterator&Countable are preserved in the expected
/// type shown in the InvalidArgument message — not just the first part.
#[test]
fn intersection_error_message_contains_both_parts() {
    let issues = check(
        r#"<?php
interface Iterator {}
interface Countable {}
function f(Iterator&Countable $x): void { $_ = $x; }
function test(): void { f("hello"); }
"#,
    );
    assert_eq!(issues.len(), 1);
    let IssueKind::InvalidArgument { expected, .. } = &issues[0].kind else {
        panic!("expected InvalidArgument, got {:?}", issues[0].kind);
    };
    assert_eq!(expected, "Iterator&Countable");
}

/// Same as above but via docblock annotation.
#[test]
fn docblock_intersection_error_message_contains_both_parts() {
    let issues = check(
        r#"<?php
interface Iterator {}
interface Countable {}
/** @param Iterator&Countable $x */
function f($x): void { $_ = $x; }
function test(): void { f(42); }
"#,
    );
    assert_eq!(issues.len(), 1);
    let IssueKind::InvalidArgument { expected, .. } = &issues[0].kind else {
        panic!("expected InvalidArgument, got {:?}", issues[0].kind);
    };
    assert_eq!(expected, "Iterator&Countable");
}

/// Three-part intersection: all three names must appear in the expected type.
#[test]
fn three_part_intersection_error_message_contains_all_parts() {
    let issues = check(
        r#"<?php
interface Iterator {}
interface Countable {}
interface Stringable {}
function f(Iterator&Countable&Stringable $x): void { $_ = $x; }
function test(): void { f("hello"); }
"#,
    );
    assert_eq!(issues.len(), 1);
    let IssueKind::InvalidArgument { expected, .. } = &issues[0].kind else {
        panic!("expected InvalidArgument, got {:?}", issues[0].kind);
    };
    assert_eq!(expected, "Iterator&Countable&Stringable");
}

/// Nullable intersection: null is accepted, but a scalar is not.
#[test]
fn nullable_docblock_intersection_accepts_null_rejects_scalar() {
    let issues_null = check(
        r#"<?php
interface Iterator {}
interface Countable {}
/** @param Iterator&Countable|null $x */
function f($x): void { $_ = $x; }
function test(): void { f(null); }
"#,
    );
    assert!(
        issues_null.is_empty(),
        "null should be accepted: {:?}",
        issues_null
    );

    let issues_scalar = check(
        r#"<?php
interface Iterator {}
interface Countable {}
/** @param Iterator&Countable|null $x */
function f($x): void { $_ = $x; }
function test(): void { f("hello"); }
"#,
    );
    assert_eq!(issues_scalar.len(), 1);
    assert!(matches!(
        issues_scalar[0].kind,
        IssueKind::InvalidArgument { .. }
    ));
}
