===description===
Use of globals makes function impure
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @pure */
function addCumulativeGlobals(int $left) : int {
    $GLOBALS["i"] ??= 0;
//  ^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $i in a @pure function
    $GLOBALS["i"] += $left;
//  ^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $i in a @pure function
    return $left;
}
===expect===
