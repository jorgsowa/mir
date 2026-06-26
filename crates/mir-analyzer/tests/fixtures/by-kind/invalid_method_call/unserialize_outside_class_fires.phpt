===description===
$this->__construct() called directly from a non-lifecycle method fires regardless of class
===file===
<?php
class A {
    public function __construct() {}
    public function restore(): void {
        $this->__construct();
    }
}
===expect===
DirectConstructorCall@5:8-5:28: Cannot call constructor of A directly
