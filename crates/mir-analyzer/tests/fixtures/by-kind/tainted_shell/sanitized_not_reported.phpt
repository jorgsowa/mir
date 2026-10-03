===description===
sanitized not reported
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $cmd = escapeshellarg($_GET['cmd']);
    exec($cmd);
}
===expect===
