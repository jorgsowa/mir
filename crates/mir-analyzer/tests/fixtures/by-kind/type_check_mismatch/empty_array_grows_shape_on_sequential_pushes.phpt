===description===
Several sequential `[]` pushes in straight-line code keep growing the same
list shape, one property per push, staying list-shaped throughout.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<int> $arr */
function test(array $arr): void {
    if ($arr === []) {
        $arr[] = 1;
        $arr[] = 2;
        $arr[] = 3;
        /** @mir-check $arr is array{0: 1, 1: 2, 2: 3} */
        $_ = $arr;
    }
}
===expect===
