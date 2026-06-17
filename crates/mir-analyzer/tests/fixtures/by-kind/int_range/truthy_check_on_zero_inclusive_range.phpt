===description===
An int range that includes 0 is not a redundant condition in a truthy check;
`int<0,10>` can be both truthy (1-10) and falsy (0).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0, 10> $x */
function test(int $x): void {
    if ($x) {
        /** @mir-check $x is int<0, 10> */
        $_ = $x;
    }
}

/** @param non-negative-int $x */
function test_nonneg(int $x): void {
    if ($x) {
        /** @mir-check $x is non-negative-int */
        $_ = $x;
    }
}
===expect===

