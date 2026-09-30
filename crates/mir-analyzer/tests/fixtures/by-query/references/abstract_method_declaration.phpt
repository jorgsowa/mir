===description===
References from an abstract method's declaration name include the declaration and its call sites.
===cursor===
references include_declaration
===file===
<?php
abstract class Shape {
    abstract public function ar<CURSOR>ea(): float;
}
function describe(Shape $s): float {
    return $s->area();
}
===expect===
test.php@3:29-3:33
test.php@6:15-6:19
