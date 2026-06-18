===description===
array_merge on list inputs returns a list; non-empty if any input is non-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $a
 *  @param list<int> $b */
function test_merge_lists(array $a, array $b): void {
    $r = array_merge($a, $b);
    /** @mir-check $r is list<string|int> */
    $_ = $r;
}

/** @param non-empty-list<string> $a
 *  @param list<int> $b */
function test_merge_non_empty_first(array $a, array $b): void {
    $r = array_merge($a, $b);
    /** @mir-check $r is non-empty-list<string|int> */
    $_ = $r;
}

/** @param list<string> $a
 *  @param non-empty-list<int> $b */
function test_merge_non_empty_second(array $a, array $b): void {
    $r = array_merge($a, $b);
    /** @mir-check $r is non-empty-list<string|int> */
    $_ = $r;
}
===expect===
