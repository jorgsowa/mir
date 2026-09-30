===description===
Docblock readonly property set in constructor and also outside class
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

$a = new A();
$a->bar = "goodbye";
//<^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property A::$bar outside of constructor
===expect===
