===description===
array_is_list() narrows the argument to a list type in the true branch.
===config===
php_version=8.1
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param array<int, string> $arr */
function test_int_keyed(array $arr): void {
    if (array_is_list($arr)) {
        /** @mir-check $arr is list<string> */
        $_ = $arr;
    }
}

/** @param non-empty-array<int, string> $arr */
function test_non_empty_int_keyed(array $arr): void {
    if (array_is_list($arr)) {
        /** @mir-check $arr is non-empty-list<string> */
        $_ = $arr;
    }
}
===expect===
