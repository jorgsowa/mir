===description===
`($c is class-string ? A : B)` resolves to A for a `Foo::class` argument and to B for a non-string
===file===
<?php
class Foo {}
/** @return ($c is class-string ? int : string) */
function pick(string|int $c) { return is_string($c) ? 1 : ''; }

$a = pick(Foo::class);
/** @mir-check $a is int */
echo $a;
$b = pick(1);
/** @mir-check $b is string */
echo $b;
