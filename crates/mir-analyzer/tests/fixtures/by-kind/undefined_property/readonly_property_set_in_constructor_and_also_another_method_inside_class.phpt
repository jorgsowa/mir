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
    }
}
===expect===
ReadonlyPropertyAssignment@13:9-13:31: Cannot assign to readonly property A::$bar outside of constructor
