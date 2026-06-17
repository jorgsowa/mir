===description===
Comparison on `negative-int` intersects with its implicit `int<min,-1>` bound.
`$n > -5` on `negative-int` narrows to `int<-4,-1>`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param negative-int $n */
function test(int $n): void {
    if ($n > -5) {
        /** @mir-check $n is int<-4, -1> */
        $_ = $n;
    } else {
        /** @mir-check $n is int<min, -5> */
        $_ = $n;
    }
}
===expect===
