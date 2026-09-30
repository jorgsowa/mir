===description===
`method_exists()` on the receiver suppresses UndefinedMethod for an array callable held in a variable.
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
