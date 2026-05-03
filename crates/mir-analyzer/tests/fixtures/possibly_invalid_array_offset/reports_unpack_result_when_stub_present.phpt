===description===
reports unpack result when stub present
===file===
<?php
function test(): void {
    [$a] = unpack('N', pack('N', 1));
    var_dump($a);
}
===expect===
PossiblyInvalidArrayOffset@3:4: Array offset might be invalid: expects 'array', got 'array<int, mixed>|false'
PossiblyInvalidArgument@3:23: Argument $string of unpack() expects 'string', possibly different type 'string|false' provided
===ignore===
TODO
