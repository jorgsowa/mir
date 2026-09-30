===description===
Go-to-definition on an abstract method's declaration name resolves to the declaration itself.
===cursor===
definition
===file===
<?php
abstract class Shape {
    abstract public function ar<CURSOR>ea(): float;
}
===expect===
test.php@3:4-3:43
