===description===
$this->__construct() inside __unserialize fires (__unserialize is not in the exemption list)
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
DirectConstructorCall@6:8-6:28: Cannot call constructor of A directly
