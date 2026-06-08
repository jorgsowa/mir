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
ReadonlyPropertyAssignment@15:9-15:29: Cannot assign to readonly property B::$bar outside of constructor
