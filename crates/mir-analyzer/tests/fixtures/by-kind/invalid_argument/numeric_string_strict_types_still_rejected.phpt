===description===
Under strict_types=1 a numeric-string is not coerced to a float or int|float param
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

function takes_number(int|float $x): void {}
function takes_float(float $x): void {}

/** @param numeric-string $ns */
function strict($ns): void {
    takes_number($ns);
//               ^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got 'numeric-string'
    takes_float("1.5");
//              ^^^^^ InvalidArgument: Argument $x of takes_float() expects 'float', got '"1.5"'
}
