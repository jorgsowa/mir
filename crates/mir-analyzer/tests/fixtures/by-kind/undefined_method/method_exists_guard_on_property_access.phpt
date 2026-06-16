===description===
method_exists() guard on a typed property suppresses UndefinedMethod for that object
===config===
suppress=MissingReturnType
===file===
<?php
class Notification {}

class Queue {
    public Notification $notification;

    public function __construct() {
        $this->notification = new Notification();
    }
}

function send(Queue $q): void {
    if (method_exists($q->notification, 'via')) {
        $q->notification->via('mail');
    }
}
===expect===
