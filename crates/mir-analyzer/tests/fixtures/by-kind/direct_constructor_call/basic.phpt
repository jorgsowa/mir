===description===
DirectConstructorCall fires when __construct() is called directly.
===file===
<?php
class Foo {
    public function __construct() {}
}

$a = new Foo();
$a->__construct();
===expect===
DirectConstructorCall@7:1-7:18: Cannot call constructor of Foo directly
