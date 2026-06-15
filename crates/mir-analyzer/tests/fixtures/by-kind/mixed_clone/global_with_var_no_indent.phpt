===description===
Global with var no indent
===file===
<?php
/** @var mixed $a */
$a = 5;
clone $a;
===expect===
MixedClone@4:0-4:8: cannot clone mixed
