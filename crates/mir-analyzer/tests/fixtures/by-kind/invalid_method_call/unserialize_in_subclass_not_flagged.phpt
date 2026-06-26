===description===
$this->__construct() in a subclass __unserialize is not flagged
===file===
<?php
class Base {
    public function __construct() {}
}
class Child extends Base {
    /** @param array<string,mixed> $data */
    public function __unserialize(array $data): void {
        $this->__construct();
    }
}
===expect===
