===file===
<?php
function run(): void {
    $cmd = 'grep ' . $_GET['needle'];
    shell_exec($cmd);
}
===expect===
TaintedShell: Tainted shell command — possible command injection
