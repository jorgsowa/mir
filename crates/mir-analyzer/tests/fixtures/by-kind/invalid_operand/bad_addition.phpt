===description===
Bad addition
===config===
suppress=UnusedVariable
===file===
<?php
$a = "b" + 5;
===expect===
InvalidOperand@2:6-2:13: Operator '+' not supported between '"b"' and '5'
