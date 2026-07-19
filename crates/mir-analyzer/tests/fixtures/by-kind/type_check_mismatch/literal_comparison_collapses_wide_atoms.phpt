===description===
`$x === 'foo'`/`$x === 5` collapses a wide `string|int|mixed`/`scalar`/
`mixed` atom to the exact literal, not just the narrower named subtypes
(TNonEmptyString, TIntRange, ...) — every atom in the union can be proven to
be exactly that literal, the same reasoning the narrower siblings already use.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function narrowsWideStringToLiteral(string $x): void {
    if ($x === 'foo') {
        /** @mir-check $x is 'foo' */
        $_ = $x;
    }
}

/** @param scalar $x */
function narrowsScalarToStringLiteral($x): void {
    if ($x === 'foo') {
        /** @mir-check $x is 'foo' */
        $_ = $x;
    }
}

function narrowsMixedToStringLiteral(mixed $x): void {
    if ($x === 'foo') {
        /** @mir-check $x is 'foo' */
        $_ = $x;
    }
}

function narrowsWideIntToLiteral(int $x): void {
    if ($x === 5) {
        /** @mir-check $x is 5 */
        $_ = $x;
    }
}

/** @param scalar $x */
function narrowsScalarToIntLiteral($x): void {
    if ($x === 5) {
        /** @mir-check $x is 5 */
        $_ = $x;
    }
}

function narrowsMixedToIntLiteral(mixed $x): void {
    if ($x === 5) {
        /** @mir-check $x is 5 */
        $_ = $x;
    }
}
===expect===
