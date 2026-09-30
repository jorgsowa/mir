===description===
Direct constructor call on this
===file===
<?php
class A {
    public function __construct() {}
    public function f(): void { $this->__construct(); }
//                              ^^^^^^^^^^^^^^^^^^^^ DirectConstructorCall: Cannot call constructor of A directly
}
$a = new A;
$a->f();

===expect===
