===description===
`$x |> f(...)` has the return type of `f($x)`, not the type of the callable.
===file===
<?php
function double(int $x): int { return $x * 2; }

$a = 5 |> double(...);
/** @trace $a */
$b = "hi" |> strtoupper(...);
/** @trace $b */
$c = [3, 1, 2] |> array_sum(...);
/** @trace $c */
echo $a + $c, $b;
===expect===
Trace@6:0-6:29: Type of $a is int
Trace@8:0-8:33: Type of $b is non-empty-string
Trace@10:0-10:17: Type of $c is int|float
