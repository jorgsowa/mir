===description===
Missing array offset after unset
===file===
<?php
$x = ["a" => "value", "b" => "value"];
unset($x["a"]);
echo $x["a"];
===expect===
NonExistentArrayOffset@4:8-4:11: Array offset 'a' does not exist
