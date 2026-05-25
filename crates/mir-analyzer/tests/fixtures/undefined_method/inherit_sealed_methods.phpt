===description===
inheritSealedMethods
===file===
<?php
/**
 * @seal-methods
 */
class A {
    public function __call(string $method, array $args) {}
}

class B extends A {}

$b = new B();
$b->foo();
===expect===
UndefinedMagicMethod
===ignore===
TODO
