===description===
possiblyInvalidOperand
===file===
<?php
$b = rand(0, 1) ? [] : 4;
echo $b + 5;
===expect===
PossiblyInvalidOperand
===ignore===
TODO
