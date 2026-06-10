===description===
Inherit sealed methods with static
===file===
<?php
/**
 * @seal-methods
 */
class A {
    public static function __callStatic(string $method, array $args) {}
}

class B extends A {}
B::foo();
===expect===
