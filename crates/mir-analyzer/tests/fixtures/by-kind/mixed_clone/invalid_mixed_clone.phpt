===description===
Invalid mixed clone
===file===
<?php
/** @var mixed $a */
$a = 5;
/** @mir-check $a is mixed */
clone $a;
//<^^^^^^^^ MixedClone: cannot clone mixed
