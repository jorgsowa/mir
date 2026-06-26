===description===
$this->__construct() inside __clone is not flagged (PHP re-initialization pattern)
===file===
<?php
class A {
    public function __construct() {}
    public function __clone(): void { $this->__construct(); }
}
===expect===
