===description===
$this->__construct() in a helper called from __unserialize still fires (exemption is method-direct only)
===file===
<?php
class A {
    public function __construct() {}
    public function init(): void {
        $this->__construct();
    }
    /** @param array<string,mixed> $data */
    public function __unserialize(array $data): void {
        $this->init();
    }
}
===expect===
DirectConstructorCall@5:8-5:28: Cannot call constructor of A directly
