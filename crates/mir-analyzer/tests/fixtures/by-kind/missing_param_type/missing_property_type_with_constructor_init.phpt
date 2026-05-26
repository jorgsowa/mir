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
MissingPropertyType
===ignore===
TODO
