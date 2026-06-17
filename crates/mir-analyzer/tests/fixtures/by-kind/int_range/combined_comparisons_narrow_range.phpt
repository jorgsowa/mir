===description===
Chained comparisons with `&&` narrow the type on both ends.
`$x > 0 && $x < 10` on `int` gives `int<1,9>`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x > 0 && $x < 10) {
        /** @mir-check $x is int<1, 9> */
        $_ = $x;
    }
}
===expect===
