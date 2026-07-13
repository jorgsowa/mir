===description===
method_exists(Foo::class, 'method') guard suppresses UndefinedMethod for
the static call inside the true branch.
===file===
<?php
class Notification {}

function dispatch(): void {
    if (method_exists(Notification::class, 'broadcastOn')) {
        Notification::broadcastOn();
    }
}
===expect===
