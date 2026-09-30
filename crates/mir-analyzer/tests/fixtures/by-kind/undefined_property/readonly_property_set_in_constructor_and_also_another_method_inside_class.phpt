===description===
Readonly property set in constructor and also another method inside class
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

    public function setBar() : void {
        $this->bar = "goodbye";
//      ^^^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property A::$bar outside of constructor
    }
}
===expect===
