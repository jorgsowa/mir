===description===
Trace variable
===config===
suppress=UnusedVariable
===file===
<?php
/** @trace $a */
$a = getmypid();
===expect===
Trace@3:0-3:16: Type of $a is mixed
