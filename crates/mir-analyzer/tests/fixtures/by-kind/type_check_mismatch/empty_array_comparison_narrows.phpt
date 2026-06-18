===description===
$arr !== [] narrows $arr to non-empty in the true branch (and === [] narrows to non-empty in the false branch).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<int> $arr */
function test_not_equal_narrows(array $arr): void {
    if ($arr !== []) {
        /** @mir-check $arr is non-empty-list<int> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_equal_else_narrows(array $arr): void {
    if ($arr === []) {
        // empty branch — no check needed
    } else {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}
===expect===
