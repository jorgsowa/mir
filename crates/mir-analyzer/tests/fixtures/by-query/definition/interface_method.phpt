===description===
Go-to-definition on a call through an interface-typed parameter lands on the interface method.
===cursor===
definition
===file===
<?php
interface Shape {
    public function area(): float;
}
function describe(Shape $s): float {
    return $s->ar<CURSOR>ea();
}
===expect===
test.php@3:4-3:34
