===description===
array_unique preserves element types and non-empty status (keys may have gaps — not list).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-list<string> $items */
function test_non_empty_list(array $items): void {
    $r = array_unique($items);
    /** @mir-check $r is non-empty-array<int, string> */
    $_ = $r;
}

/** @param list<string> $items */
function test_possibly_empty_list(array $items): void {
    $r = array_unique($items);
    /** @mir-check $r is array<int, string> */
    $_ = $r;
}

/** @param non-empty-array<string, int> $map */
function test_non_empty_map(array $map): void {
    $r = array_unique($map);
    /** @mir-check $r is non-empty-array<string, int> */
    $_ = $r;
}
===expect===
