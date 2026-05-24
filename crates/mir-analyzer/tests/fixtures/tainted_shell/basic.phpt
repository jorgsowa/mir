===description===
basic
===file===
<?php
function test(): void {
    $cmd = $_GET['cmd'];
    exec($cmd);
}
===expect===
TaintedShell@4:5: Tainted shell command — possible command injection
