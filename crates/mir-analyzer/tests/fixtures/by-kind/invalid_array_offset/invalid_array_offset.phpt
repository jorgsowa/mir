===description===
Invalid array offset
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$x = ["a"];
$y = $x["b"];
//      ^^^ NonExistentArrayOffset: Array offset 'b' does not exist
===expect===
