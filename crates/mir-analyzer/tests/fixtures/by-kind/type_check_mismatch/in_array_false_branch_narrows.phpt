===description===
in_array false-branch narrows a finite literal-union needle by removing matched values from the union.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param "a"|"b"|"c"|"d" $mode */
function test_false_branch_removes_literals(string $mode): void {
    if (!in_array($mode, ['a', 'b'])) {
        /** @mir-check $mode is "c"|"d" */
        $_ = $mode;
    }
}

/** @param 1|2|3|4|5 $code */
function test_false_branch_int(int $code): void {
    if (!in_array($code, [1, 2, 3])) {
        /** @mir-check $code is 4|5 */
        $_ = $code;
    }
}
===expect===
