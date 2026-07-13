===description===
FN: unary `-`/`+` never checked for a non-numeric operand, unlike binary
arithmetic and unary `~`.
===config===
suppress=UnusedVariable
===file===
<?php
$a = [1, 2];
$b = -$a;
===expect===
InvalidOperand@3:6-3:8: Operator '-' not supported between 'array{0: 1, 1: 2}' and ''
