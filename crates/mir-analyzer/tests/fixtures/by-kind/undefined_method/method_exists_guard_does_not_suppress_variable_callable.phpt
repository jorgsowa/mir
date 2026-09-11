===description===
method_exists() guard does not suppress a variable holding the callable (its value may have been assigned elsewhere)
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
        $cb = [$n, 'broadcastOn'];
        register_shutdown($cb);
    }
}
===expect===
UndefinedMethod@11:26-11:29: Method Notification::broadcastOn() does not exist
