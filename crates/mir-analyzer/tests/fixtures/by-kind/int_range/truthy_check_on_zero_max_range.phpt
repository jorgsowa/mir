===description===
Truthy check on int<min, 0> tightens the upper bound:
`int<min, 0>` → `negative-int` (truthy) / `0` (falsy);
`int<-5, 0>` → `int<-5, -1>` (truthy) / `0` (falsy).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<min, 0> $x */
function test_neg_or_zero(int $x): void {
    if ($x) {
        /** @mir-check $x is negative-int */
        $_ = $x;
    } else {
        /** @mir-check $x is 0 */
        $_ = $x;
    }
}

/** @param int<-5, 0> $y */
function test_bounded_neg_or_zero(int $y): void {
    if ($y) {
        /** @mir-check $y is int<-5, -1> */
        $_ = $y;
    } else {
        /** @mir-check $y is 0 */
        $_ = $y;
    }
}
===expect===
