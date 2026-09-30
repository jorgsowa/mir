===description===
Missing property type with constructor init and null default
===file===
<?php
class A {
    public $foo = null;
//  ^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property A::$foo has no type annotation

    public function __construct() {
        $this->foo = 5;
    }
}
===expect===
