===description===
++ and -- on ranged int types preserve bounds
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test_pre_inc(int $n): void {
    ++$n;
    /** @mir-check $n is int<2, max> */
    $_ = $n;
}

/** @param positive-int $n */
function test_post_inc(int $n): void {
    $n++;
    /** @mir-check $n is int<2, max> */
    $_ = $n;
}

/** @param non-negative-int $n */
function test_pre_dec(int $n): void {
    --$n;
    /** @mir-check $n is int<-1, max> */
    $_ = $n;
}

/** @param int<5, 10> $n */
function test_range_inc(int $n): void {
    ++$n;
    /** @mir-check $n is int<6, 11> */
    $_ = $n;
}
===expect===
