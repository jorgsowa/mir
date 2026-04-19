===source===
<?php
function test(): void {
    [$a] = unpack('N', pack('N', 1));
    var_dump($a);
}
===expect===
PossiblyInvalidArrayOffset: Array offset might be invalid: expects 'array', got 'array<int, mixed>|false'
