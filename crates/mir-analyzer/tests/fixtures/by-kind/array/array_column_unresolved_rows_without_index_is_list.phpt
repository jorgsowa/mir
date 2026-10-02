===description===
array_column without $index_key is a list even when the row shape or column is unresolvable;
array_map over it stays a list.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $rows, array $objects, string $col): void {
    $plain = array_column($rows, 'id');
    /** @mir-check $plain is list<mixed> */
    $_ = $plain;

    $dynamic = array_column($rows, $col);
    /** @mir-check $dynamic is list<mixed> */
    $_ = $dynamic;

    $null_index = array_column($objects, 'id', null);
    /** @mir-check $null_index is list<mixed> */
    $_ = $null_index;

    $ids = array_map(intval(...), array_column($rows, 'id'));
    /** @mir-check $ids is list<int> */
    $_ = $ids;

    $ids_str = array_map('intval', array_column($rows, 'id'));
    /** @mir-check $ids_str is list<int> */
    $_ = $ids_str;
}
===expect===
