===description===
unpack not reported
===config===
suppress=ForbiddenCode
===file===
<?php
function test(): void {
    $r = unpack('N*', pack('N*', 1));
    var_dump($r);
}
===expect===
PossiblyInvalidArgument@3:23-3:36: Argument $string of unpack() expects 'string', possibly different type 'string|false' provided
