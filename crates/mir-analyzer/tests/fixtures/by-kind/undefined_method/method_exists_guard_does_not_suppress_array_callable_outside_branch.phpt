===description===
`method_exists()` does not suppress array callables outside its branch.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Notification {}

function register_shutdown(callable $cb): void {
    $cb();
}

function dispatch(Notification $n): void {
    if (method_exists($n, 'broadcastOn')) {
    }
    register_shutdown([$n, 'broadcastOn']);
//                    ^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Notification::broadcastOn() does not exist
}
