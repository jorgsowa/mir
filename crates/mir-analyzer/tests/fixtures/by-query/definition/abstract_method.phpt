===description===
A call through an abstract-class type lands on the abstract method.
===cursor===
definition
===file===
<?php
abstract class Shape {
    abstract public function area(): float;
}
function describe(Shape $s): float {
    return $s->ar<CURSOR>ea();
}
===expect===
test.php@3:4-3:43
