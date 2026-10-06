===description===
$this->__construct() called directly from a non-lifecycle method fires regardless of class
===file===
<?php
class A {
    public function __construct() {}
    public function restore(): void {
        $this->__construct();
//      ^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of A directly
    }
}
