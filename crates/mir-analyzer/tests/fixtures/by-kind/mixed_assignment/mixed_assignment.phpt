===description===
Mixed assignment
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed */
$a = 5;
$b = $a;
//<^^^^^^^ MixedAssignment: Variable $b is assigned a mixed type
===expect===
