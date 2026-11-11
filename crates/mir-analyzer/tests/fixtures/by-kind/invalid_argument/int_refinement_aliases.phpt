===description===
`non-zero-int`, `non-positive-int`, `int-range<>` and `number` accept the values they admit and reject the rest.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/** @param non-zero-int $value */
function acceptsNonZero($value): void {}

/** @param non-positive-int $value */
function acceptsNonPositive($value): void {}

/** @param int-range<0, 255> $value */
function acceptsByte($value): void {}

/** @param number $value */
function acceptsNumber($value): void {}

acceptsNonZero(1);
acceptsNonZero(-1);
acceptsNonZero(0);
//             ^ InvalidArgument: Argument $value of acceptsNonZero() expects 'negative-int|positive-int', got '0'

acceptsNonPositive(0);
acceptsNonPositive(-5);
acceptsNonPositive(1);
//                 ^ InvalidArgument: Argument $value of acceptsNonPositive() expects 'int<min, 0>', got '1'

acceptsByte(200);
acceptsByte(256);
//          ^^^ InvalidArgument: Argument $value of acceptsByte() expects 'int<0, 255>', got '256'

acceptsNumber(1);
acceptsNumber(1.5);
acceptsNumber('abc');
//            ^^^^^ InvalidArgument: Argument $value of acceptsNumber() expects 'int|float', got '"abc"'
