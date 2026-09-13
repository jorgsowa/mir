===description===
`method_exists()` does not suppress array callables outside its branch.
===config===
suppress=MissingReturnType
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
}
===expect===
UndefinedMethod@11:22-11:41: Method Notification::broadcastOn() does not exist
