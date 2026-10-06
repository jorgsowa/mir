===description===
FirstClassCallable:UndefinedMethod
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$queue = new SplQueue;
$closure = $queue->undefined(...);
//                 ^^^^^^^^^ UndefinedMethod: Method SplQueue::undefined() does not exist
$count = $closure();
