===description===
A class in an `instanceof` check lands on the class declaration.
===cursor===
definition
===file===
<?php
final class Greeter {}
function f(object $o): bool {
    return $o instanceof Gree<CURSOR>ter;
}
===expect===
test.php@2:6-2:22
