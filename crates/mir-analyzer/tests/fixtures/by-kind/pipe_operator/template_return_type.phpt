===description===
A generic callable on the right of `|>` infers its template from the left
operand, as an ordinary call does.
===file===
<?php
/**
 * @template T
 * @param T $value
 * @return list<T>
 */
function listOf(mixed $value): array { return [$value]; }

$a = 5 |> listOf(...);
/** @trace $a */
$b = "x" |> listOf(...);
/** @trace $b */
echo count($a), count($b);
===expect===
Trace@11:0-11:24: Type of $a is list<5>
Trace@13:0-13:26: Type of $b is list<"x">
