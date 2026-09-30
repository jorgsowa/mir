===description===
FirstClassCallable:UndefinedMethod
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$queue = new SplQueue;
$closure = $queue->undefined(...);
//                 ^^^^^^^^^ UndefinedMethod: Method SplQueue::undefined() does not exist
$count = $closure();

===expect===
