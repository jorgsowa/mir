===description===
Unary negation propagates int range and literal types:
-positive-int → negative-int; -negative-int → positive-int; -(int<a,b>) → int<-b,-a>.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test_negate_positive($n): void {
    $r = -$n;
    /** @mir-check $r is negative-int */
    $_ = $r;
}

/** @param negative-int $n */
function test_negate_negative($n): void {
    $r = -$n;
    /** @mir-check $r is positive-int */
    $_ = $r;
}

/** @param int<2, 8> $n */
function test_negate_range($n): void {
    $r = -$n;
    /** @mir-check $r is int<-8, -2> */
    $_ = $r;
}

/** @param int<-5, 3> $n */
function test_negate_mixed_range($n): void {
    $r = -$n;
    /** @mir-check $r is int<-3, 5> */
    $_ = $r;
}
===expect===
