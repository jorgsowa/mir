===description===
Invalid array offset
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$x = ["a"];
$y = $x["b"];
===expect===
NonExistentArrayOffset@3:8-3:11: Array offset 'b' does not exist
