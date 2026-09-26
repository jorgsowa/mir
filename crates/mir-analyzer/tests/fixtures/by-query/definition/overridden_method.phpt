===description===
A call on the subclass lands on the override, not the parent method.
===cursor===
definition
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {
    public function greet(): string { return 'hey'; }
}
echo (new Child())->gr<CURSOR>eet();
===expect===
test.php@6:4-6:53
