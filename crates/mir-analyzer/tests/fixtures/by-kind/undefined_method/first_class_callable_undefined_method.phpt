===description===
FirstClassCallable:UndefinedMethod
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$queue = new SplQueue;
$closure = $queue->undefined(...);
$count = $closure();

===expect===
