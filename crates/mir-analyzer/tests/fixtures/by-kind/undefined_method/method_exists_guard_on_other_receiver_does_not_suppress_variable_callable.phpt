===description===
A `method_exists()` guard on a different receiver does not suppress a variable-held array callable.
===config===
suppress=MissingReturnType
===file===
<?php
class Notification {}
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function dispatch(Notification $n, Other $o): void {
    if (method_exists($o, 'broadcastOn')) {
        $cb = [$n, 'broadcastOn'];
        register_shutdown($cb);
//                        ^^^ UndefinedMethod: Method Notification::broadcastOn() does not exist
    }
}
===expect===
