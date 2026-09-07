===description===
FALSE POSITIVE reproducer. PHP's `~`, `&`, `|`, `^` return `string` (byte-wise)
when all operands are `string`, not `int`. mir previously hardcoded `int` for
these operators, so `bin2hex(~$bytes)` flagged a bogus PossiblyInvalidArgument.
`<<`/`>>` always coerce to int regardless of operand type, so they keep the
old int-only behavior.
===config===
suppress=UnusedParam
php_version=8.4
===file===
<?php

function bitwiseNotStringIntoStringParam(string $bytes): string {
    return bin2hex(~$bytes);
}

function bitwiseNotStringResultType(string $bytes): void {
    $r = ~$bytes;
    /** @mir-check $r is string */
    $_ = $r;
}

function bitwiseAndStrings(string $a, string $b): void {
    $r = $a & $b;
    /** @mir-check $r is string */
    $_ = $r;
}

function bitwiseOrStrings(string $a, string $b): void {
    $r = $a | $b;
    /** @mir-check $r is string */
    $_ = $r;
}

function bitwiseXorStrings(string $a, string $b): void {
    $r = $a ^ $b;
    /** @mir-check $r is string */
    $_ = $r;
}

function bitwiseNotIntStaysInt(int $n): void {
    $r = ~$n;
    /** @mir-check $r is int */
    $_ = $r;
}

function bitwiseAndMixedTypesStaysInt(string $a, int $b): void {
    $r = $a & $b;
    /** @mir-check $r is int */
    $_ = $r;
}

function shiftLeftStringsStaysInt(string $a, string $b): void {
    $r = $a << $b;
    /** @mir-check $r is int */
    $_ = $r;
}

function shiftRightStringsStaysInt(string $a, string $b): void {
    $r = $a >> $b;
    /** @mir-check $r is int */
    $_ = $r;
}
===expect===
