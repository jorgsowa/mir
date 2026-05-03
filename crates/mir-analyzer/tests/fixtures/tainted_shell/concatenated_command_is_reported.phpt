===description===
concatenated command is reported
===file===
<?php
function run(): void {
    $cmd = 'grep ' . $_GET['needle'];
    shell_exec($cmd);
}
===expect===
TaintedShell@4:4: Tainted shell command — possible command injection
===ignore===
TODO
