===description===
Trace variables comma
===file===
<?php
/** @trace $a, $b */
$a = getmypid();
$b = getmypid();
===expect===
Trace
===ignore===
TODO
