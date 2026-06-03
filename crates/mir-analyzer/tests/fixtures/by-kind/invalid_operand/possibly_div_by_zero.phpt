===description===
Possibly div by zero
===file===
<?php
$a = 5 / (rand(0, 1) ? 2 : null);
===expect===
PossiblyNullOperand@2:6-2:33: Operator '/' operand '2|null' might be null
