===description===
concatenated command is reported
===config===
suppress=ForbiddenCode,MixedArrayAccess
===file===
<?php
function run(): void {
    $cmd = 'grep ' . $_GET['needle'];
    shell_exec($cmd);
}
===expect===
TaintedShell@4:4-4:20: Tainted shell command — possible command injection
