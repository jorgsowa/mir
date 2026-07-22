===description===
`array_push($arr, ...$values)` with several arguments in one call pushes
each value in order, same as that many sequential `$arr[] = …;` writes —
the resulting shape gains one property per pushed value.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $arr = [];
    array_push($arr, 1, 2, 3);
    /** @mir-check $arr is array{0: 1, 1: 2, 2: 3} */
    $_ = $arr;
}
===expect===
