===description===
ImpureFunctionCall fires once for each impure call inside a @pure function.
===file===
<?php
/** @pure */
function twiceImpure(): string {
    $a = mt_rand(0, 10);
    $b = mt_rand(0, 20);
    return (string)($a + $b);
}

===expect===
ImpureFunctionCall@4:10-4:24: Calling impure function mt_rand() in a @pure function
ImpureFunctionCall@5:10-5:24: Calling impure function mt_rand() in a @pure function
