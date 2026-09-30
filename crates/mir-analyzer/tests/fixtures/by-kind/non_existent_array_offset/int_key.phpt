===description===
Accessing a non-existent int key in a list array
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$x = ["a"];
$y = $x["b"];
//      ^^^ NonExistentArrayOffset: Array offset 'b' does not exist
===expect===
