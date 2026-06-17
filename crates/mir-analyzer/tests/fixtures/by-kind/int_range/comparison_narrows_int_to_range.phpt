===description===
Comparison operators narrow integer variables to bounded ranges.
`$x < 5` narrows `$x: int` to `int<min, 4>` in the true branch and `int<5, max>` in the false branch.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $x): void {
    if ($x < 5) {
        /** @mir-check $x is int<min, 4> */
        $_ = $x;
    } else {
        /** @mir-check $x is int<5, max> */
        $_ = $x;
    }
}
===expect===
