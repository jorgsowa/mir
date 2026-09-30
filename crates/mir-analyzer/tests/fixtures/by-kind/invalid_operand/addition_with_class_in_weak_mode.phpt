===description===
Addition with class in weak mode
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hi" + (new stdClass);
//   ^^^^^^^^^^^^^^^^^^^^^ InvalidOperand: Operator '+' not supported between '"hi"' and 'stdClass'
===expect===
