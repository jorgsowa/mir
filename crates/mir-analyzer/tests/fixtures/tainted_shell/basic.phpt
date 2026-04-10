===source===
<?php
function test(): void {
    $cmd = $_GET['cmd'];
    exec($cmd);
}
===expect===
TaintedShell: exec($cmd)
