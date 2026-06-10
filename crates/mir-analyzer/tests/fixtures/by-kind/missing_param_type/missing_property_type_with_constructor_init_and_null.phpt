===description===
Missing property type with constructor init and null
===file===
<?php
class A {
    public $foo;

    public function __construct() {
        $this->foo = 5;
    }

    public function makeNull(): void {
        $this->foo = null;
    }
}
===expect===
MissingPropertyType@3:5-3:16: Property A::$foo has no type annotation
