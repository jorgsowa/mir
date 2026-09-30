===description===
FN: prefix -- never checked its operand, unlike postfix --.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
--$a;
//^^ InvalidOperand: Operator '--' not supported for operand of type '"hello"'
===expect===
