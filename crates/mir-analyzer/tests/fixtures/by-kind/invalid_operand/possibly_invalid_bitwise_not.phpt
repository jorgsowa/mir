===description===
Possibly invalid bitwise not
===config===
suppress=UnusedVariable
===file===
<?php
$a = ~(rand(0, 1) ? 2 : null);
//    ^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullOperand: Operator '~' operand '2|null' might be null
===expect===
