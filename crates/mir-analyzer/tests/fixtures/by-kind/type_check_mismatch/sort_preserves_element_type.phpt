===description===
In-place sort functions preserve element types; re-indexing sorts (sort/rsort/usort/shuffle) convert to list; key-preserving sorts (asort/arsort/ksort/krsort/uasort/uksort) keep the original type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<int> $arr */
function test_sort_list(array $arr): void {
    sort($arr);
    /** @mir-check $arr is list<int> */
    $_ = $arr;
}

/** @param non-empty-list<string> $arr */
function test_rsort_non_empty_list(array $arr): void {
    rsort($arr);
    /** @mir-check $arr is non-empty-list<string> */
    $_ = $arr;
}

/** @param list<int> $arr */
function test_usort_list(array $arr): void {
    usort($arr, fn($a, $b) => $a - $b);
    /** @mir-check $arr is list<int> */
    $_ = $arr;
}

/** @param list<string> $arr */
function test_asort_preserves_type(array $arr): void {
    asort($arr);
    /** @mir-check $arr is list<string> */
    $_ = $arr;
}

/** @param non-empty-list<float> $arr */
function test_ksort_non_empty(array $arr): void {
    ksort($arr);
    /** @mir-check $arr is non-empty-list<float> */
    $_ = $arr;
}
===expect===
