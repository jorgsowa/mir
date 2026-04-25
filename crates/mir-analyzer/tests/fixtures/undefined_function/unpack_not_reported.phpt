===file===
<?php
function test(): void {
    $r = unpack('N*', pack('N*', 1));
    var_dump($r);
}
===expect===
