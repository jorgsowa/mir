===description===
mir-check passes when type matches exactly
===file===
<?php
$x = "hello";
/** @mir-check $x is string */
echo $x;
===expect===
