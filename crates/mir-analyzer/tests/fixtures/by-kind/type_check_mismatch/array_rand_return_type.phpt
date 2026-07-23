===description===
array_rand() narrows by a literal $num: omitted or 1 -> the array's key
type alone (PHP 8 throws on empty input, never returns false); a literal
count > 1 -> a non-empty-list of the key type. A non-literal $num falls
back to the stub.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-array<string, int> $arr */
function test_omitted_num(array $arr): void {
    /** @mir-check array_rand($arr) is string */
    $_ = array_rand($arr);
}

/** @param non-empty-list<int> $arr */
function test_literal_num_one(array $arr): void {
    /** @mir-check array_rand($arr, 1) is int */
    $_ = array_rand($arr, 1);
}

/** @param non-empty-array<string, int> $arr */
function test_literal_num_multiple(array $arr): void {
    /** @mir-check array_rand($arr, 2) is non-empty-list<string> */
    $_ = array_rand($arr, 2);
}

/** @param non-empty-array<string, int> $arr */
function test_non_literal_num_fallback(array $arr, int $num): void {
    /** @mir-check array_rand($arr, $num) is int|string|array */
    $_ = array_rand($arr, $num);
}
===expect===
