===description===
InvalidArrayOffset fires when an array is used as an array key.
===file===
<?php
$arr = [1, 2, 3];
$key = [0, 1];
echo $arr[$key];
===expect===
InvalidArrayOffset@4:11-4:15: Array offset expects 'array-key', got 'array{0: 0, 1: 1}'
