===description===
Narrowing positive-int in the false branch of a comparison
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    if ($n > 5) {
        /** @mir-check $n is int<6, max> */
        $_ = $n;
    } else {
        /** @mir-check $n is int<1, 5> */
        $_ = $n;
    }
}
===expect===

