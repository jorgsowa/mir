===description===
Possibly invalid concat
===file===
<?php
$b = rand(0, 1) ? [] : "hello";
echo $b . "goodbye";
===expect===
PossiblyInvalidOperand@3:5-3:19: Operator '.' might not be supported between 'array{}|"hello"' and '"goodbye"'
