===description===
A nested conditional return that resolves to float is rejected for an int parameter under strict_types.
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

/**
 * @return ($left is int ? ($right is int ? int : float) : float)
 */
function add(int|float $left, int|float $right): int|float {
    return $left + $right;
}

function takesInt(int $value): void {}
function takesFloat(float $value): void {}

takesInt(add(1, 2));
takesFloat(add(1, 1.5));
takesFloat(add(1.5, 1));
takesInt(add(1, 1.5));
//       ^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'float'
takesInt(add(1.5, 1));
//       ^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got 'float'
