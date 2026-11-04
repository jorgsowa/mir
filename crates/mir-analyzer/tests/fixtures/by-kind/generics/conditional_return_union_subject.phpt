===description===
A conditional `@return` with a union subject (`$b is 0|1`) resolves for an argument inside or outside it
===file===
<?php
/** @return ($b is 0|1 ? int : string) */
function pick(int $b) { return $b < 2 ? 1 : ''; }

$a = pick(1);
/** @mir-check $a is int */
echo $a;
$c = pick(2);
/** @mir-check $c is string */
echo $c;
