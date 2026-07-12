===description===
UndefinedProperty suppressed when property is unset via unset(), matching isset()
===file===
<?php
class Mailable {
    protected function hasGroup(): void {
        unset($this->messageGroup);
    }
}
===expect===
