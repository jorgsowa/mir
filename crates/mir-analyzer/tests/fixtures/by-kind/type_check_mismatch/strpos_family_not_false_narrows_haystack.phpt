===description===
strpos()/stripos()/strrpos()/strripos() (and mb_ variants) compared with
!== false / === false narrow the haystack like str_contains() does.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_strpos_not_false(string $s): void {
    if (strpos($s, 'x') !== false) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_false_not_strpos(string $s): void {
    if (false !== strpos($s, 'x')) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_mb_stripos_not_false(string $s): void {
    if (mb_stripos($s, 'x') !== false) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_strrpos_equal_false_not_narrowed(string $s): void {
    if (strrpos($s, 'x') === false) {
        // Not found tells us nothing about $s itself.
        /** @mir-check $s is string */
        $_ = $s;
    }
}

function test_empty_needle_not_narrowed(string $s): void {
    if (strpos($s, '') !== false) {
        // Empty needle is always found, so we can't narrow the haystack.
        /** @mir-check $s is string */
        $_ = $s;
    }
}
===expect===
