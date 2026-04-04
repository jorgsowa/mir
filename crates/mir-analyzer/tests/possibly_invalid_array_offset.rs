// crates/mir-analyzer/tests/possibly_invalid_array_offset.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_destructure_of_array_or_false() {
    // array|false return → has_non_array && has_array → fires PossiblyInvalidArrayOffset
    let src = "<?php\n/** @return array|false */\nfunction get(): array|false { return false; }\nfunction test(): void {\n    [$a, $b] = get();\n}\n";
    let issues = check(src);
    // [$a, $b] destructure LHS at line 5, col 4 (inside function body, indented 4 spaces)
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 5, 4);
}

#[test]
fn does_not_report_destructure_of_plain_array() {
    let src = "<?php\n/** @return array */\nfunction get(): array { return []; }\nfunction test(): void {\n    [$a, $b] = get();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
fn reports_destructure_of_only_false() {
    // false alone: has_non_array=true, has_array=false → rule does NOT fire
    // (no array type in union → PossiblyInvalidArrayOffset does NOT fire)
    let src = "<?php\nfunction test(): void {\n    $v = false;\n    [$a] = $v;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
fn reports_both_elements_of_multi_var_destructure() {
    // Both $a and $b should be in scope even when the issue fires
    let src = "<?php\n/** @return array|false */\nfunction get(): array|false { return false; }\nfunction test(): void {\n    [$a, $b] = get();\n    echo $a + $b;\n}\n";
    let issues = check(src);
    // [$a, $b] destructure at line 5, col 4
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 5, 4);
}

#[test]
fn does_not_report_after_false_check() {
    // if ($r !== false) { [$a] = $r; } — $r is narrowed to array in the if-branch
    let src = "<?php\n/** @return array|false */\nfunction get(): array|false { return false; }\nfunction test(): void {\n    $r = get();\n    if ($r !== false) {\n        [$a] = $r;\n    }\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
#[ignore = "known gap: unpack() stub missing — returns mixed instead of array|false, so PossiblyInvalidArrayOffset does not fire"]
fn does_not_report_unpack_result_when_stub_present() {
    // unpack() returns array|false; once stub is present this should fire PossiblyInvalidArrayOffset
    let src = "<?php\nfunction test(): void {\n    [$a] = unpack('N', pack('N', 1));\n}\n";
    let issues = check(src);
    // [$a] destructure at line 3, col 4
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 3, 4);
}

#[test]
fn does_not_report_plain_array_offset_access() {
    // $arr[0] is a direct offset access, not destructuring — should not fire
    let src = "<?php\nfunction test(): void {\n    $arr = [1, 2, 3];\n    $x = $arr[0];\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}
