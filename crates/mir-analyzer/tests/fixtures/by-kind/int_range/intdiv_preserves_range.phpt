===description===
intdiv() on non-negative dividend with positive divisor infers a bounded range:
non-negative-int / positive-int → non-negative-int; int<0,N> / K → int<0, N/K>.
===config===
suppress=UnusedVariable,UnusedParam,MissingThrowsDocblock
===file===
<?php
/** @param non-negative-int $n */
function test_non_negative_by_literal($n): void {
    $r = intdiv($n, 3);
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}

/** @param int<0, 100> $n */
function test_bounded_range($n): void {
    $r = intdiv($n, 10);
    /** @mir-check $r is int<0, 10> */
    $_ = $r;
}

/** @param positive-int $n */
function test_positive_by_literal($n): void {
    $r = intdiv($n, 2);
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}
===expect===
