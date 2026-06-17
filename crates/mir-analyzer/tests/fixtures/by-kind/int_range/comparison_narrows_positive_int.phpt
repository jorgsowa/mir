===description===
Comparison on `positive-int` intersects with its implicit `int<1,max>` bound.
`$n < 5` on `positive-int` narrows to `int<1,4>`, not `int<min,4>`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n < 5) {
        /** @mir-check $n is int<1, 4> */
        $_ = $n;
    } else {
        /** @mir-check $n is int<5, max> */
        $_ = $n;
    }
}
===expect===
