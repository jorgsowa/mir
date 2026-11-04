===description===
An omitted argument with an integer default resolves a conditional `@return` on that parameter
===file===
<?php
/** @return ($b is 1 ? int : null) */
function pick(int $b = 1): ?int { return $b === 1 ? 1 : null; }

$a = pick();
/** @mir-check $a is int */
echo $a;
