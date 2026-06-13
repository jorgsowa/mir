===description===
String function call
===config===
suppress=UnusedVariable
===file===
<?php
$bad_one = "hello";
$a = $bad_one(1);
===expect===
MixedAssignment@3:1-3:17: Variable $a is assigned a mixed type
