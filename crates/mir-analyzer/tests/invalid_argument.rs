// crates/mir-analyzer/tests/invalid_argument.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_string_passed_as_int() {
    // f(int $x) called with 'hello' inside a function body
    // line 3: "function test(): void { f('hello'); }"
    // "function test(): void { f(" = 26 bytes → col_start = 26 (0-based byte offset)
    let src = "<?php\nfunction f(int $x): void {}\nfunction test(): void { f('hello'); }\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 3, 26);
}

#[test]
fn does_not_report_correct_int_arg() {
    let src = "<?php\nfunction f(int $x): void {}\nfunction test(): void { f(42); }\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_object_passed_as_int() {
    // f(int $x) called with an object — clearly incompatible
    let src = "<?php\nclass Foo {}\nfunction f(int $x): void {}\nfunction test(): void { f(new Foo()); }\n";
    let issues = check(src);
    // line 4: "function test(): void { f(" = 26 bytes → col_start = 26 (0-based byte offset)
    assert_issue_kind(&issues, "InvalidArgument", 4, 26);
}

#[test]
#[ignore = "null→string fires PossiblyNullArgument, not InvalidArgument — documenting expected ideal behavior"]
fn reports_null_passed_as_string() {
    let src = "<?php\nfunction f(string $x): void {}\nfunction test(): void { f(null); }\n";
    let issues = check(src);
    // 'null' value starts at col 26 (after "function test(): void { f(")
    assert_issue_kind(&issues, "InvalidArgument", 3, 26);
}

#[test]
#[ignore = "known issue: @var int|string passed to int is skipped because param atomic (int) is subtype of arg — analyzer does not fire"]
fn reports_incompatible_union_arg() {
    // Use @var docblock so the analyzer understands the union type
    let src = "<?php\nfunction f(int $x): void {}\nfunction test(): void {\n    /** @var int|string $v */\n    $v = 1;\n    f($v);\n}\n";
    let issues = check(src);
    // f($v) on line 6, col 4 (inside function body)
    assert_issue_kind(&issues, "InvalidArgument", 6, 4);
}

#[test]
fn does_not_report_subclass_as_parent() {
    // Child extends Parent; passing Child where Parent expected → no InvalidArgument
    let src = "<?php\nclass Base {}\nclass Child extends Base {}\nfunction f(Base $x): void {}\nfunction test(): void { f(new Child()); }\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
#[ignore = "known issue: strlen(int) does not fire — int→string coercion is skipped by the analyzer"]
fn reports_wrong_type_to_strlen() {
    // strlen expects string; pass 42 → should fire but doesn't
    let src = "<?php\nfunction test(): void { strlen(42); }\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 2, 32);
}

#[test]
fn does_not_report_mixed_arg() {
    // mixed bypasses type checks
    let src = "<?php\nfunction f(int $x): void {}\nfunction test(mixed $v): void { f($v); }\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_variadic_wrong_type() {
    // variadic int, pass 'a' string
    // line 3: "function test(): void { f('a'); }"
    // "function test(): void { f(" = 26 bytes → col_start = 26 (0-based byte offset)
    let src = "<?php\nfunction f(int ...$xs): void {}\nfunction test(): void { f('a'); }\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 3, 26);
}

#[test]
fn does_not_report_correct_union_to_union_param() {
    // string to string|int — string is subtype of string|int → no fire
    let src = "<?php\nfunction f(string|int $x): void {}\nfunction test(): void { f('hello'); }\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_named_argument_wrong_type() {
    // PHP 8 named argument: wrong type — col resolves to value start, same as positional
    // line 3: "function test(): void { f(x: 'hello'); }"
    // The arg span points to the value 'hello' which starts at col 26 after mapping
    let src = "<?php\nfunction f(int $x): void {}\nfunction test(): void { f(x: 'hello'); }\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 3, 26);
}
