===description===
range() with integer literal bounds returns non-empty-list<int<min,max>>.
===config===
suppress=UnusedVariable
===file===
<?php

function test_ascending(): void {
    $r = range(1, 5);
    /** @mir-check $r is non-empty-list<int<1, 5>> */
    $_ = $r;
}

function test_descending(): void {
    $r = range(5, 1);
    /** @mir-check $r is non-empty-list<int<1, 5>> */
    $_ = $r;
}

function test_single_point(): void {
    $r = range(3, 3);
    /** @mir-check $r is non-empty-list<int<3, 3>> */
    $_ = $r;
}

function test_zero_based(): void {
    $r = range(0, 9);
    /** @mir-check $r is non-empty-list<int<0, 9>> */
    $_ = $r;
}
===expect===
