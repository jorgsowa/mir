===description===
Multiplying non-negative int ranges produces a non-negative result:
non-negative-int * non-negative-int → int<0, max>; bounded × bounded → bounded product.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-negative-int $w
 * @param non-negative-int $h
 */
function test_unbounded($w, $h): void {
    $area = $w * $h;
    /** @mir-check $area is int<0, max> */
    $_ = $area;
}

/**
 * @param int<0, 10> $w
 * @param int<0, 5> $h
 */
function test_bounded($w, $h): void {
    $area = $w * $h;
    /** @mir-check $area is int<0, 50> */
    $_ = $area;
}
===expect===
