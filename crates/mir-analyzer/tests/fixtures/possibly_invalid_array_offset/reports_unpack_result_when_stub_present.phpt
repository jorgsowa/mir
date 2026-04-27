===file===
<?php
function test(): void {
    [$a] = unpack('N', pack('N', 1));
    var_dump($a);
}
===expect===
PossiblyInvalidArgument: Argument $string of unpack() expects 'string', possibly different type 'string|false' provided
PossiblyInvalidArrayOffset: Array offset might be invalid: expects 'array', got 'array<int, mixed>|false'
