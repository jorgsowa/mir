===description===
Invalid exponent
===config===
suppress=UnusedVariable
===file===
<?php
$a = "x" ^ 1;
===expect===
InvalidOperand@2:5-2:12: Operator '^' not supported between '"x"' and '1'
