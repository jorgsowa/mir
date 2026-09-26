===description===
Go-to-definition on an inherited method call lands on the declaring class's method.
===cursor===
definition
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {}
echo (new Child())->gre<CURSOR>et();
===expect===
test.php@3:4-3:52
