===description===
does not report destructure of only false
===file===
<?php
function test(): void {
    $v = false;
    [$a] = $v;
    var_dump($a);
}
===expect===
