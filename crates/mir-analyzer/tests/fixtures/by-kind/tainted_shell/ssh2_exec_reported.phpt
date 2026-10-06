===description===
`ssh2_exec()` runs a remote command and is a shell sink.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test($conn): void {
    ssh2_exec($conn, $_GET['cmd']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
