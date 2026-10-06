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
//      ^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property A::$bar outside of constructor
    }
}

$a = new A();
$a->bar = "goodbye";
//<^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property A::$bar outside of constructor
