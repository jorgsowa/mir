===description===
M15: excluding the empty string (`$x !== ''` / `$x === '' → return`) upgrades
a bare `string` to `non-empty-string`, both as a direct guard and via
`assert()`. A union member (`int|string`) also narrows its string atom,
covering the leaked-atom shape that surfaces as PossiblyInvalidArgument on a
real `int|non-empty-string`-typed callee.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int|non-empty-string $key */
function formatKey(int|string $key): void {}

function run(int|string $key): void {
    if ($key === '') {
        return;
    }
    formatKey($key);
}

function guard(string $x): void {
    if ($x === '') {
        return;
    }
    /** @mir-check $x is non-empty-string */
    $_ = $x;
}

function guardNegated(string $x): void {
    if ($x !== '') {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    }
}

function viaAssert(string $x): void {
    assert($x !== '');
    /** @mir-check $x is non-empty-string */
    $_ = $x;
}
===expect===
