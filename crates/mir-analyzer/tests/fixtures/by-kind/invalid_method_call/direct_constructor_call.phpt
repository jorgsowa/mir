===description===
Direct constructor call
===file===
<?php
class A {
    public function __construct() {}
}
$a = new A;
$a->__construct();

===expect===
DirectConstructorCall@6:1-6:18: Cannot call constructor of A directly
