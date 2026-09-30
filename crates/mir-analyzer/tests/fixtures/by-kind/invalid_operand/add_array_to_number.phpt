===description===
Add array to number
===config===
suppress=UnusedVariable
===file===
<?php
$a = [1] + 1;
//   ^^^^^^^ InvalidOperand: Operator '+' not supported between 'array{0: 1}' and '1'
===expect===
