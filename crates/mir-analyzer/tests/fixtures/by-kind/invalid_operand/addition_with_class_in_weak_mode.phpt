===description===
Addition with class in weak mode
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hi" + (new stdClass);
===expect===
InvalidOperand@2:5-2:26: Operator '+' not supported between '"hi"' and 'stdClass'
