===description===
sanitized not reported
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function test(): void {
    $cmd = escapeshellarg($_GET['cmd']);
    exec($cmd);
}
===expect===
