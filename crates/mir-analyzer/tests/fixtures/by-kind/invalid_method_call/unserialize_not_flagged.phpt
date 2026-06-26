===description===
$this->__construct() inside __unserialize is not flagged (PHP 8.0+ re-initialization pattern)
===file===
<?php
class A {
    public function __construct() {}
    /** @param array<string,mixed> $data */
    public function __unserialize(array $data): void {
        $this->__construct();
    }
}
===expect===
