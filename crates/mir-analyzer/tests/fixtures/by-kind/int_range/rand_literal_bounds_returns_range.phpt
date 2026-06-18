===description===
rand($min, $max) / mt_rand / random_int with literal integer bounds return int<min, max>.
With no arguments or variable bounds, falls through to the stub return type.
===config===
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
function test_rand(): void {
    $r = rand(1, 100);
    /** @mir-check $r is int<1, 100> */
    $_ = $r;
}

function test_mt_rand(): void {
    $r = mt_rand(0, 7);
    /** @mir-check $r is int<0, 7> */
    $_ = $r;
}

function test_random_int_pos(): void {
    $r = random_int(1, 10);
    /** @mir-check $r is int<1, 10> */
    $_ = $r;
}

function test_rand_no_args(): void {
    $r = rand();
    /** @mir-check $r is int */
    $_ = $r;
}
===expect===
