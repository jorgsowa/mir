===description===
Invalid g m p operation
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$a = gmp_init(2);
$b = "a" + $a;
===expect===
InvalidOperand@3:5-3:13: Operator '+' not supported between '"a"' and 'mixed'
