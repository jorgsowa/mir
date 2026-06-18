===description===
array_fill() with a positive count returns a non-empty list.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_literal_count(): void {
    $arr = array_fill(0, 3, 'x');
    /** @mir-check $arr is non-empty-list<'x'> */
    $_ = $arr;
}

/** @param positive-int $n */
function test_positive_int_count(int $n): void {
    $arr = array_fill(0, $n, 42);
    /** @mir-check $arr is non-empty-list<42> */
    $_ = $arr;
}

/** @param int $n */
function test_plain_int_count(int $n): void {
    $arr = array_fill(0, $n, 'v');
    // When count is a plain int (could be 0), falls back to stub's array type.
    /** @mir-check $arr is array<mixed, mixed> */
    $_ = $arr;
}
===expect===
