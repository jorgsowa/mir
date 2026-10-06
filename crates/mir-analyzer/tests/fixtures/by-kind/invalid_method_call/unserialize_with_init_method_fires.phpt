===description===
$this->__construct() in a helper called from __unserialize still fires (exemption is method-direct only)
===file===
<?php
class A {
    public function __construct() {}
    public function init(): void {
        $this->__construct();
//      ^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of A directly
    }
    /** @param array<string,mixed> $data */
    public function __unserialize(array $data): void {
        $this->init();
    }
}
