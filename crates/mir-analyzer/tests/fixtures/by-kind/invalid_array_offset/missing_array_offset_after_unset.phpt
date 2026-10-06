===description===
Missing array offset after unset
===file===
<?php
$x = ["a" => "value", "b" => "value"];
unset($x["a"]);
echo $x["a"];
//      ^^^ NonExistentArrayOffset: Array offset 'a' does not exist
