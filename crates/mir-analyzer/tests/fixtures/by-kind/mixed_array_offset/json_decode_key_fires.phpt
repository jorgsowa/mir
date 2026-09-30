===description===
MixedArrayOffset fires when json_decode() result (which is mixed) is used as array key
===config===
suppress=MixedAssignment
===file===
<?php
$key = json_decode('"hello"');
$arr = ['hello' => 1, 'world' => 2];
echo $arr[$key];
//        ^^^^ MixedArrayOffset: Mixed type used as array offset
===expect===
