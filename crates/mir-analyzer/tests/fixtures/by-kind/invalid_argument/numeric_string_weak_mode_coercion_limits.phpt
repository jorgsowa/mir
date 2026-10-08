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
//               ^^^^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got 'string'
    takes_number("abc");
//               ^^^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got '"abc"'
    takes_int($ns);
//            ^^^ InvalidArgument: Argument $x of takes_int() expects 'int', got 'numeric-string'
    takes_int("1.5");
//            ^^^^^ InvalidArgument: Argument $x of takes_int() expects 'int', got '"1.5"'
    takes_int("12");
//            ^^^^ InvalidArgument: Argument $x of takes_int() expects 'int', got '"12"'
    takes_number("NAN");
//               ^^^^^ InvalidArgument: Argument $x of takes_number() expects 'int|float', got '"NAN"'
    $s = (string)$f;
    /** @mir-check $s is string */
}
