===description===
FirstClassCallable:UndefinedMethod
===ignore===
TODO
===file===
<?php
$queue = new SplQueue;
$closure = $queue->undefined(...);
$count = $closure();

===expect===
