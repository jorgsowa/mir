===description===
array_key_first/array_key_last return int|string (never null) for non-empty collections;
int-only for list inputs.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-list<string> $list */
function test_list_key_first(array $list): void {
    $k = array_key_first($list);
    /** @mir-check $k is int */
    $_ = $k;
}

/** @param non-empty-list<string> $list */
function test_list_key_last(array $list): void {
    $k = array_key_last($list);
    /** @mir-check $k is int */
    $_ = $k;
}

/** @param non-empty-array<string, int> $map */
function test_map_key_first(array $map): void {
    $k = array_key_first($map);
    /** @mir-check $k is int|string */
    $_ = $k;
}

/** @param non-empty-array<string, int> $map */
function test_map_key_last(array $map): void {
    $k = array_key_last($map);
    /** @mir-check $k is int|string */
    $_ = $k;
}
===expect===
