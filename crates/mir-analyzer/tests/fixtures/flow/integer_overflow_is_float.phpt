===description===
Literal integer arithmetic that overflows is a float, as in PHP; `PHP_INT_MIN` is an int and `PHP_INT_MIN % -1` is 0.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = PHP_INT_MAX + 1;
/** @mir-check $a is float */
$a;
$b = PHP_INT_MIN - 1;
/** @mir-check $b is float */
$b;
$c = PHP_INT_MAX * 2;
/** @mir-check $c is float */
$c;
$d = -PHP_INT_MIN;
/** @mir-check $d is float */
$d;
$e = PHP_INT_MIN / -1;
/** @mir-check $e is float */
$e;
$f = PHP_INT_MIN % -1;
/** @mir-check $f is 0 */
$f;
$g = PHP_INT_MIN;
/** @mir-check $g is -9223372036854775808 */
$g;
