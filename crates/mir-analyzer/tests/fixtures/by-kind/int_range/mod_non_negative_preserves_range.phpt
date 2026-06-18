===description===
Modulo of non-negative int by a positive literal gives a bounded range: int<0, divisor-1>.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-negative-int $n
 */
function test_non_negative($n): void {
    $r = $n % 5;
    /** @mir-check $r is int<0, 4> */
    $_ = $r;
}

/**
 * @param int<0, 100> $n
 */
function test_bounded($n): void {
    $r = $n % 10;
    /** @mir-check $r is int<0, 9> */
    $_ = $r;
}
===expect===
