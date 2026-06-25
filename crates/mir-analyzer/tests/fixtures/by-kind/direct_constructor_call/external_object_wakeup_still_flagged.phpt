===description===
Calling __construct() on an external object from __wakeup is still flagged — exemption is only for $this.
===file===
<?php
class Foo {
    public function __construct() {}

    public function __wakeup(): void {
        $other = new Foo();
        $other->__construct();
    }
}
===expect===
DirectConstructorCall@7:8-7:29: Cannot call constructor of Foo directly
