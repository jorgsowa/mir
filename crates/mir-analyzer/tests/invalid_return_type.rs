// crates/mir-analyzer/tests/invalid_return_type.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_string_returned_from_int_function() {
    // return 'hello' from function declared as int
    // line 3: "    return 'hello';" — col 4 (0-indexed, "    " = 4 spaces)
    let src = "<?php\nfunction f(): int {\n    return 'hello';\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn does_not_report_correct_return_type() {
    let src = "<?php\nfunction f(): int {\n    return 42;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn reports_null_returned_from_non_nullable() {
    // return null from function declared as string (non-nullable) — should fire but doesn't
    let src = "<?php\nfunction f(): string {\n    return null;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn reports_bare_return_from_non_void() {
    // bare `return;` from int function — returns void but declared int
    let src = "<?php\nfunction f(): int {\n    return;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn does_not_report_subclass_return() {
    // return Child from function declared as Base → no InvalidReturnType
    let src = "<?php\nclass Base {}\nclass Child extends Base {}\nfunction f(): Base {\n    return new Child();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn does_not_report_mixed_return() {
    // return mixed from int declared — mixed bypasses checks
    let src = "<?php\nfunction f(): int {\n    $x = json_decode('{}');\n    return $x;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn reports_return_null_from_void() {
    // `return null;` from void function should fire
    let src = "<?php\nfunction f(): void {\n    return null;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn reports_wrong_union_return() {
    // return int|string from function declared as int — should fire
    // line 4: "    return $x;" — col 4 (0-indexed, "    " = 4 spaces)
    let src = "<?php\nfunction f(): int {\n    $x = true ? 1 : 'hello';\n    return $x;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 4, 4);
}
