===description===
staticInvocationWithMagicMethodFoo
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

A::foo();
===expect===
InvalidStaticInvocation
===ignore===
TODO
