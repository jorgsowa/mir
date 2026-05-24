===description===
possiblyInvalidBitwiseNot
===file===
<?php
$a = ~(rand(0, 1) ? 2 : null);
===expect===
PossiblyInvalidOperand
===ignore===
TODO
