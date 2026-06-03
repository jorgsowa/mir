===description===
Invalid g m p operation
===file===
<?php
$a = gmp_init(2);
$b = "a" + $a;
===expect===
InvalidOperand@3:6-3:14: Operator '+' not supported between '"a"' and 'mixed'
