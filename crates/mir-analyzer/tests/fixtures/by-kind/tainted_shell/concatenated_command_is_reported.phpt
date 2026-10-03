===description===
concatenated command is reported
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(): void {
    $cmd = 'grep ' . $_GET['needle'];
    shell_exec($cmd);
//  ^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
===expect===
