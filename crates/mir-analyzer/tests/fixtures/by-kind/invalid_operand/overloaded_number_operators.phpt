===description===
`GMP` and `BcMath\Number` overload arithmetic: combined with the same class,
an int or a numeric string, the result is that class. A non-numeric string
operand or a mix of the two classes is still `InvalidOperand`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use BcMath\Number;

function takesNumber(Number $value): void {}
function takesGmp(GMP $value): void {}
function takesInt(int $value): void {}

function run(Number $n, GMP $g, int $i): void {
    takesNumber($n * $i);
    takesNumber($n + $n);
    takesNumber('1.5' - $n);
    takesGmp($g + $g);
    takesGmp($g * $i);
    takesGmp($g << 2);
    $n += 1;
    takesNumber($n);
    takesInt($n * 2);
//           ^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'BcMath\Number'
    takesNumber($n + 'abc');
//              ^^^^^^^^^^ InvalidArgument: Argument $value of takesNumber() expects 'BcMath\Number', got 'int|float'
//              ^^^^^^^^^^ InvalidOperand: Operator '+' not supported between 'BcMath\Number' and '"abc"'
    takesGmp($g + $n);
//           ^^^^^^^ InvalidArgument: Argument $value of takesGmp() expects 'GMP', got 'int|float'
//           ^^^^^^^ InvalidOperand: Operator '+' not supported between 'GMP' and 'BcMath\Number'
}
