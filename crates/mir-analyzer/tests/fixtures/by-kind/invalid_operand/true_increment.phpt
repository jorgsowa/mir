===description===
True increment
===config===
suppress=UnusedVariable
===file===
<?php
$a = true;
$a++;
//<^^ InvalidOperand: Operator '++' not supported for operand of type 'true'
===expect===
