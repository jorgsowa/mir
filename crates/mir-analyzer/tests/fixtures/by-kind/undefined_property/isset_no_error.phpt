===description===
UndefinedProperty suppressed when property is guarded by isset()
===file===
<?php
class Mailable {
    protected function hasGroup(): bool {
        return isset($this->messageGroup);
    }
}
===expect===
