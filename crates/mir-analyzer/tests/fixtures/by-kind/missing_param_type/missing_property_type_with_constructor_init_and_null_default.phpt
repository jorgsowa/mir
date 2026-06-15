===description===
Missing property type with constructor init and null default
===file===
<?php
class A {
    public $foo = null;

    public function __construct() {
        $this->foo = 5;
    }
}
===expect===
MissingPropertyType@3:4-3:22: Property A::$foo has no type annotation
