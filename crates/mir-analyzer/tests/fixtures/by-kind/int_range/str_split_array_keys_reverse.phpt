===description===
str_split() on a non-empty string returns non-empty-list<non-empty-string>.
array_keys() on a non-empty array returns a non-empty list.
array_reverse() preserves non-emptiness.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string $s */
function test_str_split(string $s): void {
    $parts = str_split($s);
    /** @mir-check $parts is non-empty-list<non-empty-string> */
    $_ = $parts;
}

/** @param non-empty-list<string> $arr */
function test_array_keys(array $arr): void {
    $keys = array_keys($arr);
    // A list's keys are always sequential integers, so TKey now binds to
    // int (not its int|string bound) once the list shape itself is matched.
    /** @mir-check $keys is non-empty-list<int> */
    $_ = $keys;
}

/** @param non-empty-list<string> $arr */
function test_array_reverse(array $arr): void {
    $rev = array_reverse($arr);
    /** @mir-check $rev is non-empty-list<string> */
    $_ = $rev;
}
===expect===
