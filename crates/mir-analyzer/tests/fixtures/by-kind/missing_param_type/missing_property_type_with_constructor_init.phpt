===description===
Missing property type with constructor init
===file===
<?php
class A {
    public $foo;

    public function __construct() {
        $this->foo = 5;
    }
}
===expect===
MissingPropertyType@3:5-3:16: Property A::$foo has no type annotation
