===description===
count() on a sealed keyed-array shape with all required keys returns the exact literal count
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = [1, 2, 3];
    $n = count($a);
    /** @mir-check $n is 3 */
    $_ = $n;

    $b = ['x' => 1, 'y' => 2];
    $m = count($b);
    /** @mir-check $m is 2 */
    $_ = $m;

    $c = [];
    $z = count($c);
    /** @mir-check $z is 0 */
    $_ = $z;
}
===expect===
