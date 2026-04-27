===file===
<?php
function test(): void {
    $r = unpack('N*', pack('N*', 1));
    var_dump($r);
}
===expect===
PossiblyInvalidArgument: Argument $string of unpack() expects 'string', possibly different type 'string|false' provided
