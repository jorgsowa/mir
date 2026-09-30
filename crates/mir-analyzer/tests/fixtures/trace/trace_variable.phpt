===description===
Trace variable
===config===
suppress=UnusedVariable
===file===
<?php
/** @trace $a */
$a = getmypid();
//<^^^^^^^^^^^^^^^^ Trace: Type of $a is mixed
===expect===
