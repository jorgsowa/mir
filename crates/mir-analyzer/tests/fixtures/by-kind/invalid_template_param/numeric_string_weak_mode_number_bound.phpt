===description===
Weak mode coerces numeric strings to a template bounded by int|float; the result is int|float.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of int|float
 * @param T $n
 * @return T
 */
function twice($n) { return $n; }

/** @param numeric-string $ns */
function f(string $s, int $i, $ns): void {
    $a = twice($ns);
    /** @mir-check $a is int|float */
    $b = twice('5');
    /** @mir-check $b is int|float */
    $c = twice($i);
    /** @mir-check $c is int */
    $d = twice($s);
//       ^^^^^^^^^ InvalidScalarArgument: Argument $n of twice() expects 'int|float', got scalar 'string'
    $e = twice('abc');
//       ^^^^^^^^^^^^ InvalidArgument: Argument $n of twice() expects 'int|float', got '"abc"'
}
