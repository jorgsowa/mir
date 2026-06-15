===description===
Trace variables
===config===
suppress=UnusedVariable
===file===
<?php
/** @trace $a $b */
$a = getmypid();
$b = getmypid();
===expect===
Trace@3:0-3:16: Type of $a is mixed
