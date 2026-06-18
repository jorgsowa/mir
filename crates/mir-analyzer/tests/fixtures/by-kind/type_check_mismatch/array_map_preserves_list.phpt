===description===
array_map() on a list returns a list; on a non-empty-list returns a non-empty-list.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<string> $strs */
function test_list(array $strs): void {
    $result = array_map('strtoupper', $strs);
    /** @mir-check $result is list<string> */
    $_ = $result;
}

/** @param non-empty-list<string> $strs */
function test_non_empty_list(array $strs): void {
    $result = array_map('strtoupper', $strs);
    /** @mir-check $result is non-empty-list<string> */
    $_ = $result;
}
===expect===
