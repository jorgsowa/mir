===description===
InvalidArrayOffset does NOT fire for valid array key types (int, string).
===file===
<?php
$arr = ["a" => 1, "b" => 2];
$key = "a";
echo $arr[$key];
===expect===
