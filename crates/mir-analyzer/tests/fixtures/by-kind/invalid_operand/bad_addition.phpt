===description===
Bad addition
===config===
suppress=UnusedVariable
===file===
<?php
$a = "b" + 5;
===expect===
InvalidOperand@2:5-2:12: Operator '+' not supported between '"b"' and '5'
