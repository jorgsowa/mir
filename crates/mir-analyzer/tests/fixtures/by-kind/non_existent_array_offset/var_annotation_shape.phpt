===description===
Bare @var shape annotation on assignment is honored for subsequent key accesses
===file===
<?php
$a = ["k" => 1];
/** @var array{id: int, name: string} */
$b = $a;
echo $b["id"];
echo $b["name"];
===expect===
