===description===
is_numeric on a literal-string "123" is always true (numeric literal string eliminated from false branch);
is_numeric on "hello" is always false (non-numeric literal string eliminated from true branch)
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_always_true(string $s): void {
    $s = "123";
    if (is_numeric($s)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        // always taken
    }
}
function test_always_false(string $s): void {
    $s = "hello";
    if (is_numeric($s)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        // never taken
    }
}
===expect===
