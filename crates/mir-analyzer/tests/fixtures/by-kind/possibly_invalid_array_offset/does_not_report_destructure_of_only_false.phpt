===description===
does not report destructure of only false
===config===
suppress=ForbiddenCode,MixedAssignment
===file===
<?php
function test(): void {
    $v = false;
    [$a] = $v;
    var_dump($a);
}
===expect===
