===description===
An integer `exit` operand is a status code, not output.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    exit((int) $_GET['code']);
}
