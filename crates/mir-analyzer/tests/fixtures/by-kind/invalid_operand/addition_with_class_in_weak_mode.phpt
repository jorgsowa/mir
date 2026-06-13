===description===
Addition with class in weak mode
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hi" + (new stdClass);
===expect===
InvalidOperand@2:6-2:27: Operator '+' not supported between '"hi"' and 'stdClass'
