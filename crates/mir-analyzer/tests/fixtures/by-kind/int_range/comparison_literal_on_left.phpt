===description===
Comparison with literal on the left (`5 > $x`) is equivalent to `$x < 5`.
Operator is flipped automatically during normalization.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if (5 > $x) {
        /** @mir-check $x is int<min, 4> */
        $_ = $x;
    } else {
        /** @mir-check $x is int<5, max> */
        $_ = $x;
    }
}
===expect===
