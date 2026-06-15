===description===
Use of static makes function impure
===config===
suppress=MixedAssignment,UnusedVariable
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
ImpureStaticVariable@5:11-5:17: Using static variable $i in a @pure function
