===description===
Invalid g m p operation
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$a = gmp_init(2);
$b = "a" + $a;
//   ^^^^^^^^ InvalidOperand: Operator '+' not supported between '"a"' and 'mixed'
===expect===
