===description===
References from an interface method's declaration name include the declaration and its call sites.
===cursor===
references include_declaration
===file===
<?php
interface Shape {
    public function ar<CURSOR>ea(): float;
}
function describe(Shape $s): float {
    return $s->area();
}
===expect===
test.php@3:20-3:24
test.php@6:15-6:19
