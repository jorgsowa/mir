===description===
A first-class callable of a method lands on the method.
===cursor===
definition
===file===
<?php
final class Greeter {
    public function greet(): string { return 'hi'; }
}
$f = (new Greeter())->gr<CURSOR>eet(...);
===expect===
test.php@3:4-3:52
