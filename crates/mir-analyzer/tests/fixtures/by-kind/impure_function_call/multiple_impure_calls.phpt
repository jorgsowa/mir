===description===
ImpureFunctionCall fires once for each impure call inside a @pure function.
===file===
<?php
/** @pure */
function twiceImpure(): string {
    $a = mt_rand(0, 10);
//       ^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mt_rand() in a @pure function
    $b = mt_rand(0, 20);
//       ^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mt_rand() in a @pure function
    return (string)($a + $b);
}
