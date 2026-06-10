===description===
Trace variable
===file===
<?php
/** @trace $a */
$a = getmypid();
===expect===
Trace@3:1-3:17: Type of $a is mixed
