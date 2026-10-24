===description===
A function-name string or an invokable object on the right of `|>` is called
with the left operand.
===file===
<?php
function double(int $x): int { return $x * 2; }
final class Stringify {
    public function __invoke(int $x): string { return (string) $x; }
}

$a = 5 |> 'double';
/** @trace $a */
$b = 5 |> new Stringify();
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $a is int
/** @trace $b */
echo $a, $b;
//<^^^^^^^^^^^^ Trace: Type of $b is string
