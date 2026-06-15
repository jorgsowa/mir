===description===
Invalid mixed clone
===file===
<?php
/** @var mixed $a */
$a = 5;
/** @mir-check $a is mixed */
clone $a;
===expect===
MixedClone@5:0-5:8: cannot clone mixed
