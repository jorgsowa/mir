// crates/mir-analyzer/tests/undefined_class.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_new_unknown_class() {
    // new inside function body — analyzed in Pass 2
    let src = "<?php\nfunction test(): void {\n    new UnknownClass();\n}\n";
    let issues = check(src);
    // "    new " = 8 chars → UnknownClass at col 8
    assert_issue_kind(&issues, "UndefinedClass", 3, 8);
}

#[test]
fn does_not_report_stdclass() {
    // stdClass is a built-in PHP class — must not fire
    let src = "<?php\nfunction test(): void {\n    new stdClass();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn does_not_report_user_defined_class() {
    // User defines the class in the same file — no issue expected
    let src = "<?php\nclass MyClass {}\nfunction test(): void {\n    new MyClass();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn reports_unknown_class_in_param_type_hint() {
    // Type hints are checked in Pass 1
    let src = "<?php\nfunction f(UnknownClass $x): void {}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedClass", 2, 11);
}

#[test]
fn reports_unknown_class_in_return_type_hint() {
    // Return type hints are checked in Pass 1
    let src = "<?php\nfunction f(): UnknownClass {\n    return null;\n}\n";
    let issues = check(src);
    // Check for at least the return type hint issue on line 2 (col 14 = after "function f(): ")
    assert_issue_kind(&issues, "UndefinedClass", 2, 14);
}

#[test]
fn reports_extension_class_via_use_alias() {
    // `use ast\Node` where `ast\Node` does not exist in codebase
    let src = "<?php\nuse ast\\Node;\nfunction f(Node $x): void {}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedClass", 3, 11);
}

#[test]
fn does_not_report_known_aliased_class() {
    // User defines Bar, then aliases it as Baz — no issue expected
    let src = "<?php\nclass Bar {}\nuse Bar as Baz;\nfunction f(Baz $x): void {}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn reports_instanceof_unknown_class() {
    let src = "<?php\nfunction test($x): bool {\n    return $x instanceof NoSuchClass;\n}\n";
    let issues = check(src);
    // "    return $x instanceof " = 25 chars → NoSuchClass starts at col 25
    assert_issue_kind(&issues, "UndefinedClass", 3, 25);
}

#[test]
fn does_not_report_after_suppression() {
    // @psalm-suppress UndefinedClass on the expression should suppress the issue
    let src = "<?php\nfunction test(): void {\n    /**\n     * @psalm-suppress UndefinedClass\n     */\n    new NoSuchClass();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}
