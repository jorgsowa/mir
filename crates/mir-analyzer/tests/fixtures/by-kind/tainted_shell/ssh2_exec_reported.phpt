===description===
`ssh2_exec()` runs a remote command and is a shell sink.
===config===
suppress=MixedArgument,MixedArrayAccess,MissingParamType
===file===
<?php
function test($conn): void {
    ssh2_exec($conn, $_GET['cmd']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
===expect===
