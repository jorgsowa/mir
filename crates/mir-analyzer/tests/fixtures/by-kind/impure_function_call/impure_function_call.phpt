===description===
ImpureFunctionCall
===file===
<?php
/** @pure */
function myPure(int $n): int {
    return mt_rand(0, $n);
}

===expect===
ImpureFunctionCall@4:12-4:26: Calling impure function mt_rand() in a @pure function
