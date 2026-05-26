===description===
subclass passed to a function where template is bound to the parent class should pass
===file===
<?php
interface Event {}

final class MyEvent implements Event {}

/**
 * @template E of Event
 */
function dispatch(Event $e): void {
    echo (string) $e;
}

dispatch(new MyEvent());
===expect===
