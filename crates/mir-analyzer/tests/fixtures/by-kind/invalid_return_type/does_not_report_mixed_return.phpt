===description===
does not report mixed return
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(): int {
    $x = json_decode('{}');
    return $x;
}
===expect===
