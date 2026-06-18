===description===
array_fill_keys uses the values of $keys as result keys and $value as each result value; non-empty $keys produces a non-empty result.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param non-empty-list<string> $keys */
function test_non_empty_string_keys(array $keys, int $value): void {
    $result = array_fill_keys($keys, $value);
    /** @mir-check $result is non-empty-array<string, int> */
    $_ = $result;
}

/** @param list<string> $keys */
function test_possibly_empty_keys(array $keys, bool $value): void {
    $result = array_fill_keys($keys, $value);
    /** @mir-check $result is array<string, bool> */
    $_ = $result;
}
===expect===
