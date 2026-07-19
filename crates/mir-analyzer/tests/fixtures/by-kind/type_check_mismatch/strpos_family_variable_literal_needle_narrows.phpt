===description===
strpos()/mb_strpos() family recognizes a variable holding a single non-empty
string literal as the needle, not just an inline literal — mirrors
array_key_exists()'s handling of a variable-held literal key.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_strpos_not_false_variable_needle(string $s): void {
    $needle = 'x';
    if (strpos($s, $needle) !== false) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_empty_variable_needle_not_narrowed(string $s): void {
    $needle = '';
    if (strpos($s, $needle) !== false) {
        // Empty needle is always found, so we can't narrow the haystack.
        /** @mir-check $s is string */
        $_ = $s;
    }
}

function test_unresolved_variable_needle_not_narrowed(string $s, string $needle): void {
    // $needle isn't narrowed to a single literal — must not assume non-empty.
    if (strpos($s, $needle) !== false) {
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
