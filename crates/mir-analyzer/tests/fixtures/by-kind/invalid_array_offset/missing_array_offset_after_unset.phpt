===description===
Missing array offset after unset
===ignore===
TODO
===file===
<?php
$x = ["a" => "value", "b" => "value"];
unset($x["a"]);
echo $x["a"];
===expect===
