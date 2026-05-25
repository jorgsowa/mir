===description===
Trace variables
===file===
<?php
/** @trace $a $b */
$a = getmypid();
$b = getmypid();
===expect===
Trace
===ignore===
TODO
