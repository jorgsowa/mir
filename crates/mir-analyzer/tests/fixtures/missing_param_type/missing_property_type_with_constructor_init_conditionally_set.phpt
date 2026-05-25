===description===
Missing property type with constructor init conditionally set
===file===
<?php
class A {
    public $foo;

    public function __construct() {
        if (rand(0, 1)) {
            $this->foo = 5;
        }
    }
}
===expect===
MissingPropertyType
===ignore===
TODO
