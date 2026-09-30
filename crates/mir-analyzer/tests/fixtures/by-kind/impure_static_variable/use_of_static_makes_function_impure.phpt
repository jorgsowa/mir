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
//         ^^^^^^ ImpureStaticVariable: Using static variable $i in a @pure function
    $i += $left;
    return $left;
}
===expect===
