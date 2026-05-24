===description===
traceVariablesComma
===file===
<?php
/** @psalm-trace $a, $b */
$a = getmypid();
$b = getmypid();
===expect===
Trace
===ignore===
TODO
