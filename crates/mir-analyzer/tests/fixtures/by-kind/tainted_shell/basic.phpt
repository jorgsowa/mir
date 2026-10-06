===description===
Basic
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $cmd = $_GET['cmd'];
    exec($cmd);
//  ^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
