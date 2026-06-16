===description===
method_exists() guard does not suppress UndefinedMethod outside the guarded branch
===file===
<?php
class Notification {}

function dispatch(Notification $n): void {
    $n->broadcastOn();
}
===expect===
UndefinedMethod@5:4-5:21: Method Notification::broadcastOn() does not exist
