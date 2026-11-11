===description===
An int argument is checked against the int types of a union that lists any, not widened to its `float` member: `0` is rejected for `non-empty-mixed`, `non-empty-scalar` and `positive-int|float`. Without an int type in the union, an int still widens to `float`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param non-empty-mixed $value */
function nonEmptyMixed($value): void {}

/** @param non-empty-scalar $value */
function nonEmptyScalar($value): void {}

/** @param positive-int|float $value */
function positiveOrFloat($value): void {}

/** @param float|string $value */
function floatOrString($value): void {}

nonEmptyMixed(1);
nonEmptyMixed(1.5);
nonEmptyMixed(0);
//            ^ InvalidArgument: Argument $value of nonEmptyMixed() expects 'non-falsy-string|negative-int|positive-int|float|true|non-empty-array|object|resource', got '0'
nonEmptyScalar(0);
//             ^ InvalidArgument: Argument $value of nonEmptyScalar() expects 'non-falsy-string|negative-int|positive-int|float|true', got '0'
positiveOrFloat(3);
positiveOrFloat(0.0);
positiveOrFloat(0);
//              ^ InvalidArgument: Argument $value of positiveOrFloat() expects 'positive-int|float', got '0'
positiveOrFloat(-1);
//              ^^ InvalidArgument: Argument $value of positiveOrFloat() expects 'positive-int|float', got '-1'
floatOrString(0);
