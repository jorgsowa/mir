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
TaintedShell@4:5-4:15: Tainted shell command — possible command injection
