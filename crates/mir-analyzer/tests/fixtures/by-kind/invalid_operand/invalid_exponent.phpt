===description===
Invalid exponent
===file===
<?php
$a = "x" ^ 1;
===expect===
InvalidOperand@2:6-2:13: Operator '^' not supported between '"x"' and '1'
