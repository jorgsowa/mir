===description===
Readonly property set in constructor and also outside class with allow private
===file===
<?php
class A {
    /**
     * @readonly
     * @allow-private-mutation
     */
    public string $bar;

    public function __construct() {
        $this->bar = "hello";
    }

    public function setAgain() : void {
        $this->bar = "hello";
    }
}

$a = new A();
$a->bar = "goodbye";
===expect===
ReadonlyPropertyAssignment@14:9-14:29: Cannot assign to readonly property A::$bar outside of constructor
ReadonlyPropertyAssignment@19:1-19:20: Cannot assign to readonly property A::$bar outside of constructor
