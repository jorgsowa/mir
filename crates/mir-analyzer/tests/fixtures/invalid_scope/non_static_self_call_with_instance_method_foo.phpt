===description===
Non static self call with instance method foo
===file===
<?php
class A {
    public function foo(): void {}

    // Has "magic methods"
    public function __call(string $method, array $args) {}
    public static function __callStatic(string $method, array $args) {}
}

class B extends A {
    public static function bar(): void {
        self::foo();
    }
}
===expect===
NonStaticSelfCall
===ignore===
TODO
