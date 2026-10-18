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
/** @trace $b */
echo $a, $b;
===expect===
Trace@9:0-9:26: Type of $a is int
Trace@11:0-11:12: Type of $b is string
