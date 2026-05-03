===description===
unpack not reported
===file===
<?php
function test(): void {
    $r = unpack('N*', pack('N*', 1));
    var_dump($r);
}
===expect===
PossiblyInvalidArgument@3:22: Argument $string of unpack() expects 'string', possibly different type 'string|false' provided
===ignore===
TODO
