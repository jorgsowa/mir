===description===
Invalid g m p operation
===file===
<?php
$a = gmp_init(2);
$b = "a" + $a;
===expect===
InvalidOperand
===ignore===
TODO
