===description===
AbstractMethodCall does NOT fire when a concrete child calls self::foo() — self resolves to the child class whose foo() is concrete.
===file===
<?php
abstract class Base {
    abstract public function foo(): void;
}
class Child extends Base {
    public function foo(): void {}
    public function bar(): void {
        self::foo();
    }
}
===expect===
