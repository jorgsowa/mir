===description===
array_pop/array_shift return the value type (not mixed) for typed collections;
null-free for non-empty inputs.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-list<string> $list */
function test_pop_non_empty_list(array $list): void {
    $v = array_pop($list);
    /** @mir-check $v is string */
    $_ = $v;
}

/** @param non-empty-list<string> $list */
function test_shift_non_empty_list(array $list): void {
    $v = array_shift($list);
    /** @mir-check $v is string */
    $_ = $v;
}

/** @param list<int> $list */
function test_pop_possibly_empty_list(array $list): void {
    $v = array_pop($list);
    /** @mir-check $v is int|null */
    $_ = $v;
}

/** @param non-empty-array<string, int> $map */
function test_pop_non_empty_map(array $map): void {
    $v = array_pop($map);
    /** @mir-check $v is int */
    $_ = $v;
}
===expect===
