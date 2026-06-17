===description===
Comparison on `non-negative-int` intersects with its implicit `int<0,max>` bound.
`$n < 3` on `non-negative-int` narrows to `int<0,2>`, not `int<min,2>`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-negative-int $n */
function test(int $n): void {
    if ($n < 3) {
        /** @mir-check $n is int<0, 2> */
        $_ = $n;
    } else {
        /** @mir-check $n is int<3, max> */
        $_ = $n;
    }
}
===expect===
