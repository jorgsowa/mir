===description===
AbstractMethodCall does NOT fire when calling a concrete implementation.
===file===
<?php
abstract class Shape {
    abstract public function area(): float;
    public function describe(): string { return "shape"; }
}

class Circle extends Shape {
    public function area(): float { return 3.14; }
}

$c = new Circle();
echo $c->area();
===expect===
