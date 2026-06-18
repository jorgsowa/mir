===description===
array_push and array_unshift update the by-ref array to a non-empty type with the pushed element types merged in.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_push_same_type(array $arr, string $val): void {
    array_push($arr, $val);
    /** @mir-check $arr is non-empty-list<string> */
    $_ = $arr;
}

/** @param list<string> $arr */
function test_push_widens_value_type(array $arr, int $extra): void {
    array_push($arr, $extra);
    /** @mir-check $arr is non-empty-list<string|int> */
    $_ = $arr;
}

/** @param list<string> $arr */
function test_unshift_same_type(array $arr, string $val): void {
    array_unshift($arr, $val);
    /** @mir-check $arr is non-empty-list<string> */
    $_ = $arr;
}

/** @param non-empty-list<int> $arr */
function test_push_already_non_empty(array $arr, int $val): void {
    array_push($arr, $val);
    /** @mir-check $arr is non-empty-list<int> */
    $_ = $arr;
}
===expect===
