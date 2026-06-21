===description===
XOR with an array operand is invalid; string literals are valid (PHP allows string bitwise ops)
===config===
suppress=UnusedVariable
===file===
<?php
$a = [1, 2] ^ 1;
===expect===
InvalidOperand@2:5-2:15: Operator '^' not supported between 'array{0: 1, 1: 2}' and '1'
