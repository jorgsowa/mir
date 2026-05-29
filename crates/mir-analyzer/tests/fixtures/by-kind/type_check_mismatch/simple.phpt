===description===
mir-check with simple int vs string mismatch
===file===
<?php
$x = 42;
/** @mir-check $x is string */
echo $x;
===expect===
TypeCheckMismatch@4:1-4:9: Type of $x is expected to be string, got int
