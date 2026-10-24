===description===
A closure, arrow function or callable variable on the right of `|>` is
invoked with the left operand; the result is the callable's return type.
===file===
<?php
$half = function (int $x): float { return $x / 2; };

$a = [1, 2] |> (fn(array $xs): int => count($xs));
/** @trace $a */
$b = 7 |> $half;
//<^^^^^^^^^^^^^^^^ Trace: Type of $a is int<0, max>
/** @trace $b */
$c = 3 |> function (int $x): string { return str_repeat('x', $x); };
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $b is float
/** @trace $c */
echo $a, $b, $c;
//<^^^^^^^^^^^^^^^^ Trace: Type of $c is string
