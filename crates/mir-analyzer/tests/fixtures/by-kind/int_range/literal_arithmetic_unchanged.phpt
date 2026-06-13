===description===
arithmetic on plain ints (no range operand) is unchanged — stays `int`, no spurious range
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $a, int $b): void {
    $c = $a + $b;
    /** @mir-check $c is int */
    $_ = $c;
}
===expect===
