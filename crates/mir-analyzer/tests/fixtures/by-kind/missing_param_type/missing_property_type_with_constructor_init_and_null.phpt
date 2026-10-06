===description===
Missing property type with constructor init and null
===file===
<?php
class A {
    public $foo;
//  ^^^^^^^^^^^ MissingPropertyType: Property A::$foo has no type annotation

    public function __construct() {
        $this->foo = 5;
    }

    public function makeNull(): void {
        $this->foo = null;
    }
}
