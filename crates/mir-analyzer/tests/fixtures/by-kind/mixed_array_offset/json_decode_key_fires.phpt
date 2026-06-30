===description===
MixedArrayOffset fires when json_decode() result (which is mixed) is used as array key
===config===
suppress=MixedAssignment
===file===
<?php
$key = json_decode('"hello"');
$arr = ['hello' => 1, 'world' => 2];
echo $arr[$key];
===expect===
MixedArrayOffset@4:10-4:14: Mixed type used as array offset
