===description===
Possibly div by zero
===config===
suppress=UnusedVariable
===file===
<?php
$a = 5 / (rand(0, 1) ? 2 : null);
===expect===
PossiblyNullOperand@2:5-2:32: Operator '/' operand '2|null' might be null
