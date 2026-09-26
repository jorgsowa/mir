===description===
Go-to-definition on `parent::method()` lands on the parent's method, not the override.
===cursor===
definition
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {
    public function greet(): string { return parent::gr<CURSOR>eet() . '!'; }
}
===expect===
test.php@3:4-3:52
