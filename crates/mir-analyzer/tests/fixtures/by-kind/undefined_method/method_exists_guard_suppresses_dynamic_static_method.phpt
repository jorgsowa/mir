===description===
method_exists($cls, 'method') guard suppresses UndefinedMethod for a
dynamic class-string static call ($cls::method()) inside the true branch.
===file===
<?php
class Notification {}

/** @param class-string<Notification> $cls */
function dispatch(string $cls): void {
    if (method_exists($cls, 'broadcastOn')) {
        $cls::broadcastOn();
    }
}
===expect===
