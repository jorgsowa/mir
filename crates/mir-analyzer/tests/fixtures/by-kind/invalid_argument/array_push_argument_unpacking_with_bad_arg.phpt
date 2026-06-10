===description===
Array push argument unpacking with bad arg
===ignore===
TODO
===file===
<?php
$a = [];
$b = "hello";

$a[] = "foo";

array_push($a, ...$b);
===expect===
