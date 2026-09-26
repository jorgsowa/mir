===description===
The class in an `instanceof` check resolves to the class.
===cursor===
symbol
===file===
<?php
final class Greeter {}
function f(object $o): bool {
    return $o instanceof Gree<CURSOR>ter;
}
===expect===
kind: class Greeter
type: class-string
