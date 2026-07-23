===description===
array_column, scoped to a single resolvable row shape and literal string/int
$column_key: with no $index_key, pulls a column into a fresh 0-indexed list;
with an $index_key naming another property, re-keys the result by that
property's value instead. `$column_key === null` (whole rows) keeps the full
row shape as the value, with no column-presence exclusion of its own.
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

    $whole_rows = array_column($rows, null);
    /** @mir-check $whole_rows is non-empty-list<array{'id': int, 'name': string, 'active': bool}> */
    $_ = $whole_rows;

    $whole_rows_by_id = array_column($rows, null, 'id');
    /** @mir-check $whole_rows_by_id is non-empty-array<int, array{'id': int, 'name': string, 'active': bool}> */
    $_ = $whole_rows_by_id;
}
===expect===
