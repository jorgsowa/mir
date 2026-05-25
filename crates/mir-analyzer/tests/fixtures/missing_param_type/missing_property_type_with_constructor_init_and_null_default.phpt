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
MissingPropertyType
===ignore===
TODO
