===description===
mir-check with simple int vs string mismatch
===file===
<?php
$x = 42;
/** @mir-check $x is string */
echo $x;
//<^^^^^^^^ TypeCheckMismatch: Type of $x is expected to be string, got int
===expect===
