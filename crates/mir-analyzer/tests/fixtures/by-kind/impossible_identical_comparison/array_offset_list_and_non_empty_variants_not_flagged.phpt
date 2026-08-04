===description===
L3, `list<T>`/`non-empty-array<K,V>`/`non-empty-list<T>` variants: "non-empty"
only proves at least one element exists, not that a specific offset does —
every generic-array-shaped atom must get the same treatment as plain
`array<K,V>`.
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
