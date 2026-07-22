===description===
array_column, scoped to a single resolvable row shape and literal string/int
$column_key: with no $index_key, pulls a column into a fresh 0-indexed list;
with an $index_key naming another property, re-keys the result by that
property's value instead.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<array{id: int, name: string, active: bool}> $rows
 */
function test(array $rows): void {
    $names = array_column($rows, 'name');
    /** @mir-check $names is non-empty-list<string> */
    $_ = $names;

    $by_id = array_column($rows, 'name', 'id');
    /** @mir-check $by_id is non-empty-array<int, string> */
    $_ = $by_id;
}
===expect===
