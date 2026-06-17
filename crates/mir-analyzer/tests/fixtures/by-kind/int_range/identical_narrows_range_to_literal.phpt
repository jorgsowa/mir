===description===
`$x === 5` on `int<0,10>` narrows to `TLiteralInt(5)` in the true branch.
The false branch keeps the range unchanged.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0,10> $x */
function test(int $x): void {
    if ($x === 5) {
        /** @mir-check $x is 5 */
        $_ = $x;
    } else {
        /** @mir-check $x is int<0, 10> */
        $_ = $x;
    }
}
===expect===
