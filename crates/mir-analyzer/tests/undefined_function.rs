// crates/mir-analyzer/tests/undefined_function.rs
use mir_issues::IssueKind;
use mir_test_utils::{assert_issue, assert_no_issue, check};

#[test]
fn reports_unknown_function() {
    let issues = check("<?php\nfunction test(): void {\n    foo();\n}\n");
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        3,
        4,
    );
}

#[test]
fn does_not_report_strlen() {
    let issues = check("<?php\nstrlen('hello');\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_array_map() {
    let issues = check("<?php\narray_map(fn($x) => $x, [1, 2, 3]);\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_user_defined_function() {
    let issues = check("<?php\nfunction myFn(): void {}\nmyFn();\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn reports_global_namespace_unknown_function() {
    // Leading \ forces global namespace lookup; still unknown
    let issues = check("<?php\nfunction test(): void {\n    \\nonExistent();\n}\n");
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "nonExistent".into() },
        3,
        4,
    );
}

#[test]
fn does_not_report_unpack() {
    // unpack() is a PHP builtin — must be in stubs
    // NOTE: this test currently FAILS if unpack() stub is missing (see CLAUDE.md gap analysis)
    let issues = check("<?php\n$r = unpack('N*', pack('N*', 1));\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_suppressed_call() {
    let src = "<?php\n/** @psalm-suppress UndefinedFunction */\nnoSuchFunction();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn reports_inside_method_body() {
    let src = "<?php\nclass A {\n    public function go(): void {\n        missing();\n    }\n}\n";
    let issues = check(src);
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "missing".into() },
        4,
        8,
    );
}

#[test]
fn reports_each_call_site_independently() {
    let src = "<?php\nfunction test(): void {\n    foo();\n    foo();\n}\n";
    let issues = check(src);
    // Two separate call sites — one on line 3 and one on line 4
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        3,
        4,
    );
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        4,
        4,
    );
}
