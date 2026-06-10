===description===
undefinedMixinClassWithStaticMethodCall_WithMagicMethod
===file===
<?php
/**
 * @method baz()
 * @mixin B
 */
class A {
    public static function __callStatic(string $name, array $arguments) {}
}

A::foo();
===expect===
