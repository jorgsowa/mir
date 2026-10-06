===description===
Global with var no indent
===file===
<?php
/** @var mixed $a */
$a = 5;
clone $a;
//<^^^^^^^^ MixedClone: cannot clone mixed
