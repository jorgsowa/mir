===description===
Array destructuring invalid list
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$a = 42;

list($id1, $name1) = $a;
===expect===
