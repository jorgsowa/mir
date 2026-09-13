===description===
Null comparisons on list and non-empty array reads are allowed when the offset may be absent.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<string> $list */
function testList(array $list): void {
    $v = $list[3];
    if ($v !== null) {}
}

/** @param non-empty-array<int, string> $map */
function testNonEmptyArray(array $map): void {
    $v = $map[3];
    if ($v !== null) {}
}

/** @param non-empty-list<string> $list */
function testNonEmptyList(array $list): void {
    $v = $list[3];
    if ($v !== null) {}
}
===expect===
