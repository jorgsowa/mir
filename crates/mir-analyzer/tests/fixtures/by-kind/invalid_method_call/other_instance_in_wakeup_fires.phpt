===description===
Calling __construct() on another instance inside __wakeup fires (exemption is only for $this)
===file===
<?php
class A {
    public function __construct() {}
    public function __wakeup(): void {
        $other = new A;
        $other->__construct();
    }
}
===expect===
DirectConstructorCall@6:8-6:29: Cannot call constructor of A directly
