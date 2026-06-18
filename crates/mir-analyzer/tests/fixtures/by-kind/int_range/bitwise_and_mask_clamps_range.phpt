===description===
$x & mask where mask is a non-negative literal returns int<0, mask>.
$x >> n where $x is non-negative returns non-negative-int.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_and_byte_mask(int $n): void {
    $r = $n & 0xFF;
    /** @mir-check $r is int<0, 255> */
    $_ = $r;
}

function test_and_nibble_mask(int $n): void {
    $r = $n & 0x0F;
    /** @mir-check $r is int<0, 15> */
    $_ = $r;
}

function test_and_zero_mask(int $n): void {
    $r = $n & 0;
    /** @mir-check $r is 0 */
    $_ = $r;
}

function test_mask_on_left(int $n): void {
    $r = 0xFF & $n;
    /** @mir-check $r is int<0, 255> */
    $_ = $r;
}

/** @param non-negative-int $n */
function test_right_shift_non_negative($n): void {
    $r = $n >> 1;
    /** @mir-check $r is non-negative-int */
    $_ = $r;
}
===expect===
