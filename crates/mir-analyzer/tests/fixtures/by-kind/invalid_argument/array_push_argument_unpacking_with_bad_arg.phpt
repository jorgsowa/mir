===description===
Array push argument unpacking with bad arg
===file===
<?php
$a = [];
$b = "hello";

$a[] = "foo";

array_push($a, ...$b);
//             ^^^^^ InvalidUnpack: Only arrays and Traversables can be unpacked, string given
