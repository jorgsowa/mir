===description===
directConstructorCall
===file===
<?php
class A {
    public function __construct() {}
}
$a = new A;
$a->__construct();

===expect===
DirectConstructorCall
===ignore===
TODO
