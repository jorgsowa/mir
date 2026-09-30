===description===
String function call
===config===
suppress=UnusedVariable
===file===
<?php
$bad_one = "hello";
$a = $bad_one(1);
//<^^^^^^^^^^^^^^^^ MixedAssignment: Variable $a is assigned a mixed type
===expect===
