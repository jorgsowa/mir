===description===
Arithmetic on named int types preserves bounds:
- positive-int + 1 yields int<2, max>
- non-negative-int + 1 yields int<1, max> (positive-int range)
- positive-int + positive-int yields int<2, max>
- negative-int - 1 yields int<min, -2>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $a */
function test_pos_plus_one(int $a): void {
    $x = $a + 1;
    /** @mir-check $x is int<2, max> */
    $_ = $x;
}

/** @param non-negative-int $a */
function test_nonneg_plus_one(int $a): void {
    $x = $a + 1;
    /** @mir-check $x is int<1, max> */
    $_ = $x;
}

/**
 * @param positive-int $a
 * @param positive-int $b
 */
function test_pos_plus_pos(int $a, int $b): void {
    $x = $a + $b;
    /** @mir-check $x is int<2, max> */
    $_ = $x;
}

/** @param negative-int $a */
function test_neg_minus_one(int $a): void {
    $x = $a - 1;
    /** @mir-check $x is int<min, -2> */
    $_ = $x;
}
===expect===
