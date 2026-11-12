===description===
`pow()` on int and float operands is typed like `**`: two int literals fold to their int result, so passing `pow(10, 8)` to an int parameter is accepted. An overflowing or negative exponent result is a float, and a non-literal int base gives `int|float`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function setMax(int $max): void {}

setMax(pow(10, 8));
setMax(10 ** 8);
setMax(pow(2, -1));
//     ^^^^^^^^^^ InvalidArgument: Argument $max of setMax() expects 'int', got 'float'

$overflow = pow(2, 63);
/** @mir-check $overflow is float */
$half = 2 ** -1;
/** @mir-check $half is float */
$mixed = pow(2.0, 3);
/** @mir-check $mixed is float */
echo $overflow + $half + $mixed;

function square(int $n): void
{
    $squared = pow($n, 2);
    /** @mir-check $squared is int|float */
    echo $squared;
}
