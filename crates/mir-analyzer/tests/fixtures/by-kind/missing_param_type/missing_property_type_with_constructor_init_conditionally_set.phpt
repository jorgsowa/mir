===description===
Missing property type with constructor init conditionally set
===file===
<?php
class A {
    public $foo;
//  ^^^^^^^^^^^ MissingPropertyType: Property A::$foo has no type annotation

    public function __construct() {
        if (rand(0, 1)) {
            $this->foo = 5;
        }
    }
}
