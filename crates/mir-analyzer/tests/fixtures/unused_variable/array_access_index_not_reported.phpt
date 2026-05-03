===description===
array access index not reported
===file===
<?php
function test(array $arr): mixed {
    $keys = array_keys($arr);
    return $arr[$keys[0]];
}
===expect===
===ignore===
TODO
