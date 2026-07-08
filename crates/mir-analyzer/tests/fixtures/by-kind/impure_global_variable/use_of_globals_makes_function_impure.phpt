===description===
Use of globals makes function impure
===config===
suppress=MixedArrayAccess
===file===
<?php
/** @pure */
function addCumulativeGlobals(int $left) : int {
    $GLOBALS["i"] ??= 0;
    $GLOBALS["i"] += $left;
    return $left;
}
===expect===
ImpureGlobalVariable@4:4-4:17: Using global variable $i in a @pure function
ImpureGlobalVariable@5:4-5:17: Using global variable $i in a @pure function
