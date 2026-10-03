===description===
MixedArrayAccess does NOT fire after an instanceof check narrows mixed to a concrete object type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(mixed $a): void {
    if ($a instanceof ArrayAccess) {
        $v = $a[0];
    }
}
===expect===
