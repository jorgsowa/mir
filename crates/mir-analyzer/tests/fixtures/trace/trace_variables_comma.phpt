===description===
Trace variables comma
===file===
<?php
/** @trace $a, $b */
$a = getmypid();
$b = getmypid();
===expect===
Trace@3:1-3:17: Type of $a is mixed
