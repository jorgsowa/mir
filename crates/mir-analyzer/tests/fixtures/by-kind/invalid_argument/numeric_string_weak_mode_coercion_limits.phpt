===description===
Numeric-string coercion stays limited to what PHP accepts: plain/non-numeric strings, fractional strings to int, and strict_types are still rejected
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_number(int|float $x): void {}
function takes_int(int $x): void {}

/** @param numeric-string $ns */
function limits(string $plain, $ns, float $f): void {
    takes_number($plain);
//               ^^^^^^ InvalidScalarArgument: Argument $x of takes_number() expects 'int|float', got scalar 'string'
    takes_number("abc");
//               ^^^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got '"abc"'
    takes_int($ns);
//            ^^^ InvalidScalarArgument: Argument $x of takes_int() expects 'int', got scalar 'numeric-string'
    takes_int("1.5");
//            ^^^^^ InvalidScalarArgument: Argument $x of takes_int() expects 'int', got scalar '"1.5"'
    takes_int("12");
//            ^^^^ InvalidScalarArgument: Argument $x of takes_int() expects 'int', got scalar '"12"'
    takes_number("NAN");
//               ^^^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got '"NAN"'
    $s = (string)$f;
    /** @mir-check $s is string */
}
