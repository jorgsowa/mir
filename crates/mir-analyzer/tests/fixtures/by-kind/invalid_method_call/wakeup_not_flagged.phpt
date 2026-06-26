===description===
$this->__construct() inside __wakeup is not flagged (PHP re-initialization pattern)
===file===
<?php
class A {
    public function __construct() {}
    public function __wakeup(): void { $this->__construct(); }
}
===expect===
