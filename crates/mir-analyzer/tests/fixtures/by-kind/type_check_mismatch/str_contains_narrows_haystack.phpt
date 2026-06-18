===description===
str_contains/str_starts_with/str_ends_with with a non-empty literal needle narrows
the haystack to non-empty-string in the true branch.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_str_contains(string $s): void {
    if (str_contains($s, 'x')) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_str_starts_with(string $s): void {
    if (str_starts_with($s, 'prefix')) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_str_ends_with(string $s): void {
    if (str_ends_with($s, 'suffix')) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_empty_needle_not_narrowed(string $s): void {
    if (str_contains($s, '')) {
        // Empty needle always returns true, so we can't narrow the haystack.
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
