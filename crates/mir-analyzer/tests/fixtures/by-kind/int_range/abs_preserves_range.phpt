===description===
abs() on a typed int argument infers a tighter return type:
negative-int → positive-int; non-negative-int → non-negative-int; int → non-negative-int;
bounded ranges reflect the absolute-value transformation.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param negative-int $n */
function test_negative($n): void {
    $r = abs($n);
    /** @mir-check $r is positive-int */
    $_ = $r;
}

/** @param non-negative-int $n */
function test_non_negative($n): void {
    $r = abs($n);
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}

/** @param int $n */
function test_int($n): void {
    $r = abs($n);
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}

/** @param int<-5, 3> $n */
function test_mixed_range($n): void {
    $r = abs($n);
    /** @mir-check $r is int<0, 5> */
    $_ = $r;
}

/** @param int<-8, -2> $n */
function test_negative_range($n): void {
    $r = abs($n);
    /** @mir-check $r is int<2, 8> */
    $_ = $r;
}

/** @param int<2, 6> $n */
function test_positive_range($n): void {
    $r = abs($n);
    /** @mir-check $r is int<2, 6> */
    $_ = $r;
}
===expect===
