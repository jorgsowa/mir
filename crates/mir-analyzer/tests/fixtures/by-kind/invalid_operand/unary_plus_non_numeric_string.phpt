===description===
FN: unary `+` never checked for a non-numeric operand, unlike binary
arithmetic and unary `~`.
===config===
suppress=UnusedVariable
===file===
<?php
$a = +"abc";
//    ^^^^^ InvalidOperand: Operator '+' not supported for operand of type '"abc"'
===expect===
