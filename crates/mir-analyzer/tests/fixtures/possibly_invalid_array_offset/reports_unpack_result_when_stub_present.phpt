===source===
<?php
function test(): void {
    [$a] = unpack('N', pack('N', 1));
    var_dump($a);
}
===expect===
PossiblyInvalidArrayOffset: [$a] = unpack('N', pack('N', 1))
