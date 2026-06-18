===description===
Casts on literal values fold to the result literal: (string)42 = "42", (int)"5" = 5, (bool)0 = false
===config===
suppress=UnusedVariable,UnusedParam,RedundantCast
===file===
<?php
function test(): void {
    $a = (string) 42;
    /** @mir-check $a is "42" */
    $_ = $a;

    $b = (string) true;
    /** @mir-check $b is "1" */
    $_ = $b;

    $c = (int) "5";
    /** @mir-check $c is 5 */
    $_ = $c;

    $d = (int) true;
    /** @mir-check $d is 1 */
    $_ = $d;

    $e = (int) false;
    /** @mir-check $e is 0 */
    $_ = $e;

    $f = (bool) 0;
    /** @mir-check $f is false */
    $_ = $f;

    $g = (bool) 1;
    /** @mir-check $g is true */
    $_ = $g;

    $h = (bool) "";
    /** @mir-check $h is false */
    $_ = $h;

    $i = (bool) "hello";
    /** @mir-check $i is true */
    $_ = $i;
}
===expect===
