===description===
array_search narrows the return key type from the haystack: list → int|false, array<string,T> → string|false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_search_list(array $arr, string $needle): void {
    $key = array_search($needle, $arr);
    /** @mir-check $key is int|false */
    $_ = $key;
}

/** @param array<string, int> $arr */
function test_search_string_keyed(array $arr, int $needle): void {
    $key = array_search($needle, $arr);
    /** @mir-check $key is string|false */
    $_ = $key;
}
===expect===
