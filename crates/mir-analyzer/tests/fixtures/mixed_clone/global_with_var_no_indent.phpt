===description===
globalWithVarNoIndent
===file===
<?php
/** @var mixed $a */
$a = 5;
clone $a;
===expect===
MixedClone@4:1: cannot clone mixed
