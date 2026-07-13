===description===
array_reduce infers the result type from the callback's return type, not bare mixed.
===config===
suppress=UnusedVariable
===file===
<?php
/** @param list<int> $ints */
function sumInts(array $ints): void {
    $r = array_reduce($ints, fn(int $c, int $x): int => $c + $x, 0);
    /** @mir-check $r is int */
    $_ = $r;
}
===expect===
