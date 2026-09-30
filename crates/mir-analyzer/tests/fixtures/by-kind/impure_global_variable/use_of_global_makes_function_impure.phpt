===description===
Use of global makes function impure
===file===
<?php
/** @pure */
function addCumulative(int $left) : int {
    /** @var int */
    global $i;
//         ^^ ImpureGlobalVariable: Using global variable $i in a @pure function
    $i ??= 0;
    $i += $left;
    return $left;
}
===expect===
