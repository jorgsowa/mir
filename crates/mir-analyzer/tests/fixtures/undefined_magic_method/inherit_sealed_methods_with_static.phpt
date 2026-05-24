===description===
inheritSealedMethodsWithStatic
===file===
<?php
/**
 * @psalm-seal-methods
 */
class A {
    public static function __callStatic(string $method, array $args) {}
}

class B extends A {}
B::foo();
===expect===
UndefinedMagicMethod
===ignore===
TODO
