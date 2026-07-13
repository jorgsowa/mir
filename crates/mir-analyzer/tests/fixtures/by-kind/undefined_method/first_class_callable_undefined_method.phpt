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
UndefinedMethod@3:19-3:28: Method SplQueue::undefined() does not exist
