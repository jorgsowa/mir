===description===
Positional `list()`/`[$a, $b]` destructuring of a literal array must resolve
each target's type from the source's per-index property, not fall back to
mixed.
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    [$a, $b] = ['x', 5];
    /** @mir-check $a is string */
    echo 1;
    /** @mir-check $b is int */
    echo 2;

    list($c, $d) = ['x', 5];
    /** @mir-check $c is string */
    echo 3;
    /** @mir-check $d is int */
    echo 4;
}
===expect===
