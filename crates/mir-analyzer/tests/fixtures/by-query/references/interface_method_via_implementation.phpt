===description===
A call typed against the implementing class counts as a reference to the interface method.
===cursor===
references
===file===
<?php
interface Shape {
    public function area(): float;
}
final class Square implements Shape {
    public function area(): float { return 1.0; }
}
function viaInterface(Shape $s): float { return $s->ar<CURSOR>ea(); }
function viaClass(Square $s): float { return $s->area(); }
===expect===
test.php@8:52-8:56
test.php@9:49-9:53
