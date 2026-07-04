===description===
Truthy check on an int range that includes 0 tightens the lower bound:
`int<0,10>` → `int<1,10>` (truthy) / `0` (falsy); `non-negative-int` → `positive-int` (truthy).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 10> $x */
function test(int $x): void {
    if ($x) {
        /** @mir-check $x is int<1, 10> */
        $_ = $x;
    } else {
        /** @mir-check $x is 0 */
        $_ = $x;
    }
}

/** @param non-negative-int $x */
function test_nonneg(int $x): void {
    if ($x) {
        /** @mir-check $x is positive-int */
        $_ = $x;
    } else {
        /** @mir-check $x is 0 */
        $_ = $x;
    }
}
===expect===
