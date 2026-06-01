===description===
UndefinedProperty suppressed when property is guarded by ?? operator
===file===
<?php
class Mailable {
    protected function newQueuedJob(): mixed {
        $messageGroup = $this->messageGroup ?? null;
        return $messageGroup;
    }
}
===expect===
