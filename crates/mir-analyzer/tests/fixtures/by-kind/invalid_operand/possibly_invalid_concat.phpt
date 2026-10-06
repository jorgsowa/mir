===description===
Possibly invalid concat
===file===
<?php
$b = rand(0, 1) ? [] : "hello";
echo $b . "goodbye";
//   ^^^^^^^^^^^^^^ PossiblyInvalidOperand: Operator '.' might not be supported between 'array{}|"hello"' and '"goodbye"'
