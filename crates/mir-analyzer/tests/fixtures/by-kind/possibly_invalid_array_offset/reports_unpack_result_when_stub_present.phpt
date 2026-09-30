===description===
reports unpack result when stub present
===config===
suppress=ForbiddenCode,MixedAssignment
===file===
<?php
function test(): void {
    [$a] = unpack('N', pack('N', 1));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArrayOffset: Array offset might be invalid: expects 'array', got 'array<int, mixed>|false'
    var_dump($a);
}
===expect===
