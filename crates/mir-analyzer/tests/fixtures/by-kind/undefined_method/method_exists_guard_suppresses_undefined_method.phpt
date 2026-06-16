===description===
method_exists() guard suppresses UndefinedMethod inside the true branch
===config===
suppress=MissingReturnType
===file===
<?php
class Notification {}

function dispatch(Notification $n): void {
    if (method_exists($n, 'broadcastOn')) {
        $n->broadcastOn();
    }
}
===expect===
