===description===
key($array) narrows to the array's own key type (plus null, since the
internal pointer's position isn't tracked) instead of the stub's unrefined
int|string|null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param list<string> $arr */
function test_key_list_is_int(array $arr): void {
    /** @mir-check key($arr) is int|null */
    $_ = key($arr);
}

/** @param array<string, int> $arr */
function test_key_string_keyed(array $arr): void {
    /** @mir-check key($arr) is string|null */
    $_ = key($arr);
}

/** @param non-empty-list<string> $arr */
function test_key_non_empty_still_includes_null(array $arr): void {
    /** @mir-check key($arr) is int|null */
    $_ = key($arr);
}

/** @param array{a: int, 0: string} $arr */
function test_key_mixed_keys_fallback(array $arr): void {
    /** @mir-check key($arr) is string|int|null */
    $_ = key($arr);
}
===expect===
