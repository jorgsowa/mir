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
        // always taken
    }
}
function test_always_false(string $s): void {
    $s = "hello";
    if (is_numeric($s)) {
        // never taken
    }
}
===expect===
RedundantCondition@4:8-4:22: Condition is always true/false for type 'bool'
RedundantCondition@10:8-10:22: Condition is always true/false for type 'bool'
