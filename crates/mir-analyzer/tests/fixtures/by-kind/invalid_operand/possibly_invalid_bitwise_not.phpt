===description===
Possibly invalid bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~(rand(0, 1) ? 2 : null);
===expect===
PossiblyNullOperand@2:6-2:29: Operator '~' operand '2|null' might be null
