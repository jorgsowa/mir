===description===
useOfGlobalsMakesFunctionImpure
===file===
<?php
/** @pure */
function addCumulativeGlobals(int $left) : int {
    $GLOBALS["i"] ??= 0;
    $GLOBALS["i"] += $left;
    return $left;
}
===expect===
ImpureGlobalVariable
===ignore===
TODO
