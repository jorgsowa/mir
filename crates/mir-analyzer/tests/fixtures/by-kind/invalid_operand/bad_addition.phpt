===description===
Bad addition
===config===
suppress=UnusedVariable
===file===
<?php
$a = "b" + 5;
//   ^^^^^^^ InvalidOperand: Operator '+' not supported between '"b"' and '5'
===expect===
