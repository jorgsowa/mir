===description===
gettimeofday(true) returns float, not array|float — casting to int must not emit InvalidCast

===config===
suppress=UnusedVariable
===file===
<?php
$t = gettimeofday(true);
$ms = (int)($t * 1000);

===expect===
