===description===
invalidMixedClone
===file===
<?php
/** @var mixed $a */
$a = 5;
/** @mir-check $a is mixed */
clone $a;
===expect===
MixedClone@5:1: cannot clone mixed
