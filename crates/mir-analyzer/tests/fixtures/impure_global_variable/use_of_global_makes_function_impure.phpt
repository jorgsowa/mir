===description===
useOfGlobalMakesFunctionImpure
===file===
<?php
/** @pure */
function addCumulative(int $left) : int {
    /** @var int */
    global $i;
    $i ??= 0;
    $i += $left;
    return $left;
}
===expect===
ImpureGlobalVariable
===ignore===
TODO
