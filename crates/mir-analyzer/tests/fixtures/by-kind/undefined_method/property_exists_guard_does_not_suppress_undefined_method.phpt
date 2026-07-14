===description===
property_exists() does not suppress UndefinedMethod — properties and methods
are independent namespaces, unlike method_exists().
===file===
<?php
class Notification {}

function dispatch(Notification $n): void {
    if (property_exists($n, 'broadcastOn')) {
        $n->broadcastOn();
    }
}
===expect===
UndefinedMethod@6:8-6:25: Method Notification::broadcastOn() does not exist
