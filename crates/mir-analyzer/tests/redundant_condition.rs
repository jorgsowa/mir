// crates/mir-analyzer/tests/redundant_condition.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_null_check_on_non_nullable() {
    // $x is string — checking === null is always false
    let src = "<?php\nfunction f(string $x): void {\n    if ($x === null) {}\n}\n";
    let issues = check(src);
    // condition `$x === null` starts at col 8 (after "    if (")
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn reports_not_null_check_on_non_nullable() {
    let src = "<?php\nfunction f(string $x): void {\n    if ($x !== null) {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn does_not_report_null_check_on_nullable() {
    let src = "<?php\nfunction f(?string $x): void {\n    if ($x === null) {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "RedundantCondition");
}

#[test]
fn reports_is_string_on_string_type() {
    let src = "<?php\nfunction f(string $x): void {\n    if (is_string($x)) {}\n}\n";
    let issues = check(src);
    // is_string($x) starts at col 8
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn does_not_report_is_string_on_union() {
    let src = "<?php\nfunction f(string|int $x): void {\n    if (is_string($x)) {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "RedundantCondition");
}

#[test]
fn reports_redundant_check_after_narrowing() {
    // After $x is narrowed to string in first branch, second check is redundant
    let src = "<?php\nfunction f(string|int $x): void {\n    if (is_string($x)) {\n        if (is_string($x)) {}\n    }\n}\n";
    let issues = check(src);
    // inner is_string($x) at line 4, col 12
    assert_issue_kind(&issues, "RedundantCondition", 4, 12);
}
