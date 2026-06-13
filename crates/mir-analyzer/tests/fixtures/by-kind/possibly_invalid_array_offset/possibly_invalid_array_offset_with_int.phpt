===description===
Possibly invalid array offset with int
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$x = rand(0, 5) > 2 ? ["a" => 5] : "hello";
$y = $x[0];
===expect===
NonExistentArrayOffset@3:9-3:10: Array offset '0' does not exist
