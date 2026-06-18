===description===
Excluding a value at the edge of an int range tightens the bound:
non-negative-int !== 0 → positive-int; positive-int !== 1 → int<2,max>; int<min,-1> !== -1 → negative-int without -1
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-negative-int $a */
function test_noneg(int $a): void {
    if ($a !== 0) {
        /** @mir-check $a is positive-int */
        $_ = $a;
    }
}

/** @param positive-int $b */
function test_pos(int $b): void {
    if ($b !== 1) {
        /** @mir-check $b is int<2, max> */
        $_ = $b;
    }
}

/** @param int<5, 10> $c */
function test_range_lo(int $c): void {
    if ($c !== 5) {
        /** @mir-check $c is int<6, 10> */
        $_ = $c;
    }
}

/** @param int<5, 10> $d */
function test_range_hi(int $d): void {
    if ($d !== 10) {
        /** @mir-check $d is int<5, 9> */
        $_ = $d;
    }
}
===expect===
