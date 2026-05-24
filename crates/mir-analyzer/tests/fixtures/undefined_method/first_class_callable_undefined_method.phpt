===description===
FirstClassCallable:UndefinedMethod
===file===
<?php
$queue = new SplQueue;
$closure = $queue->undefined(...);
$count = $closure();

===expect===
UndefinedMethod
===ignore===
TODO
