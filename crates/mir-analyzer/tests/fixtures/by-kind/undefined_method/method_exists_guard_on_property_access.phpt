===description===
method_exists() guard on a typed property suppresses UndefinedMethod for that object
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
