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
//<^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $a is list<5>
/** @trace $b */
echo count($a), count($b);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $b is list<"x">
