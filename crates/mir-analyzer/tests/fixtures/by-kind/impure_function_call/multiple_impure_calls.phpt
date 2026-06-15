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
ImpureFunctionCall@4:9-4:23: Calling impure function mt_rand() in a @pure function
ImpureFunctionCall@5:9-5:23: Calling impure function mt_rand() in a @pure function
