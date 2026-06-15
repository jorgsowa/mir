===description===
Readonly promoted property assign operator
===file===
<?php
class A {
    public function __construct(public readonly string $bar) {
    }
}

$a = new A("hello");
$a->bar = "goodbye";
===expect===
ReadonlyPropertyAssignment@8:0-8:19: Cannot assign to readonly property A::$bar outside of constructor
