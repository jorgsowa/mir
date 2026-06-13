===description===
mir-check detects int vs string type mismatch
===config===
suppress=UnusedVariable
===file===
<?php
$x = 5;
/** @mir-check $x is int */
$x = "hello";
/** @mir-check $x is string */
echo $x;
===expect===
