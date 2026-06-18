===description===
array_slice preserves element type; list source without preserve_keys returns list.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $items */
function test_list_slice(array $items): void {
    $r = array_slice($items, 1);
    /** @mir-check $r is list<string> */
    $_ = $r;
}

/** @param list<string> $items */
function test_list_slice_with_length(array $items): void {
    $r = array_slice($items, 0, 3);
    /** @mir-check $r is list<string> */
    $_ = $r;
}

/** @param array<string, int> $map */
function test_assoc_slice(array $map): void {
    $r = array_slice($map, 1);
    /** @mir-check $r is array<string, int> */
    $_ = $r;
}
===expect===
