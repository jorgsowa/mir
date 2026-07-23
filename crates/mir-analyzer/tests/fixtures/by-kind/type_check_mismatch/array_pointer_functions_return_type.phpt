===description===
reset()/end() narrow to the array's value type (plus false); current()/
next()/prev() always include false too, even for a provably non-empty
source, since the pointer's position from prior calls isn't tracked.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-list<int> $arr */
function test_reset_non_empty(array $arr): void {
    /** @mir-check reset($arr) is int */
    $_ = reset($arr);
}

/** @param list<int> $arr */
function test_end_possibly_empty(array $arr): void {
    /** @mir-check end($arr) is int|false */
    $_ = end($arr);
}

/** @param non-empty-list<int> $arr */
function test_current_still_includes_false(array $arr): void {
    /** @mir-check current($arr) is int|false */
    $_ = current($arr);
}

/** @param non-empty-list<int> $arr */
function test_next_still_includes_false(array $arr): void {
    /** @mir-check next($arr) is int|false */
    $_ = next($arr);
}

/** @param non-empty-list<int> $arr */
function test_prev_still_includes_false(array $arr): void {
    /** @mir-check prev($arr) is int|false */
    $_ = prev($arr);
}
===expect===
