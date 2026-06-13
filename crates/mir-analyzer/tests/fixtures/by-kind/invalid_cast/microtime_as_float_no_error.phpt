===description===
microtime(true) returns float, not string|float — casting to int must not emit InvalidCast

===config===
suppress=UnusedVariable
===file===
<?php
$t = microtime(true);
$ms = (int)($t * 1000);

===expect===
