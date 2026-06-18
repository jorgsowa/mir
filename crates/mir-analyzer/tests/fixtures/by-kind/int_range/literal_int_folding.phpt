===description===
Arithmetic on two integer literals produces a literal result: 5 + 3 = 8, 10 - 4 = 6, 3 * 7 = 21
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $a = 5 + 3;
    /** @mir-check $a is 8 */
    $_ = $a;

    $b = 10 - 4;
    /** @mir-check $b is 6 */
    $_ = $b;

    $c = 3 * 7;
    /** @mir-check $c is 21 */
    $_ = $c;

    $d = 10 / 2;
    /** @mir-check $d is 5 */
    $_ = $d;

    $e = 10 % 3;
    /** @mir-check $e is 1 */
    $_ = $e;
}
===expect===
