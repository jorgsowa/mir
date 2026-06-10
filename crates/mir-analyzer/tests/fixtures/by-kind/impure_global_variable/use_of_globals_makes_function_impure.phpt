===description===
Use of globals makes function impure
===ignore===
TODO
===file===
<?php
/** @pure */
function addCumulativeGlobals(int $left) : int {
    $GLOBALS["i"] ??= 0;
    $GLOBALS["i"] += $left;
    return $left;
}
===expect===
