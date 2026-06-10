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
