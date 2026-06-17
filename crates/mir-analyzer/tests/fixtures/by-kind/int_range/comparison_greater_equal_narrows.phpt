===description===
`$x > 0` on an `int<0, max>` (from count) narrows to `int<1, max>` in true branch.
The false branch gets `int<0, 0>` (i.e. exactly zero).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string> $arr */
function test(array $arr): void {
    $n = count($arr);
    if ($n > 0) {
        /** @mir-check $n is int<1, max> */
        $_ = $n;
    } else {
        /** @mir-check $n is int<0, 0> */
        $_ = $n;
    }
}
===expect===
