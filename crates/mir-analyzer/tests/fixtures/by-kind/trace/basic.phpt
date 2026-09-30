===description===
Trace emits the inferred type of a variable via @trace in a docblock.
===config===
suppress=UnusedVariable
===file===
<?php
$x = 42;
/** @trace $x */
$y = $x + 1;
//<^^^^^^^^^^^^ Trace: Type of $x is 42
===expect===
