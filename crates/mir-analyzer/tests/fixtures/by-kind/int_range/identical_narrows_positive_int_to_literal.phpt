===description===
`$n === 3` on `positive-int` narrows to `TLiteralInt(3)` in the true branch.
A value outside the subtype bounds (e.g. -1 on positive-int) produces an empty
intersection and leaves the type unchanged (dead-code branch).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n === 3) {
        /** @mir-check $n is 3 */
        $_ = $n;
    } else {
        /** @mir-check $n is positive-int */
        $_ = $n;
    }
}
===expect===
