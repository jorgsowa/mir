===description===
Trace variables
===config===
suppress=UnusedVariable
===file===
<?php
/** @trace $a $b */
$a = getmypid();
//<^^^^^^^^^^^^^^^^ Trace: Type of $a is mixed
$b = getmypid();
===expect===
