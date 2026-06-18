===description===
(string) cast returns non-empty-string when the source type guarantees a non-empty output.
Ints, floats, and true all stringify to non-empty results.
===config===
suppress=UnusedVariable,UnusedParam,RedundantCast
===file===
<?php
/** @param positive-int $n */
function test_positive_int(int $n): void {
    $s = (string) $n;
    /** @mir-check $s is non-empty-string */
    $_ = $s;
}

function test_literal_int(): void {
    $s = (string) 42;
    /** @mir-check $s is '42' */
    $_ = $s;
}

/** @param int $n */
function test_int(int $n): void {
    $s = (string) $n;
    /** @mir-check $s is non-empty-string */
    $_ = $s;
}

function test_true(): void {
    $s = (string) true;
    /** @mir-check $s is '1' */
    $_ = $s;
}

function test_float(): void {
    $s = (string) 3.14;
    /** @mir-check $s is '3.14' */
    $_ = $s;
}

function test_nullable_string(?string $x): void {
    $s = (string) $x;
    /** @mir-check $s is string */
    $_ = $s;
}
===expect===
