===description===
Readonly property set in constructor and also outside class
===file===
<?php
class A {
    public readonly string $bar;

    public function __construct() {
        $this->bar = "hello";
    }
}

$a = new A();
$a->bar = "goodbye";
===expect===
ReadonlyPropertyAssignment@11:0-11:19: Cannot assign to readonly property A::$bar outside of constructor
