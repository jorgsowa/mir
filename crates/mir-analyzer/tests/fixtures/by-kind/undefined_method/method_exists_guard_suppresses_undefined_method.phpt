===description===
method_exists() guard suppresses UndefinedMethod inside the true branch
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Notification {}

function dispatch(Notification $n): void {
    if (method_exists($n, 'broadcastOn')) {
        $n->broadcastOn();
    }
}
