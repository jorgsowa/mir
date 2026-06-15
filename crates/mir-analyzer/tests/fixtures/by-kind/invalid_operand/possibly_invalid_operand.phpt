===description===
Possibly invalid operand
===file===
<?php
$b = rand(0, 1) ? [] : 4;
echo $b + 5;
===expect===
PossiblyInvalidOperand@3:5-3:11: Operator '+' might not be supported between 'array{}|4' and '5'
