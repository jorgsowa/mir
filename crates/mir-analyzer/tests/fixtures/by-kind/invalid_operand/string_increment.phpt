===description===
String increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$a++;
//<^^ InvalidOperand: Operator '++' not supported for operand of type '"hello"'
===expect===
