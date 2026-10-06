===description===
method_exists(Foo::class, 'method') guard does not suppress UndefinedMethod
for a static call outside the guarded branch.
===file===
<?php
class Notification {}

function dispatch(): void {
    Notification::broadcastOn();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Notification::broadcastOn() does not exist
}
