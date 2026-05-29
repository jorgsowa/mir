===description===
mir-check emits TypeCheckMismatch when type does not match
===file===
<?php
$x = 42;
/** @mir-check $x is string */
echo $x;
===expect===
TypeCheckMismatch@4:1-4:9: Type of $x is expected to be string, got int
