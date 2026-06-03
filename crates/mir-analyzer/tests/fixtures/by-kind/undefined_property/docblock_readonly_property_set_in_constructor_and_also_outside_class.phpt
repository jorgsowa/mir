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
===expect===
ReadonlyPropertyAssignment@14:1-14:20: Cannot assign to readonly property A::$bar outside of constructor
