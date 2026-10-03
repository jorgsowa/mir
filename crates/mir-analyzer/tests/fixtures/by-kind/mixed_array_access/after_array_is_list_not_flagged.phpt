===description===
MixedArrayAccess does NOT fire after array_is_list() because that check narrows mixed to list<mixed>, removing the mixed atom.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(mixed $a): void {
    if (array_is_list($a)) {
        $v = $a[0];
    }
}
===expect===
