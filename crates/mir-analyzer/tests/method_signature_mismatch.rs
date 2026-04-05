// crates/mir-analyzer/tests/method_signature_mismatch.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_override_narrowing_param_type() {
    // Parent accepts string; Child accepts only int — narrowing is not allowed
    let src = "<?php\nclass Base {\n    public function f(string $x): void {}\n}\nclass Child extends Base {\n    public function f(int $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn reports_override_widening_return_type() {
    // Parent returns int; Child returns int|string — widening return is not allowed
    let src = "<?php\nclass Base {\n    public function f(): int { return 1; }\n}\nclass Child extends Base {\n    public function f(): int|string { return 1; }\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_compatible_override() {
    let src = "<?php\nclass Base {\n    public function f(string $x): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn reports_override_adds_required_param() {
    // Parent has 0 params; Child has 1 required param
    let src = "<?php\nclass Base {\n    public function f(): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_override_with_optional_extra_param() {
    // Extra param with default is allowed
    let src = "<?php\nclass Base {\n    public function f(): void {}\n}\nclass Child extends Base {\n    public function f(string $x = 'default'): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn reports_override_removes_default() {
    // Parent has optional param ($x with default); Child makes it required — fires
    let src = "<?php\nclass Base {\n    public function f(string $x = 'hi'): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn reports_interface_implementation_wrong_signature() {
    let src = "<?php\ninterface I {\n    public function f(string $x): void;\n}\nclass C implements I {\n    public function f(int $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_correct_interface_implementation() {
    let src = "<?php\ninterface I {\n    public function f(string $x): void;\n}\nclass C implements I {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn does_not_report_correct_abstract_implementation() {
    let src = "<?php\nabstract class Base {\n    abstract public function f(string $x): void;\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn does_not_report_override_widening_param_type() {
    // Parent accepts string; Child accepts string|int — widening is allowed
    let src = "<?php\nclass Base {\n    public function f(string $x): void {}\n}\nclass Child extends Base {\n    public function f(string|int $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn does_not_report_override_no_type_hint_on_parent() {
    // Parent has no type hint; Child adds one — no enforcement possible
    let src = "<?php\nclass Base {\n    public function f($x): void {}\n}\nclass Child extends Base {\n    public function f(int $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn reports_override_narrowing_second_param() {
    // First param matches; second param narrows — should still fire
    let src = "<?php\nclass Base {\n    public function f(string $x, string $y): void {}\n}\nclass Child extends Base {\n    public function f(string $x, int $y): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}
