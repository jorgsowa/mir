===file===
<?php
function test(): void {
    $v = false;
    [$a] = $v;
    var_dump($a);
}
===expect===
