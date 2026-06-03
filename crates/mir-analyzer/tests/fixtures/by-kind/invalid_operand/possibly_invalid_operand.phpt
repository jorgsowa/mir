===description===
Possibly invalid operand
===ignore===
TODO: arithmetic PossiblyInvalidOperand deferred until narrowing false-positives are resolved
===file===
<?php
$b = rand(0, 1) ? [] : 4;
echo $b + 5;
===expect===
PossiblyInvalidOperand
