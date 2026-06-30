===description===
MixedArrayOffset fires when a mixed-typed function parameter is used as the array key
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param mixed $key
 */
function lookup($key): void {
    $arr = ['a' => 1, 'b' => 2, 'c' => 3];
    echo $arr[$key];
}
===expect===
MixedArrayOffset@7:14-7:18: Mixed type used as array offset
