===description===
Mixed assignment
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed */
$a = 5;
$b = $a;
===expect===
MixedAssignment@4:0-4:7: Variable $b is assigned a mixed type
