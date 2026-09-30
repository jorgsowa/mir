===description===
`method_exists()` does not suppress callable variables.
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
//                        ^^^ UndefinedMethod: Method Notification::broadcastOn() does not exist
    }
}
===expect===
