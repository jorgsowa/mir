===description===
Comparison operators intersect with an existing int<a, b> range.
`$x >= 3` on `int<0, 10>` narrows to `int<3, 10>`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 10> $x */
function test(int $x): void {
    if ($x >= 3) {
        /** @mir-check $x is int<3, 10> */
        $_ = $x;
    } else {
        /** @mir-check $x is int<0, 2> */
        $_ = $x;
    }
}
===expect===
