===description===
Direct constructor call on this
===file===
<?php
class A {
    public function __construct() {}
    public function f(): void { $this->__construct(); }
}
$a = new A;
$a->f();

===expect===
DirectConstructorCall@4:33-4:53: Cannot call constructor of A directly
