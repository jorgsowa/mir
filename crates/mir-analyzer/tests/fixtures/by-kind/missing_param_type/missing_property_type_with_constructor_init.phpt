===description===
Missing property type with constructor init
===file===
<?php
class A {
    public $foo;
//  ^^^^^^^^^^^ MissingPropertyType: Property A::$foo has no type annotation

    public function __construct() {
        $this->foo = 5;
    }
}
===expect===
