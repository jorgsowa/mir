===description===
Invalid array offset
===file===
<?php
$x = ["a"];
$y = $x["b"];
===expect===
NonExistentArrayOffset@3:9-3:12: Array offset 'b' does not exist
