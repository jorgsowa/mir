===description===
Use of static makes function impure
===file===
<?php
/** @pure */
function addCumulative(int $left) : int {
    /** @var int */
    static $i = 0;
    $i += $left;
    return $left;
}
===expect===
ImpureStaticVariable@5:12-5:18: Using static variable $i in a @pure function
