===description===
`parent::method()` resolves to the parent's method.
===cursor===
symbol
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {
    public function greet(): string { return parent::gr<CURSOR>eet() . '!'; }
}
===expect===
kind: static call Base::greet
type: string
