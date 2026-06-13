===description===
Resource cannot be coerced to string
===config===
suppress=MixedArgument,MixedAssignment,UnusedParam
===file===
<?php
/** @mutation-free */
function takesString(string $s) : void {}
$a = fopen("php://memory", "r");
takesString($a);
===expect===
