===description===
Use of static makes function impure
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
