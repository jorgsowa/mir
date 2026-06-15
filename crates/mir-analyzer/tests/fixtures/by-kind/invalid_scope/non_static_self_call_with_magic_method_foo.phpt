===description===
Non static self call with magic method foo
===file===
<?php
/**
 * @method string foo()
 */
class A {
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
NonStaticSelfCall@13:8-13:19: Non-static method B::foo() cannot be called statically
