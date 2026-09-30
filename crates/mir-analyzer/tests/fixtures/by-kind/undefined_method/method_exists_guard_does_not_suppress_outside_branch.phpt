===description===
method_exists() guard does not suppress UndefinedMethod outside the guarded branch
===file===
<?php
class Notification {}

function dispatch(Notification $n): void {
    $n->broadcastOn();
//  ^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Notification::broadcastOn() does not exist
}
===expect===
