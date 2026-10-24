===description===
`$x |> f(...)` has the return type of `f($x)`, not the type of the callable.
===file===
<?php
function double(int $x): int { return $x * 2; }

$a = 5 |> double(...);
/** @trace $a */
$b = "hi" |> strtoupper(...);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $a is int
/** @trace $b */
$c = [3, 1, 2] |> array_sum(...);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $b is non-empty-string
/** @trace $c */
echo $a + $c, $b;
//<^^^^^^^^^^^^^^^^^ Trace: Type of $c is int|float
