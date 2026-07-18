===description===
str_contains/str_starts_with/str_ends_with resolve a needle argument that's
a variable already narrowed to a single non-empty literal string, same as
passing the literal inline.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_str_contains_variable_needle(string $s): void {
    $needle = 'x';
    if (str_contains($s, $needle)) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_str_starts_with_variable_needle(string $s): void {
    $needle = 'prefix';
    if (str_starts_with($s, $needle)) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_str_ends_with_variable_needle(string $s): void {
    $needle = 'suffix';
    if (str_ends_with($s, $needle)) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_empty_variable_needle_not_narrowed(string $s): void {
    $needle = '';
    if (str_contains($s, $needle)) {
        // Empty needle always returns true, so we can't narrow the haystack.
        /** @mir-check $s is string */
        $_ = $s;
    }
}

function test_not_narrowed_when_needle_is_not_a_literal(string $s, string $needle): void {
    // $needle is an unnarrowed string, not a proven literal — must not be
    // treated as non-empty.
    if (str_contains($s, $needle)) {
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
