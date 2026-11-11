===description===
Under strict_types a float argument is never converted to an int parameter.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

function takesInt(int $value): void {}
function takesNullableInt(?int $value): void {}
/** @param positive-int $value */
function takesPositive(int $value): void {}

function run(float $ratio): void {
    takesInt($ratio);
//           ^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'float'
    takesNullableInt($ratio);
//                   ^^^^^^ InvalidArgument: Argument $value of takesNullableInt() expects 'int|null', got 'float'
    takesPositive($ratio);
//                ^^^^^^ InvalidArgument: Argument $value of takesPositive() expects 'positive-int', got 'float'
    takesInt($ratio * 2);
//           ^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'float'
}
