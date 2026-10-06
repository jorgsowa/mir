===description===
InvalidArrayOffset fires when an object is used as an array key.
===file===
<?php
$arr = ["a" => 1, "b" => 2];
$obj = new stdClass();
echo $arr[$obj];
//        ^^^^ InvalidArrayOffset: Array offset expects 'array-key', got 'stdClass'
