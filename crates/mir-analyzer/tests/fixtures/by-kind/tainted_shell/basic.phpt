===description===
Basic
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $cmd = $_GET['cmd'];
    exec($cmd);
}
===expect===
TaintedShell@4:4-4:14: Tainted shell command — possible command injection
