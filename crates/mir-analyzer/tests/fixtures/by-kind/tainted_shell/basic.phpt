===description===
Basic
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
function test(): void {
    $cmd = $_GET['cmd'];
    exec($cmd);
//  ^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
===expect===
