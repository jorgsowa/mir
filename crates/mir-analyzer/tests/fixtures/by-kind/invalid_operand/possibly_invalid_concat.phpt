===description===
Possibly invalid concat
===file===
<?php
$b = rand(0, 1) ? [] : "hello";
echo $b . "goodbye";
===expect===
PossiblyInvalidOperand@3:6-3:20: Operator '.' might not be supported between 'array{}|"hello"' and '"goodbye"'
