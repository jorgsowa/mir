===source===
<?php
function test(): void {
    $cmd = escapeshellarg($_GET['cmd']);
    exec($cmd);
}
===expect===
