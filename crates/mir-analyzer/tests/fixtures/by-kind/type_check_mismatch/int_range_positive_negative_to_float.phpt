===description===
N4: All int-range variants (positive-int, negative-int, non-negative-int,
int<a,b>) must be accepted where float is expected. Previously only TInt and
TLiteralInt were coerced in the per-pair subtype path, causing false
InvalidArgument/InvalidReturnType for the narrower int variants.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function needs_float(float $f): void {}

/** @param positive-int $n */
function test_positive_int(int $n): void {
    needs_float($n);
}

/** @param negative-int $n */
function test_negative_int(int $n): void {
    needs_float($n);
}

/** @param non-negative-int $n */
function test_non_negative_int(int $n): void {
    needs_float($n);
}

/** @param int<1,100> $n */
function test_int_range(int $n): void {
    needs_float($n);
}

/** @return float */
function returns_float_from_positive_int(): float {
    /** @var positive-int $n */
    $n = 42;
    return $n;
}

/** @return float */
function returns_float_from_int_range(): float {
    /** @var int<0,255> $n */
    $n = 127;
    return $n;
}
===expect===
