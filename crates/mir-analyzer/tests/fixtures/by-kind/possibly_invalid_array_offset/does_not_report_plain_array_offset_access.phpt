===description===
does not report plain array offset access
===config===
suppress=ForbiddenCode
===file===
<?php
function test(): void {
    $arr = [1, 2, 3];
    $x = $arr[0];
    var_dump($x);
}
===expect===
