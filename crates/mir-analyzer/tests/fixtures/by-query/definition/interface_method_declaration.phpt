===description===
Go-to-definition on an interface method's declaration name resolves to the declaration itself.
===cursor===
definition
===file===
<?php
interface Shape {
    public function ar<CURSOR>ea(): float;
}
===expect===
test.php@3:4-3:34
