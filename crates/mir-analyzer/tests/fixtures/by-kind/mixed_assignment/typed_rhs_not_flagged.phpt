===description===
MixedAssignment does NOT fire when the right-hand side has a concrete type.
===config===
suppress=UnusedVariable
===file===
<?php
$a = 42;
$b = $a;

===expect===
