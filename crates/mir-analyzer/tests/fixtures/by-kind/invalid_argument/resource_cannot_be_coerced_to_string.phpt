===description===
Resource cannot be coerced to string
===ignore===
TODO
===file===
<?php
/** @mutation-free */
function takesString(string $s) : void {}
$a = fopen("php://memory", "r");
takesString($a);
===expect===
