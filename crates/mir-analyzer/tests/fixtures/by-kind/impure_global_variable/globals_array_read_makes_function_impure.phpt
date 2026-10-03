===description===
Reading $GLOBALS['x'] reaches the same external mutable state as
`global $x;`, but only the `global` statement was ever checked — a plain
read through the superglobal array bypassed the purity check entirely.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @pure */
function test(): int {
    return $GLOBALS['x'];
//         ^^^^^^^^^^^^^ ImpureGlobalVariable: Using global variable $x in a @pure function
}
===expect===
