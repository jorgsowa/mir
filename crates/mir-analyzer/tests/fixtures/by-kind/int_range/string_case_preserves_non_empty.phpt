===description===
strtolower, strtoupper, ucfirst, lcfirst, ucwords, mb_strtolower, mb_strtoupper
preserve non-empty-string when the input is provably non-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string $s */
function test_strtolower(string $s): void {
    $r = strtolower($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_strtoupper(string $s): void {
    $r = strtoupper($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_ucfirst(string $s): void {
    $r = ucfirst($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_plain_string(string $s): void {
    $r = strtolower($s);
    /** @mir-check $r is string */
    $_ = $r;
}
===expect===
