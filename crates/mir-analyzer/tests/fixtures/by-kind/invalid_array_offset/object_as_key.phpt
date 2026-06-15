===description===
InvalidArrayOffset fires when an object is used as an array key.
===file===
<?php
$arr = ["a" => 1, "b" => 2];
$obj = new stdClass();
echo $arr[$obj];
===expect===
InvalidArrayOffset@4:10-4:14: Array offset expects 'array-key', got 'stdClass'
