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
    }
}
===expect===
DirectConstructorCall@7:8-7:29: Cannot call constructor of A directly
