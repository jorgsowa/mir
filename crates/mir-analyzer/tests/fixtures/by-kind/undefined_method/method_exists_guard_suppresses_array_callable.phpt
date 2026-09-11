===description===
method_exists() guard suppresses UndefinedMethod for a [$obj, 'method'] array callable
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
        register_shutdown([$n, 'broadcastOn']);
    }
}
===expect===
