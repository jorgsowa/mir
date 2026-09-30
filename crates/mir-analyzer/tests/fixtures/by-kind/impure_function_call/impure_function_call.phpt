===description===
ImpureFunctionCall
===file===
<?php
/** @pure */
function myPure(int $n): int {
    return mt_rand(0, $n);
//         ^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mt_rand() in a @pure function
}

===expect===
