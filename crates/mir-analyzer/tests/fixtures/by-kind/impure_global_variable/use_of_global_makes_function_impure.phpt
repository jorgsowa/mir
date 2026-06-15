===description===
Use of global makes function impure
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
ImpureGlobalVariable@5:11-5:13: Using global variable $i in a @pure function
