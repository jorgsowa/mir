===description===
Readonly property set in constructor and also another method in subclass
===file===
<?php
class A {
    /**
     * @readonly
     */
    public string $bar;

    public function __construct() {
        $this->bar = "hello";
    }
}

class B extends A {
    public function setBar() : void {
        $this->bar = "hello";
    }
}
===expect===
InaccessibleProperty
===ignore===
TODO
