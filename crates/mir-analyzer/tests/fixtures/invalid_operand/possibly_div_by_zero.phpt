===description===
Possibly div by zero
===file===
<?php
$a = 5 / (rand(0, 1) ? 2 : null);
===expect===
PossiblyNullOperand
===ignore===
TODO
