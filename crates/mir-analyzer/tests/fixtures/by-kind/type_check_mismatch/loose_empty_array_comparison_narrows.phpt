===description===
$arr == [] / != [] narrow array emptiness just like the strict === []/
!== [] case — loose array equality requires identical key/value pairs, so
it's exactly as sound.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<int> $arr */
function test_not_equal_narrows(array $arr): void {
    if ($arr != []) {
        /** @mir-check $arr is non-empty-list<int> */
        $_ = $arr;
    }
}

/** @param list<string> $arr */
function test_equal_else_narrows(array $arr): void {
    if ($arr == []) {
        // empty branch — no check needed
    } else {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}

/** @param list<int>|non-empty-array<string, int> $arr */
function test_equal_narrows(array $arr): void {
    if ($arr == []) {
        /** @mir-check $arr is array{} */
        $_ = $arr;
    }
}
===expect===
