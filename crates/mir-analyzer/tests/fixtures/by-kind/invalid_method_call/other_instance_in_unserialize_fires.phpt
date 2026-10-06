===description===
Calling __construct() on another instance inside __unserialize still fires (exemption is $this-only)
===file===
<?php
class A {
    public function __construct() {}
    /** @param array<string,mixed> $data */
    public function __unserialize(array $data): void {
        $other = new A();
        $other->__construct();
//      ^^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of A directly
    }
}
