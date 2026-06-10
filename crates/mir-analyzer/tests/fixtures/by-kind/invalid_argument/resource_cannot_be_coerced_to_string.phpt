===description===
Resource cannot be coerced to string
===file===
<?php
/** @mutation-free */
function takesString(string $s) : void {}
$a = fopen("php://memory", "r");
takesString($a);
===expect===
