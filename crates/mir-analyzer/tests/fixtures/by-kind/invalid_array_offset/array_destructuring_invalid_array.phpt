===description===
Array destructuring invalid array
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$a = 42;

[$id2, $name2] = $a;
===expect===
