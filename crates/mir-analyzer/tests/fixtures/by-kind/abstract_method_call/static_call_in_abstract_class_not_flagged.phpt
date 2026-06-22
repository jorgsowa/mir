===description===
AbstractMethodCall does NOT fire for static:: calls on abstract methods — static:: uses LSB and resolves to the concrete subclass at runtime, which must implement the method.
===file===
<?php
abstract class Base {
    abstract public static function foo(): void;
    public static function bar(): void {
        static::foo();
    }
}
class Child extends Base {
    public static function foo(): void {}
}
Child::bar();
===expect===
