===description===
arrayPushArgumentUnpackingWithBadArg
===file===
<?php
$a = [];
$b = "hello";

$a[] = "foo";

array_push($a, ...$b);
===expect===
InvalidArgument
===ignore===
TODO
