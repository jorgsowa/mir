===description===
$this->__construct() in __wakeup is not flagged even when the class extends another.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    public function __construct(protected string $data) {}
}

class Child extends Base {
    public function __wakeup(): void {
        $this->__construct('restored');
    }
}
===expect===
