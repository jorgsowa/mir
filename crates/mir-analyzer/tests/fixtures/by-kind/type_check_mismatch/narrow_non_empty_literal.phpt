===description===
Narrowing non-empty-string with === 'literal' should yield 'literal' in true branch.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string $x */
function test_non_empty_literal_narrow(string $x): void {
    if ($x === 'foo') {
        /** @mir-check $x is 'foo' */
        $_ = $x;
    }
}

/** @param numeric-string $x */
function test_numeric_literal_narrow(string $x): void {
    if ($x === '42') {
        /** @mir-check $x is '42' */
        $_ = $x;
    }
}
===expect===
