===description===
min() and max() on integer subtypes infer a tighter return range:
- min(a, b): result_min = min(a_min, b_min), result_max = min(a_max, b_max)
- max(a, b): result_min = max(a_min, b_min), result_max = max(a_max, b_max)
===config===
suppress=UnusedVariable,UnusedParam,MissingParamType,MixedAssignment
===file===
<?php
/**
 * @param int<2, 8> $a
 * @param int<1, 5> $b
 */
function test_min_bounded($a, $b): void {
    $r = min($a, $b);
    /** @mir-check $r is int<1, 5> */
    $_ = $r;
}

/**
 * @param int<2, 8> $a
 * @param int<1, 5> $b
 */
function test_max_bounded($a, $b): void {
    $r = max($a, $b);
    /** @mir-check $r is int<2, 8> */
    $_ = $r;
}

/**
 * @param non-negative-int $a
 * @param positive-int $b
 */
function test_min_named($a, $b): void {
    $r = min($a, $b);
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}

/**
 * @param non-negative-int $a
 * @param positive-int $b
 */
function test_max_named($a, $b): void {
    $r = max($a, $b);
    /** @mir-check $r is positive-int */
    $_ = $r;
}

function test_min_literals(): void {
    $r = min(3, 7);
    /** @mir-check $r is int<3, 3> */
    $_ = $r;
}

function test_max_literals(): void {
    $r = max(3, 7);
    /** @mir-check $r is int<7, 7> */
    $_ = $r;
}
===expect===
