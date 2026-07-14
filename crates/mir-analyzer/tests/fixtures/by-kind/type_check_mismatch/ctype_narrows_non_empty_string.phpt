===description===
ctype_*() truthy result narrows a string argument to non-empty-string, since
every ctype_*() function returns false on the empty string.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyInvalidArgument
===file===
<?php
function test_ctype_digit(string $s): void {
    if (ctype_digit($s)) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_ctype_alpha(string $s): void {
    if (ctype_alpha($s)) {
        /** @mir-check $s is non-empty-string */
        $_ = $s;
    }
}

function test_false_branch_not_narrowed(string $s): void {
    if (ctype_digit($s)) {
        return;
    }
    /** @mir-check $s is string */
    $_ = $s;
}

/** @param int|string $x */
function test_int_atom_untouched(mixed $x): void {
    if (ctype_digit($x)) {
        // ctype_digit(int) checks the ASCII/byte range, not decimal digits —
        // only the string atom is narrowed, int is passed through unchanged.
        /** @mir-check $x is int|non-empty-string */
        $_ = $x;
    }
}
===expect===
